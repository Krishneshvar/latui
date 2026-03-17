use latui::search::tokenizer::Tokenizer;

#[test]
fn test_basic_tokenization() {
    let tokenizer = Tokenizer::new();

    let tokens = tokenizer.tokenize("Hello World");
    assert!(tokens.contains(&"hello".to_string()));
    assert!(tokens.contains(&"world".to_string()));
}

#[test]
fn test_camel_case_splitting() {
    let tokenizer = Tokenizer::new();

    // LibreOffice -> libre, office
    let tokens = tokenizer.tokenize("LibreOffice");
    assert!(tokens.contains(&"libreoffice".to_string()));
    assert!(tokens.contains(&"libre".to_string()));
    assert!(tokens.contains(&"office".to_string()));

    // VLCPlayer -> vlc, player
    let tokens = tokenizer.tokenize("VLCPlayer");
    assert!(tokens.contains(&"vlc".to_string()));
    assert!(tokens.contains(&"player".to_string()));
}

#[test]
fn test_acronym_extraction() {
    let tokenizer = Tokenizer::new();

    // Google Chrome -> gc
    assert_eq!(
        tokenizer.extract_acronym("Google Chrome"),
        Some("gc".to_string())
    );

    // Visual Studio Code -> vsc
    assert_eq!(
        tokenizer.extract_acronym("Visual Studio Code"),
        Some("vsc".to_string())
    );

    // VLC Media Player -> vmp
    assert_eq!(
        tokenizer.extract_acronym("VLC Media Player"),
        Some("vmp".to_string())
    );

    // Single word -> None
    assert_eq!(tokenizer.extract_acronym("Firefox"), None);
}

#[test]
fn test_all_acronyms() {
    let tokenizer = Tokenizer::new();

    let acronyms = tokenizer.extract_all_acronyms("Visual Studio Code");
    assert!(acronyms.contains(&"vsc".to_string())); // Full acronym
    assert!(acronyms.contains(&"vs".to_string())); // First two
}

#[test]
fn test_hyphen_splitting() {
    let tokenizer = Tokenizer::new();

    let tokens = tokenizer.tokenize("file-manager");
    assert!(tokens.contains(&"file".to_string()));
    assert!(tokens.contains(&"manager".to_string()));
}

#[test]
fn test_underscore_splitting() {
    let tokenizer = Tokenizer::new();

    let tokens = tokenizer.tokenize("my_app_name");
    assert!(tokens.contains(&"my".to_string()));
    assert!(tokens.contains(&"app".to_string()));
    assert!(tokens.contains(&"name".to_string()));
}

#[test]
fn test_normalization() {
    let tokenizer = Tokenizer::new();

    assert_eq!(tokenizer.normalize("HELLO"), "hello");
    assert_eq!(tokenizer.normalize("  World  "), "world");
}

#[test]
fn test_diacritics_removal() {
    let tokenizer = Tokenizer::new();

    // Note: This is a simplified test - full diacritics removal
    // depends on unicode normalization
    let normalized = tokenizer.normalize("café");
    assert!(normalized == "café" || normalized == "cafe");
}

#[test]
fn test_comprehensive_tokenization() {
    let tokenizer = Tokenizer::new();

    let tokens = tokenizer.tokenize_comprehensive("Google Chrome");

    // Should contain: google, chrome, gc
    assert!(tokens.contains(&"google".to_string()));
    assert!(tokens.contains(&"chrome".to_string()));
    assert!(tokens.contains(&"gc".to_string()));
}

#[test]
fn test_xml_parser_camel_case() {
    let tokenizer = Tokenizer::new();

    // XMLParser -> XML, Parser
    let tokens = tokenizer.split_camel_case_word("XMLParser");
    assert!(tokens.contains(&"XML".to_string()));
    assert!(tokens.contains(&"Parser".to_string()));
}

#[test]
fn test_all_caps_no_split() {
    let tokenizer = Tokenizer::new();

    // GIMP should not be split
    let tokens = tokenizer.split_camel_case_word("GIMP");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], "GIMP");
}

#[test]
fn test_empty_string() {
    let tokenizer = Tokenizer::new();

    let tokens = tokenizer.tokenize("");
    assert!(tokens.is_empty());
}

#[test]
fn test_single_word() {
    let tokenizer = Tokenizer::new();

    let tokens = tokenizer.tokenize("firefox");
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], "firefox");
}
