use rusqlite::Connection;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, debug, trace, instrument};

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// SQLite database for usage tracking
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create or open database at the specified path
    pub fn new(path: &Path) -> Result<Self, DatabaseError> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        debug!("SQLite database connection opened at {:?}", path);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(metadata) = std::fs::metadata(path) {
                let mut perms = metadata.permissions();
                if perms.mode() & 0o777 != 0o600 {
                    perms.set_mode(0o600);
                    let _ = std::fs::set_permissions(path, perms);
                }
            }
        }

        let mut db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    #[instrument(skip(self))]
    pub fn init_schema(&mut self) -> Result<(), DatabaseError> {
        let tx = self.conn.transaction()?;

        tx.execute(
            "CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY
            )",
            [],
        )?;

        let version: i64 = tx.query_row(
            "SELECT MAX(version) FROM schema_version",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        trace!("Current database schema version: {}", version);

        if version < 1 {
            info!("Running schema migration from version {}", version);
            // Usage statistics table
            tx.execute(
                "CREATE TABLE IF NOT EXISTS usage_stats (
                    app_id TEXT PRIMARY KEY,
                    launch_count INTEGER DEFAULT 0,
                    last_used INTEGER DEFAULT 0,
                    total_time INTEGER DEFAULT 0,
                    created_at INTEGER DEFAULT 0
                )",
                [],
            )?;

            // Query selections table (for learning)
            tx.execute(
                "CREATE TABLE IF NOT EXISTS query_selections (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    query TEXT NOT NULL,
                    app_id TEXT NOT NULL,
                    timestamp INTEGER NOT NULL
                )",
                [],
            )?;

            // Create indices for performance
            tx.execute(
                "CREATE INDEX IF NOT EXISTS idx_query ON query_selections(query)",
                [],
            )?;

            tx.execute(
                "CREATE INDEX IF NOT EXISTS idx_timestamp ON query_selections(timestamp)",
                [],
            )?;
            
            tx.execute("INSERT INTO schema_version (version) VALUES (1)", [])?;
        }

        tx.commit()?;
        debug!("Database schema successfully initialized");
        Ok(())
    }

    /// Record an app launch
    #[instrument(skip(self))]
    pub fn record_launch(&mut self, app_id: &str) -> Result<(), DatabaseError> {
        if app_id.is_empty() || app_id.len() > 1024 {
            return Err(DatabaseError::ValidationError("Invalid app_id length".into()));
        }
        let now = current_timestamp();

        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO usage_stats (app_id, launch_count, last_used, created_at)
             VALUES (?1, 1, ?2, ?2)
             ON CONFLICT(app_id) DO UPDATE SET
                launch_count = launch_count + 1,
                last_used = ?2",
            rusqlite::params![app_id, now as i64],
        )?;
        tx.commit()?;
        debug!("Recorded launch tracking metric successfully for '{}'", app_id);

        Ok(())
    }

    /// Record a query → app selection
    #[instrument(skip(self))]
    pub fn record_selection(&mut self, query: &str, app_id: &str) -> Result<(), DatabaseError> {
        if app_id.is_empty() || app_id.len() > 1024 {
            return Err(DatabaseError::ValidationError("Invalid app_id length".into()));
        }
        if query.len() > 256 {
            return Err(DatabaseError::ValidationError("Query too long".into()));
        }
        let now = current_timestamp();

        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO query_selections (query, app_id, timestamp)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![query, app_id, now as i64],
        )?;
        tx.commit()?;
        trace!("Recorded selection tracking dynamically query '{}' => app_id '{}'", query, app_id);

        Ok(())
    }

    /// Get usage statistics for an app
    pub fn get_usage_stats(&self, app_id: &str) -> Result<Option<UsageStats>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare("SELECT launch_count, last_used FROM usage_stats WHERE app_id = ?1")?;

        let mut rows = stmt
            .query(rusqlite::params![app_id])?;

        if let Some(row) = rows.next()? {
            Ok(Some(UsageStats {
                launch_count: row.get(0)?,
                last_used: row.get::<_, i64>(1)? as u64,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get query selection statistics
    pub fn get_query_stats(&self, query: &str) -> Result<Vec<(String, u32)>, DatabaseError> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT app_id, COUNT(*) as count
                 FROM query_selections
                 WHERE query = ?1
                 GROUP BY app_id
                 ORDER BY count DESC
                 LIMIT 10",
            )?;

        let rows = stmt
            .query_map(rusqlite::params![query], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            })?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }

        Ok(results)
    }

    /// Clean old query selections (configurable retention in days)
    #[instrument(skip(self))]
    pub fn cleanup_old_selections(&mut self, days_old: u64) -> Result<(), DatabaseError> {
        let expiration_time = current_timestamp() - (days_old * 24 * 3600);
        info!("Executing database retention cleanup for items older than {} days", days_old);

        let tx = self.conn.transaction()?;
        let rows_deleted = tx.execute(
            "DELETE FROM query_selections WHERE timestamp < ?1",
            rusqlite::params![expiration_time as i64],
        )?;
        tx.commit()?;
        
        debug!("Database cleanup complete. {} old records purged.", rows_deleted);

        Ok(())
    }


}

/// Usage statistics for an app
#[derive(Debug, Clone)]
pub struct UsageStats {
    pub launch_count: u32,
    pub last_used: u64,
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}


