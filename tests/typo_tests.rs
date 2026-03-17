use latui::search::typo::TypoTolerance;

#[test]
fn test_exact_match() {
    let typo = TypoTolerance::new();
    assert_eq!(typo.distance("firefox", "firefox"), 0);
    assert_eq!(typo.score("firefox", "firefox"), Some(1000.0));
}

#[test]
fn test_single_typo() {
    let typo = TypoTolerance::new();

    // Substitution: firefix -> firefox
    assert_eq!(typo.distance("firefix", "firefox"), 1);
    assert_eq!(typo.score("firefix", "firefox"), Some(150.0));

    // Deletion: fierfox -> firefox
    assert_eq!(typo.distance("fierfox", "firefox"), 1);

    // Insertion: firefx -> firefox
    assert_eq!(typo.distance("firefx", "firefox"), 1);
}

#[test]
fn test_double_typo() {
    let typo = TypoTolerance::new();

    // Two substitutions: fiirefox -> firefox
    // Note: Damerau-Levenshtein might optimize this differently
    let distance = typo.distance("fiirefox", "firefox");
    assert!(distance <= 2, "Distance should be <= 2, got {}", distance);
    assert!(typo.score("fiirefox", "firefox").is_some());
}

#[test]
fn test_transposition() {
    let typo = TypoTolerance::new();

    // Damerau-Levenshtein handles transpositions
    // "teh" -> "the" should be distance 1, not 2
    assert_eq!(typo.distance("teh", "the"), 1);

    // "chorme" -> "chrome"
    assert_eq!(typo.distance("chorme", "chrome"), 1);
}

#[test]
fn test_min_query_length() {
    let typo = TypoTolerance::new();

    // Query too short (< 4 chars)
    assert_eq!(typo.score("fir", "firefox"), None);

    // Query exactly 4 chars (should work)
    assert!(typo.score("fire", "fire").is_some());

    // Query long enough with typo
    assert!(typo.score("firefo", "firefox").is_some());
}

#[test]
fn test_max_distance() {
    let typo = TypoTolerance::new();

    // Distance 3 (too far)
    assert_eq!(typo.score("fiiireefox", "firefox"), None);

    // Distance 2 (within limit)
    assert!(typo.score("fiirefox", "firefox").is_some());
}
