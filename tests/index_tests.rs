use latui::index::trie::{Trie, MultiTokenTrie};
use latui::core::{item::Item, action::Action, searchable_item::SearchableItem};

fn create_test_item(name: &str, keywords: Vec<&str>, categories: Vec<&str>) -> SearchableItem {
    let item = Item {
        id: format!("test-{}", name),
        title: name.to_string(),
        search_text: name.to_lowercase(),
        description: None,
        action: Action::Launch("test".to_string()),
    };

    SearchableItem::new(
        item,
        name.to_string(),
        keywords.iter().map(|s| s.to_string()).collect(),
        categories.iter().map(|s| s.to_string()).collect(),
        None,
        None,
        name.to_lowercase(),
    ).unwrap()
}

#[test]
fn test_basic_trie_insert_and_search() {
    let mut trie = Trie::new();
    
    trie.insert("firefox", 0);
    trie.insert("chrome", 1);
    trie.insert("chromium", 2);
    
    // Test exact prefix match
    let results = trie.search("fire");
    assert_eq!(results, vec![0]);
    
    // Test prefix matching multiple items
    let results = trie.search("chro");
    assert!(results.contains(&1));
    assert!(results.contains(&2));
    
    // Test no match
    let results = trie.search("xyz");
    assert!(results.is_empty());
}

#[test]
fn test_trie_prefix_matching() {
    let mut trie = Trie::new();
    
    trie.insert("firefox", 0);
    trie.insert("firewall", 1);
    trie.insert("fire", 2);
    
    // All should match "fire"
    let results = trie.search("fire");
    assert_eq!(results.len(), 3);
    assert!(results.contains(&0));
    assert!(results.contains(&1));
    assert!(results.contains(&2));
    
    // Only firefox should match "firef"
    let results = trie.search("firef");
    assert_eq!(results, vec![0]);
}

#[test]
fn test_multi_token_trie_build() {
    let items = vec![
        create_test_item("Firefox", vec!["browser", "web"], vec!["Network"]),
        create_test_item("Google Chrome", vec!["browser"], vec!["Network"]),
        create_test_item("Thunar", vec!["files", "manager"], vec!["FileManager"]),
    ];

    let trie = MultiTokenTrie::build(&items);
    
    // Test searching for "fire" (should match Firefox)
    let candidates = trie.get_candidates("fire");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0], 0);
    
    // Test searching for "browser" (should match Firefox and Chrome)
    let candidates = trie.get_candidates("browser");
    assert_eq!(candidates.len(), 2);
    assert!(candidates.contains(&0));
    assert!(candidates.contains(&1));
    
    // Test searching for "files" (should match Thunar)
    let candidates = trie.get_candidates("files");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0], 2);
}

#[test]
fn test_multi_token_trie_acronyms() {
    let items = vec![
        create_test_item("Google Chrome", vec![], vec![]),
        create_test_item("GNOME Calculator", vec![], vec![]),
        create_test_item("Firefox", vec![], vec![]),
    ];

    let trie = MultiTokenTrie::build(&items);
    
    // Test acronym search "gc" (should match Google Chrome and GNOME Calculator)
    let candidates = trie.get_candidates("gc");
    assert_eq!(candidates.len(), 2);
    assert!(candidates.contains(&0));
    assert!(candidates.contains(&1));
}

#[test]
fn test_multi_token_trie_case_insensitive() {
    let items = vec![
        create_test_item("Firefox", vec![], vec![]),
        create_test_item("GIMP", vec![], vec![]),
    ];

    let trie = MultiTokenTrie::build(&items);
    
    // Test lowercase search
    let candidates = trie.get_candidates("fire");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0], 0);
    
    // Test lowercase search for uppercase name
    let candidates = trie.get_candidates("gimp");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0], 1);
}

