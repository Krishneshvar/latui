# Phase 1.1 Developer Quick Reference

## Overview
Multi-field indexing with weighted scoring for semantic search.

---

## Key Data Structures

### SearchableItem
```rust
pub struct SearchableItem {
    pub item: Item,                    // Original item
    pub name: String,                  // Weight: 10.0
    pub keywords: Vec<String>,         // Weight: 8.0
    pub categories: Vec<String>,       // Weight: 5.0
    pub generic_name: Option<String>,  // Weight: 6.0
    pub description: Option<String>,   // Weight: 3.0
    pub executable: String,            // Weight: 2.0
}
```

### SearchField
```rust
pub struct SearchField {
    pub text: String,
    pub weight: f64,
    pub field_type: FieldType,
}
```

---

## Scoring Algorithm

### Match Types & Scores
```rust
if field_text == query {
    score = 1000.0;  // Exact match
} else if field_text.starts_with(query) {
    score = 500.0;   // Prefix match
} else if field_text.split_whitespace().any(|w| w.starts_with(query)) {
    score = 300.0;   // Word boundary match
} else if field_text.contains(query) {
    score = 100.0;   // Substring match
} else {
    score = fuzzy_match(query, field_text);  // 0-200
}
```

### Final Score
```rust
weighted_score = match_score × field_weight
best_score = max(all_field_scores)
```

---

## Field Weights

| Field | Weight | Use Case |
|-------|--------|----------|
| Name | 10.0 | Exact app name matches |
| Keywords | 8.0 | Explicit search terms |
| Generic Name | 6.0 | Descriptive names |
| Categories | 5.0 | App classifications |
| Description | 3.0 | Detailed descriptions |
| Executable | 2.0 | Command names |

---

## API Usage

### Creating SearchableItem
```rust
let searchable = SearchableItem::new(
    item,
    name.to_lowercase(),
    keywords,
    categories,
    generic_name,
    description,
    executable,
);
```

### Getting Weighted Fields
```rust
let fields = searchable.get_weighted_fields();
for field in fields {
    let score = calculate_score(query, &field.text);
    let weighted = score * field.weight;
}
```

### Searching
```rust
let results = mode.search("browser");
// Returns Vec<Item> sorted by relevance
```

---

## Desktop Entry Parsing

### Keywords
```rust
let keywords: Vec<String> = entry
    .keywords::<&str>(&[])
    .map(|k| k.iter().map(|s| s.to_lowercase()).collect())
    .unwrap_or_default();
```

### Categories
```rust
let categories: Vec<String> = entry
    .categories()
    .map(|cats| cats.iter().map(|s| s.to_lowercase()).collect())
    .unwrap_or_default();
```

### Generic Name
```rust
let generic_name = entry
    .generic_name::<&str>(&[])
    .map(|g| g.to_lowercase());
```

### Description
```rust
let description = entry
    .comment::<&str>(&[])
    .map(|c| c.to_lowercase());
```

---

## Performance Tips

### Indexing
- Cache results in `~/.cache/latui/apps.json`
- Only rebuild when desktop files change
- Use `load_cache()` for fast startup

### Searching
- Early exit on exact matches
- Use best field score (not sum)
- Sort only final results

### Memory
- Store lowercase versions only
- Use `Option<String>` for optional fields
- Avoid cloning during search

---

## Common Patterns

### Adding a New Field
1. Add field to `SearchableItem`
2. Extract from desktop entry in `build_index()`
3. Add to `get_weighted_fields()` with weight
4. Update cache serialization

### Adjusting Weights
Edit `get_weighted_fields()` in `searchable_item.rs`:
```rust
fields.push(SearchField {
    text: self.name.clone(),
    weight: 10.0,  // Adjust this
    field_type: FieldType::Name,
});
```

### Adding Match Type
Edit `search()` in `apps.rs`:
```rust
if custom_match(field_text, query) {
    field_score = 250.0;  // New score
}
```

---

## Testing

### Unit Test Template
```rust
#[test]
fn test_field_weight() {
    let item = SearchableItem::new(/* ... */);
    let fields = item.get_weighted_fields();
    assert_eq!(fields[0].weight, 10.0);
}
```

### Integration Test
```rust
#[test]
fn test_semantic_search() {
    let mode = AppsMode::new();
    mode.load();
    let results = mode.search("browser");
    assert!(results.iter().any(|i| i.title.contains("Firefox")));
}
```

---

## Debugging

### Print Scores
```rust
println!("Query: {}, Item: {}, Score: {}", 
    query, searchable.name, best_score);
```

### Inspect Cache
```bash
jq '.apps[0]' ~/.cache/latui/apps.json
```

### Profile Search
```rust
let start = std::time::Instant::now();
let results = mode.search(query);
println!("Search took: {:?}", start.elapsed());
```

---

## Common Issues

### Issue: Keywords not matching
**Cause**: Keywords not extracted properly
**Fix**: Check `entry.keywords::<&str>(&[])` usage

### Issue: Low scores for good matches
**Cause**: Field weight too low
**Fix**: Increase weight in `get_weighted_fields()`

### Issue: Slow search
**Cause**: Too many fuzzy matches
**Fix**: Add early exit conditions

### Issue: Cache not loading
**Cause**: Serialization error
**Fix**: Check `SearchableItem` derives `Serialize`

---

## File Locations

```
src/
├── core/
│   └── searchable_item.rs    # Data structure
├── modes/
│   └── apps.rs               # Parsing & search
└── cache/
    └── apps_cache.rs         # Caching logic

~/.cache/latui/
└── apps.json                 # Cached data
```

---

## Quick Commands

```bash
# Build
cargo build --release

# Run
./target/release/latui

# Test
./test-phase-1.1.sh

# Clear cache
rm ~/.cache/latui/apps.json

# Check cache
jq '.apps | length' ~/.cache/latui/apps.json
```

---

**Last Updated**: Phase 1.1 Complete
**Next**: Phase 1.2 - Tokenization
