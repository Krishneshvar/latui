use latui::tracking::database::Database;
use latui::tracking::frequency::FrequencyTracker;
use std::fs;
use std::path::PathBuf;

fn temp_db_path(prefix: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("latui_{}_test_{}_{}.db", prefix, std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    path
}

#[test]
fn test_database_creation() {
    let path = temp_db_path("db");
    let db = Database::new(&path);
    assert!(db.is_ok());
    let _ = fs::remove_file(&path);
}

#[test]
fn test_record_launch() {
    let path = temp_db_path("launch");
    let mut db = Database::new(&path).unwrap();

    assert!(db.record_launch("firefox").is_ok());
    assert!(db.record_launch("firefox").is_ok());

    let stats = db.get_usage_stats("firefox").unwrap();
    assert!(stats.is_some());
    assert_eq!(stats.unwrap().launch_count, 2);

    let _ = fs::remove_file(&path);
}

#[test]
fn test_record_selection() {
    let path = temp_db_path("selection");
    let mut db = Database::new(&path).unwrap();

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
fn test_frequency_boost() {
    let path = temp_db_path("freq");
    let mut tracker = FrequencyTracker::new(&path).unwrap();

    // No launches = no boost
    assert_eq!(tracker.get_frequency_boost("firefox"), 0.0);

    // Record some launches
    for _ in 0..10 {
        tracker.record_launch("firefox").unwrap();
    }

    let boost = tracker.get_frequency_boost("firefox");
    assert!(boost > 0.0);
    assert!(boost < 100.0);

    let _ = fs::remove_file(&path);
}

#[test]
fn test_recency_boost() {
    let path = temp_db_path("recency");
    let mut tracker = FrequencyTracker::new(&path).unwrap();

    // Record a launch (will have current timestamp)
    tracker.record_launch("firefox").unwrap();

    // Should get high recency boost
    let boost = tracker.get_recency_boost("firefox");
    assert!(boost >= 40.0); // Should be in 0-1 hour or 2-6 hour range

    let _ = fs::remove_file(&path);
}

#[test]
fn test_query_boost() {
    let path = temp_db_path("query");
    let mut tracker = FrequencyTracker::new(&path).unwrap();

    // Record selections
    tracker.record_selection("br", "brave").unwrap();
    tracker.record_selection("br", "brave").unwrap();
    tracker.record_selection("br", "brave").unwrap();
    tracker.record_selection("br", "chromium").unwrap();

    // Brave should get higher boost (75% of selections)
    let brave_boost = tracker.get_query_boost("br", "brave");
    let chromium_boost = tracker.get_query_boost("br", "chromium");

    assert!(brave_boost > chromium_boost);
    assert!(brave_boost > 30.0); // 75% → 37.5 points

    let _ = fs::remove_file(&path);
}

#[test]
fn test_total_boost() {
    let path = temp_db_path("total");
    let mut tracker = FrequencyTracker::new(&path).unwrap();

    tracker.record_launch("firefox").unwrap();

    let total = tracker.get_total_boost("firefox");
    let freq = tracker.get_frequency_boost("firefox");
    let rec = tracker.get_recency_boost("firefox");

    assert_eq!(total, freq + rec);

    let _ = fs::remove_file(&path);
}

#[test]
fn test_cleanup_old_selections() {
    let path = temp_db_path("cleanup");
    let mut db = Database::new(&path).unwrap();

    db.record_selection("br", "brave").unwrap();
    db.record_selection("br", "chromium").unwrap();

    // Backdate one row so cleanup can deterministically remove it.
    let conn = rusqlite::Connection::open(&path).unwrap();
    conn.execute(
        "UPDATE query_selections SET timestamp = 1 WHERE app_id = 'chromium'",
        [],
    )
    .unwrap();

    db.cleanup_old_selections(30).unwrap();
    let stats = db.get_query_stats("br").unwrap();

    assert_eq!(stats.len(), 1);
    assert_eq!(stats[0].0, "brave");
    assert_eq!(stats[0].1, 1);

    let _ = fs::remove_file(&path);
}