#[test]
fn test_multi_token_candidates_all_match() {
    let items = vec![
        create_test_item("Firefox Browser", vec!["web"], vec![]),
        create_test_item("Google Chrome", vec!["browser"], vec![]),
        create_test_item("File Manager", vec![], vec![]),
    ];

    let trie = MultiTokenTrie::build(&items);
    
    // Test multi-token search where all tokens must match
    let tokens = vec!["fire".to_string(), "browser".to_string()];
    let candidates = trie.get_multi_token_candidates(&tokens);
    
    // Only Firefox Browser should match (has both "fire" and "browser")
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0], 0);
}

#[test]
fn test_multi_token_and_candidates() {
    let items = vec![
        create_test_item("Firefox Web Browser", vec!["browser"], vec![]),
        create_test_item("Chrome Web Browser", vec!["browser"], vec![]),
        create_test_item("Thunar File Manager", vec!["files"], vec![]),
    ];

    let trie = MultiTokenTrie::build(&items);
    
    // Test AND logic: all tokens must match
    let tokens = vec!["web".to_string(), "brow".to_string()];
    let candidates = trie.get_multi_token_candidates(&tokens);
    
    // Should match Firefox (0) and Chrome (1)
    assert_eq!(candidates.len(), 2);
    assert!(candidates.contains(&0));
    assert!(candidates.contains(&1));

    // Token that matches nothing
    let tokens = vec!["web".to_string(), "files".to_string()];
    let candidates = trie.get_multi_token_candidates(&tokens);
    assert!(candidates.is_empty());
}

#[test]
fn test_trie_empty_query() {
    let items = vec![
        create_test_item("Firefox", vec![], vec![]),
    ];

    let trie = MultiTokenTrie::build(&items);
    
    // Empty query should return empty results
    let candidates = trie.get_candidates("");
    assert!(candidates.is_empty());
}

#[test]
fn test_trie_no_duplicates() {
    let items = vec![
        create_test_item("Firefox Browser", vec!["firefox"], vec![]),
    ];

    let trie = MultiTokenTrie::build(&items);
    
    // "fire" matches both "firefox" token and "firefox" keyword
    // Should only return index once
    let candidates = trie.get_candidates("fire");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0], 0);
}

#[test]
fn test_trie_partial_token_match() {
    let items = vec![
        create_test_item("LibreOffice Writer", vec![], vec![]),
    ];

    let trie = MultiTokenTrie::build(&items);
    
    // Test partial matches
    let candidates = trie.get_candidates("lib");
    assert_eq!(candidates.len(), 1);
    
    let candidates = trie.get_candidates("off");
    assert_eq!(candidates.len(), 1);
    
    let candidates = trie.get_candidates("wri");
    assert_eq!(candidates.len(), 1);
}

#[test]
fn test_trie_category_matching() {
    let items = vec![
        create_test_item("Firefox", vec![], vec!["Network", "WebBrowser"]),
        create_test_item("Thunderbird", vec![], vec!["Network", "Email"]),
        create_test_item("Thunar", vec![], vec!["FileManager"]),
    ];

    let trie = MultiTokenTrie::build(&items);
    
    // Test category search
    let candidates = trie.get_candidates("network");
    assert_eq!(candidates.len(), 2);
    assert!(candidates.contains(&0));
    assert!(candidates.contains(&1));
    
    let candidates = trie.get_candidates("webbrowser");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0], 0);
}

#[test]
fn test_trie_performance_many_items() {
    // Create 100 test items
    let items: Vec<SearchableItem> = (0..100)
        .map(|i| create_test_item(
            &format!("App{}", i),
            vec!["test", "app"],
            vec!["Utility"],
        ))
        .collect();

    let trie = MultiTokenTrie::build(&items);
    
    // All items should match "app"
    let candidates = trie.get_candidates("app");
    assert_eq!(candidates.len(), 100);
    
    // Specific item should match
    let candidates = trie.get_candidates("app42");
    assert_eq!(candidates.len(), 1);
    assert_eq!(candidates[0], 42);
}
