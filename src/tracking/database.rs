use rusqlite::Connection;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// SQLite database for usage tracking
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create or open database at the specified path
    pub fn new(path: &Path) -> Result<Self, String> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create database directory: {}", e))?;
        }

        let conn = Connection::open(path)
            .map_err(|e| format!("Failed to open database: {}", e))?;

        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    pub fn init_schema(&self) -> Result<(), String> {
        // Usage statistics table
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS usage_stats (
                    app_id TEXT PRIMARY KEY,
                    launch_count INTEGER DEFAULT 0,
                    last_used INTEGER DEFAULT 0,
                    total_time INTEGER DEFAULT 0,
                    created_at INTEGER DEFAULT 0
                )",
                [],
            )
            .map_err(|e| format!("Failed to create usage_stats table: {}", e))?;

        // Query selections table (for learning)
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS query_selections (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    query TEXT NOT NULL,
                    app_id TEXT NOT NULL,
                    timestamp INTEGER NOT NULL
                )",
                [],
            )
            .map_err(|e| format!("Failed to create query_selections table: {}", e))?;

        // Create indices for performance
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_query ON query_selections(query)",
                [],
            )
            .map_err(|e| format!("Failed to create query index: {}", e))?;

        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_timestamp ON query_selections(timestamp)",
                [],
            )
            .map_err(|e| format!("Failed to create timestamp index: {}", e))?;

        Ok(())
    }

    /// Record an app launch
    pub fn record_launch(&self, app_id: &str) -> Result<(), String> {
        let now = current_timestamp();

        self.conn
            .execute(
                "INSERT INTO usage_stats (app_id, launch_count, last_used, created_at)
                 VALUES (?1, 1, ?2, ?2)
                 ON CONFLICT(app_id) DO UPDATE SET
                    launch_count = launch_count + 1,
                    last_used = ?2",
                rusqlite::params![app_id, now],
            )
            .map_err(|e| format!("Failed to record launch: {}", e))?;

        Ok(())
    }

    /// Record a query → app selection
    pub fn record_selection(&self, query: &str, app_id: &str) -> Result<(), String> {
        let now = current_timestamp();

        self.conn
            .execute(
                "INSERT INTO query_selections (query, app_id, timestamp)
                 VALUES (?1, ?2, ?3)",
                rusqlite::params![query, app_id, now],
            )
            .map_err(|e| format!("Failed to record selection: {}", e))?;

        Ok(())
    }

    /// Get usage statistics for an app
    pub fn get_usage_stats(&self, app_id: &str) -> Result<Option<UsageStats>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT launch_count, last_used FROM usage_stats WHERE app_id = ?1")
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let mut rows = stmt
            .query(rusqlite::params![app_id])
            .map_err(|e| format!("Failed to query usage stats: {}", e))?;

        if let Some(row) = rows.next().map_err(|e| format!("Failed to get row: {}", e))? {
            Ok(Some(UsageStats {
                launch_count: row.get(0).map_err(|e| format!("Failed to get launch_count: {}", e))?,
                last_used: row.get(1).map_err(|e| format!("Failed to get last_used: {}", e))?,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get query selection statistics
    pub fn get_query_stats(&self, query: &str) -> Result<Vec<(String, u32)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT app_id, COUNT(*) as count
                 FROM query_selections
                 WHERE query = ?1
                 GROUP BY app_id
                 ORDER BY count DESC
                 LIMIT 10",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map(rusqlite::params![query], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            })
            .map_err(|e| format!("Failed to query selections: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
        }

        Ok(results)
    }

    /// Clean old query selections (older than 30 days)
    pub fn cleanup_old_selections(&self) -> Result<(), String> {
        let thirty_days_ago = current_timestamp() - (30 * 24 * 3600);

        self.conn
            .execute(
                "DELETE FROM query_selections WHERE timestamp < ?1",
                rusqlite::params![thirty_days_ago],
            )
            .map_err(|e| format!("Failed to cleanup old selections: {}", e))?;

        Ok(())
    }

    /// Get all apps sorted by launch count
    pub fn get_top_apps(&self, limit: usize) -> Result<Vec<(String, u32)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT app_id, launch_count
                 FROM usage_stats
                 ORDER BY launch_count DESC
                 LIMIT ?1",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            })
            .map_err(|e| format!("Failed to query top apps: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
        }

        Ok(results)
    }

    /// Get recently used apps
    pub fn get_recent_apps(&self, limit: usize) -> Result<Vec<(String, u64)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT app_id, last_used
                 FROM usage_stats
                 WHERE last_used > 0
                 ORDER BY last_used DESC
                 LIMIT ?1",
            )
            .map_err(|e| format!("Failed to prepare statement: {}", e))?;

        let rows = stmt
            .query_map(rusqlite::params![limit], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u64>(1)?))
            })
            .map_err(|e| format!("Failed to query recent apps: {}", e))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| format!("Failed to read row: {}", e))?);
        }

        Ok(results)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_db_path() -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("latui_test_{}_{}.db", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        path
    }

    #[test]
    fn test_database_creation() {
        let path = temp_db_path();
        let db = Database::new(&path);
        assert!(db.is_ok());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_record_launch() {
        let path = temp_db_path();
        let db = Database::new(&path).unwrap();

        assert!(db.record_launch("firefox").is_ok());
        assert!(db.record_launch("firefox").is_ok());

        let stats = db.get_usage_stats("firefox").unwrap();
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().launch_count, 2);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_record_selection() {
        let path = temp_db_path();
        let db = Database::new(&path).unwrap();

        assert!(db.record_selection("br", "brave").is_ok());
        assert!(db.record_selection("br", "brave").is_ok());
        assert!(db.record_selection("br", "chromium").is_ok());

        let stats = db.get_query_stats("br").unwrap();
        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].0, "brave");
        assert_eq!(stats[0].1, 2);

        let _ = fs::remove_file(&path);
    }

    #[test]
    fn test_top_apps() {
        let path = temp_db_path();
        let db = Database::new(&path).unwrap();

        db.record_launch("firefox").unwrap();
        db.record_launch("firefox").unwrap();
        db.record_launch("firefox").unwrap();
        db.record_launch("chrome").unwrap();
        db.record_launch("chrome").unwrap();
        db.record_launch("brave").unwrap();

        let top = db.get_top_apps(3).unwrap();
        assert_eq!(top.len(), 3);
        assert_eq!(top[0].0, "firefox");
        assert_eq!(top[0].1, 3);

        let _ = fs::remove_file(&path);
    }
}
