# 🎯 LATUI Search System - Comprehensive Implementation Plan

> **Project:** latui - TUI Launcher for Wayland  
> **Goal:** Implement world-class search functionality comparable to Rofi, Alfred, and Raycast  
> **Status:** Planning Complete - Ready for Implementation

---

## 📊 Current State Analysis

### What We Have ✅
- Basic fuzzy matching with `nucleo-matcher`
- Simple prefix filtering
- Desktop entry parsing
- Caching system for applications
- Trie data structure (currently unused)
- Clean modular architecture

### What We're Missing ❌
- No semantic/keyword matching
- No multi-field search
- No typo tolerance beyond basic fuzzy
- No tokenization system
- No frequency/usage tracking
- Limited ranking algorithm
- Trie not integrated into search pipeline

### The Problem
When a user types "browser", they expect to see:
- Google Chrome
- Firefox
- Brave
- Chromium

But currently, these don't match because "browser" isn't in their names.

---

## 🚀 Implementation Roadmap

### Phase 1: Enhanced Search Architecture (Foundation)

#### 1.1 Multi-Field Indexing
**Problem:** Currently only searches `search_text` (lowercased name)  
**Solution:** Index multiple fields from desktop entries with different weights

**What to Implement:**
- Parse additional desktop entry fields:
  - `Keywords` - explicit keywords from .desktop file
  - `Categories` - app categories (Network, FileManager, etc.)
  - `GenericName` - generic description (e.g., "Web Browser")
  - `Comment` - app description
  - `Exec` - executable name
  
- Create weighted search fields:
  - **Name** (weight: 10.0) - exact app name
  - **Keywords** (weight: 8.0) - desktop file keywords
  - **Generic Name** (weight: 6.0) - e.g., "Web Browser"
  - **Categories** (weight: 5.0) - e.g., "Network", "FileManager"
  - **Description** (weight: 3.0) - app description
  - **Executable** (weight: 2.0) - command name

**Why This Matters:**
When you type "browser", it will match:
- Firefox → keywords: ["browser", "web", "internet"]
- Chrome → generic_name: "Web Browser"
- Brave → categories: ["Network", "WebBrowser"]

**Implementation Details:**
```rust
pub struct SearchableItem {
    pub item: Item,
    pub name_tokens: Vec<String>,
    pub keyword_tokens: Vec<String>,
    pub category_tokens: Vec<String>,
    pub generic_name_tokens: Vec<String>,
    pub description_tokens: Vec<String>,
    pub exec_tokens: Vec<String>,
}
```

---

#### 1.2 Tokenization System
**Problem:** Current search is monolithic string matching  
**Solution:** Break queries and text into meaningful tokens

**What to Implement:**
- Token extraction strategies:
  - Split on whitespace
  - Split on hyphens, underscores
  - Handle camelCase (e.g., "LibreOffice" → ["libre", "office"])
  - Extract acronyms (e.g., "Google Chrome" → ["gc"])
  - Unicode normalization (NFD/NFC)
  
- Token normalization:
  - Lowercase conversion
  - Accent removal (optional)
  - Stop word filtering (optional: "the", "a", "an")

**Examples:**
```
"Google Chrome" → ["google", "chrome", "gc"]
"file-manager" → ["file", "manager", "fm"]
"LibreOffice Writer" → ["libre", "office", "writer", "lo", "low"]
"GIMP" → ["gimp"]
"VLC Media Player" → ["vlc", "media", "player", "vmp"]
```

**Implementation Details:**
```rust
pub struct Tokenizer {
    // Extract all tokens from text
    pub fn tokenize(&self, text: &str) -> Vec<String>;
    
    // Extract acronym from multi-word text
    pub fn extract_acronym(&self, text: &str) -> Option<String>;
    
    // Normalize a single token
    pub fn normalize(&self, token: &str) -> String;
}
```

---

#### 1.3 Semantic Keyword Mapping
**Problem:** "browser" doesn't match "Chrome" or "Firefox"  
**Solution:** Build a keyword → app category mapping database

**What to Implement:**
- Static keyword database (TOML configuration file)
- Common synonyms and categories
- User-customizable mappings in `~/.config/latui/keywords.toml`
- Merge system defaults with user overrides

**Example Mappings:**
```toml
# ~/.config/latui/keywords.toml

[keywords]
browser = ["firefox", "chrome", "brave", "chromium", "epiphany", "falkon"]
editor = ["vim", "nvim", "neovim", "emacs", "code", "gedit", "kate", "nano"]
terminal = ["alacritty", "kitty", "wezterm", "konsole", "gnome-terminal", "terminator"]
files = ["thunar", "nautilus", "dolphin", "pcmanfm", "nemo", "ranger"]
music = ["spotify", "rhythmbox", "clementine", "audacious", "lollypop"]
video = ["vlc", "mpv", "celluloid", "totem", "dragon"]
email = ["thunderbird", "evolution", "geary", "mailspring"]
office = ["libreoffice", "onlyoffice", "calligra"]
image = ["gimp", "inkscape", "krita", "darktable"]
```

**Implementation Details:**
```rust
pub struct KeywordMapper {
    mappings: HashMap<String, Vec<String>>,
    
    // Load from default + user config
    pub fn load() -> Self;
    
    // Check if keyword matches app
    pub fn matches(&self, keyword: &str, app_name: &str) -> bool;
    
    // Get all apps matching a keyword
    pub fn get_matches(&self, keyword: &str) -> Vec<String>;
}
```

---

### Phase 2: Advanced Matching Algorithms

#### 2.1 Hybrid Scoring System
**Current:** Only nucleo fuzzy score  
**New:** Multi-algorithm scoring with configurable weights

**Scoring Components:**

1. **Exact Match** (score: 1000)
   - Query exactly matches field
   - Highest priority
   - Example: "firefox" → "Firefox"

2. **Prefix Match** (score: 500)
   - Field starts with query
   - Example: "fir" → "firefox"

3. **Word Boundary Match** (score: 300)
   - Query matches at word start
   - Example: "chrome" in "Google Chrome"

4. **Acronym Match** (score: 250)
   - Query matches acronym
   - Example: "gc" → "Google Chrome"

5. **Fuzzy Match** (score: 0-200)
   - Nucleo matcher score
   - Handles character skipping
   - Example: "frf" → "firefox"

6. **Keyword Match** (score: 150)
   - Semantic mapping match
   - Example: "browser" → "Firefox"

7. **Substring Match** (score: 100)
   - Query appears anywhere
   - Example: "fox" in "firefox"

8. **Typo Tolerance** (score: 50-150)
   - Levenshtein distance ≤ 2
   - Example: "firefix" → "firefox"

**Final Score Formula:**
```rust
total_score = (match_score × field_weight) + frequency_boost + recency_boost
```

**Implementation Details:**
```rust
pub struct HybridScorer {
    fuzzy_matcher: FuzzyMatcher,
    keyword_mapper: KeywordMapper,
    
    pub fn score(&mut self, query: &str, item: &SearchableItem) -> f64 {
        let mut best_score = 0.0;
        
        // Score each field with its weight
        for field in item.all_fields() {
            let field_score = self.score_field(query, field);
            let weighted = field_score * field.weight;
            best_score = best_score.max(weighted);
        }
        
        best_score
    }
    
    fn score_field(&mut self, query: &str, field: &SearchField) -> f64;
}
```

---

#### 2.2 Typo Tolerance (Levenshtein Distance)
**Problem:** "firefix" doesn't match "firefox"  
**Solution:** Calculate edit distance for near-misses

**What to Implement:**
- Levenshtein distance calculation
- Maximum distance: 2 edits
- Only apply for queries > 3 characters
- Penalize score based on distance
- Use as fallback when other methods fail

**Examples:**
```
"firefix" → "firefox" (distance: 1) ✓ score: 150
"chorme" → "chrome" (distance: 1) ✓ score: 150
"thuner" → "thunar" (distance: 1) ✓ score: 150
"fierfox" → "firefox" (distance: 2) ✓ score: 100
"fiirefox" → "firefox" (distance: 2) ✓ score: 100
```

**Implementation Details:**
```rust
pub struct TypoTolerance {
    max_distance: usize,
    min_query_length: usize,
    
    pub fn score(&self, query: &str, target: &str) -> Option<f64> {
        if query.len() < self.min_query_length {
            return None;
        }
        
        let distance = strsim::levenshtein(query, target);
        
        if distance <= self.max_distance {
            Some(match distance {
                0 => 1000.0, // exact match
                1 => 150.0,
                2 => 100.0,
                _ => 0.0,
            })
        } else {
            None
        }
    }
}
```

---

#### 2.3 Frequency & Recency Tracking
**Problem:** All apps have equal priority  
**Solution:** Track usage patterns and boost frequently/recently used apps

**What to Implement:**
- SQLite database for persistent usage stats
- Track metrics:
  - Launch count (total times opened)
  - Last used timestamp
  - Total usage time (optional)
  - Query → selection patterns
  
- Decay algorithm (recent usage weighted higher)
- Boost frequently used apps in rankings

**Database Schema:**
```sql
CREATE TABLE IF NOT EXISTS usage_stats (
    app_id TEXT PRIMARY KEY,
    launch_count INTEGER DEFAULT 0,
    last_used INTEGER DEFAULT 0,
    total_time INTEGER DEFAULT 0,
    created_at INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS query_selections (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    query TEXT NOT NULL,
    app_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL
);

CREATE INDEX idx_query ON query_selections(query);
CREATE INDEX idx_timestamp ON query_selections(timestamp);
```

**Boost Formula:**
```rust
// Logarithmic frequency boost (diminishing returns)
frequency_boost = (launch_count as f64 + 1.0).ln() * 20.0;

// Recency boost with time decay
let hours_since_use = (current_time - last_used) / 3600;
recency_boost = match hours_since_use {
    0..=1 => 50.0,      // Used in last hour
    2..=24 => 30.0,     // Used today
    25..=168 => 15.0,   // Used this week
    _ => 0.0,
};
```

**Implementation Details:**
```rust
pub struct UsageTracker {
    db: Connection,
    
    pub fn new(db_path: &Path) -> Result<Self>;
    
    pub fn record_launch(&mut self, app_id: &str);
    
    pub fn record_selection(&mut self, query: &str, app_id: &str);
    
    pub fn get_frequency_boost(&self, app_id: &str) -> f64;
    
    pub fn get_recency_boost(&self, app_id: &str) -> f64;
    
    pub fn get_query_preference(&self, query: &str) -> HashMap<String, f64>;
}
```

---

### Phase 3: Intelligent Ranking

#### 3.1 Context-Aware Ranking
**What to Implement:**
- Time-based context:
  - Work hours (9-5) → boost productivity apps
  - Evening → boost entertainment apps
  - Weekend → boost games, media
  
- Workspace context:
  - Track which apps are used together
  - Boost related apps when one is open
  
- Recent file context:
  - If user opened a .py file → boost Python IDEs
  - If user opened a .mp4 → boost video players

**Implementation Details:**
```rust
pub struct ContextAnalyzer {
    pub fn get_time_context(&self) -> TimeContext;
    pub fn get_workspace_context(&self) -> Vec<String>;
    pub fn get_recent_files(&self) -> Vec<PathBuf>;
    pub fn calculate_context_boost(&self, app: &Item) -> f64;
}
```

---

#### 3.2 Learning System
**What to Implement:**
- Track query → selection patterns
- If user types "br" and always selects "Brave", boost it
- Store learned preferences in database
- Decay old patterns over time

**Example:**
```
User types "br" 10 times:
- 8 times selects "Brave"
- 2 times selects "Chromium"

Next time "br" is typed:
- Brave gets +40 boost
- Chromium gets +10 boost
```

**Implementation Details:**
```rust
pub struct LearningSystem {
    tracker: UsageTracker,
    
    pub fn learn_from_selection(&mut self, query: &str, app_id: &str);
    
    pub fn get_learned_boost(&self, query: &str, app_id: &str) -> f64;
    
    pub fn get_top_selections(&self, query: &str, limit: usize) -> Vec<String>;
}
```

---

### Phase 4: Performance Optimizations

#### 4.1 Efficient Trie Usage
**Current:** Trie exists but unused  
**New:** Use for O(m) prefix filtering before expensive fuzzy matching

**What to Implement:**
- Build trie from all tokens (not just full names)
- Multi-token trie (each token indexed separately)
- Use trie for fast prefix lookup
- Only run fuzzy matching on trie results

**Performance Gain:**
```
Before: Fuzzy match 500 apps = ~5-10ms
After: Trie filter to 20 apps → fuzzy match = ~1-2ms
```

**Implementation Details:**
```rust
pub struct MultiTokenTrie {
    root: TrieNode,
    
    // Insert all tokens from an item
    pub fn insert_item(&mut self, item: &SearchableItem, index: usize);
    
    // Get candidate indices for query
    pub fn get_candidates(&self, query: &str) -> Vec<usize>;
    
    // Get candidates for each query token
    pub fn get_multi_token_candidates(&self, tokens: &[String]) -> Vec<usize>;
}
```

---

#### 4.2 Parallel Search
**What to Implement:**
- Use `rayon` for parallel scoring
- Split items into chunks
- Score each chunk in parallel thread
- Merge and sort results

**Performance Gain:**
```
Before: Sequential scoring = ~10ms
After: Parallel scoring (4 cores) = ~3ms
```

**Implementation Details:**
```rust
use rayon::prelude::*;

pub fn parallel_search(
    query: &str,
    items: &[SearchableItem],
    scorer: &HybridScorer,
) -> Vec<(usize, f64)> {
    items
        .par_iter()
        .enumerate()
        .filter_map(|(idx, item)| {
            let score = scorer.score(query, item);
            if score > 0.0 {
                Some((idx, score))
            } else {
                None
            }
        })
        .collect()
}
```

---

#### 4.3 Incremental Search
**What to Implement:**
- Cache previous query results
- When query extends (e.g., "fi" → "fir"), only search previous results
- Reduces search space dramatically

**Performance Gain:**
```
Query "f": Search 500 apps
Query "fi": Search 50 apps (from "f" results)
Query "fir": Search 10 apps (from "fi" results)
```

**Implementation Details:**
```rust
pub struct IncrementalSearchCache {
    last_query: String,
    last_results: Vec<usize>,
    
    pub fn should_use_cache(&self, new_query: &str) -> bool {
        new_query.starts_with(&self.last_query)
    }
    
    pub fn update(&mut self, query: String, results: Vec<usize>);
}
```

---

### Phase 5: Polish & UX Enhancements

#### 5.1 Match Highlighting
**What to Implement:**
- Return match indices from scoring
- Highlight matched characters in UI
- Use different colors for different match types:
  - Exact match: bright green
  - Prefix match: green
  - Fuzzy match: yellow
  - Keyword match: blue

**Implementation Details:**
```rust
pub struct MatchHighlight {
    pub indices: Vec<usize>,
    pub match_type: MatchType,
}

pub enum MatchType {
    Exact,
    Prefix,
    Fuzzy,
    Keyword,
    Typo,
}
```

---

#### 5.2 Search Modes
**What to Implement:**
- Special query prefixes for different search modes:
  - `^firefox` - Prefix mode (starts with)
  - `firefox$` - Suffix mode (ends with)
  - `'firefox'` - Exact mode (exact match only)
  - `/fire.*fox/` - Regex mode (advanced users)
  - `!browser` - Keyword-only mode

**Implementation Details:**
```rust
pub enum SearchMode {
    Normal,
    Prefix,
    Suffix,
    Exact,
    Regex(Regex),
    KeywordOnly,
}

pub fn parse_query(query: &str) -> (SearchMode, String);
```

---

## 📋 Implementation Priority

### Must Have (MVP) - Week 1
**Goal:** Make "browser" match Chrome, Firefox, etc.

1. ✅ **Multi-field indexing** - Parse keywords, categories, generic names
2. ✅ **Tokenization system** - Break text into searchable tokens
3. ✅ **Semantic keyword mapping** - Load keyword → app mappings
4. ✅ **Hybrid scoring** - Implement exact, prefix, fuzzy, keyword matching
5. ✅ **Trie-based prefix filtering** - Fast candidate selection

**Expected Result:**
- Typing "browser" shows all web browsers
- Typing "edit" shows all text editors
- Basic fuzzy matching still works

---

### Should Have - Week 2
**Goal:** Handle typos and learn user preferences

6. ✅ **Typo tolerance** - Levenshtein distance matching
7. ✅ **Frequency tracking** - SQLite database for usage stats
8. ✅ **Acronym matching** - "gc" → "Google Chrome"
9. ✅ **Parallel search** - Use rayon for faster scoring

**Expected Result:**
- "firefix" matches "firefox"
- Frequently used apps appear higher
- "gc" matches "Google Chrome"
- Search feels instant even with 500+ apps

---

### Nice to Have - Week 3
**Goal:** Polish and advanced features

10. ⚡ **Match highlighting** - Visual feedback on what matched
11. ⚡ **Learning system** - Remember query → selection patterns
12. ⚡ **Context-aware ranking** - Time-based and workspace-aware
13. ⚡ **Incremental search** - Cache and reuse previous results
14. ⚡ **Search modes** - Special prefixes for power users

**Expected Result:**
- Beautiful highlighted matches in UI
- Launcher learns your preferences
- Apps you need appear at the right time
- Power users can use advanced search syntax

---

## 🎯 Expected Results

### After Phase 1 (Multi-field + Keywords)
Typing **"browser"** will show:
1. Firefox (keyword match)
2. Google Chrome (generic name: "Web Browser")
3. Brave (category: "WebBrowser")
4. Chromium (keyword match)

### After Phase 2 (Typo + Frequency)
Typing **"gc"** will show:
1. Google Chrome (acronym match + high frequency)
2. GNOME Calculator (acronym match)

Typing **"firefix"** (typo) will show:
1. Firefox (typo tolerance, distance=1)

### After Phase 3 (Learning)
User types **"br"** and always selects Brave:
1. Brave (learned preference)
2. Chromium
3. Brasero

---

## 📦 Dependencies

### Current Dependencies
```toml
ratatui = "0.30.0"
crossterm = "0.29.0"
nucleo-matcher = "0.3.1"
serde = { version = "1.0.228", features = ["derive"] }
toml = "1.0.6"
anyhow = "1.0.102"
thiserror = "2.0.18"
tokio = { version = "1.50.0", features = ["full"] }
walkdir = "2.5.0"
xdg = "3.0.0"
freedesktop-desktop-entry = "0.8.1"
serde_json = "1.0.149"
```

### New Dependencies Needed
```toml
# Parallel processing
rayon = "1.10"

# Usage tracking database
rusqlite = { version = "0.32", features = ["bundled"] }

# Text normalization
unicode-normalization = "0.1"

# Levenshtein distance (already available via strsim)
strsim = "0.11"
```

---

## 🏗️ Proposed File Structure

### Current Structure
```
src/
├── main.rs
├── app/
│   ├── mod.rs
│   └── state.rs
├── cache/
│   ├── mod.rs
│   └── apps_cache.rs
├── core/
│   ├── mod.rs
│   ├── action.rs
│   ├── item.rs
│   └── mode.rs
├── index/
│   ├── mod.rs
│   └── trie.rs
├── matcher/
│   ├── mod.rs
│   └── fuzzy.rs
├── modes/
│   ├── mod.rs
│   └── apps.rs
└── ui/
    ├── mod.rs
    └── renderer.rs
```

### Proposed New Structure
```
src/
├── main.rs
│
├── app/
│   ├── mod.rs
│   └── state.rs
│
├── cache/
│   ├── mod.rs
│   └── apps_cache.rs
│
├── core/
│   ├── mod.rs
│   ├── action.rs
│   ├── item.rs              # Keep existing
│   ├── searchable_item.rs   # NEW: Multi-field item
│   └── mode.rs
│
├── search/                   # NEW MODULE
│   ├── mod.rs
│   ├── engine.rs            # Main search orchestrator
│   ├── tokenizer.rs         # Token extraction & normalization
│   ├── scorer.rs            # Hybrid scoring system
│   ├── keywords.rs          # Semantic keyword mappings
│   ├── typo.rs              # Typo tolerance (Levenshtein)
│   └── ranker.rs            # Final ranking with boosts
│
├── index/
│   ├── mod.rs
│   ├── trie.rs              # ENHANCE: Multi-token trie
│   ├── builder.rs           # NEW: Index building
│   └── inverted.rs          # NEW: Inverted index (optional)
│
├── tracking/                 # NEW MODULE
│   ├── mod.rs
│   ├── database.rs          # SQLite connection & schema
│   ├── frequency.rs         # Usage statistics
│   └── learning.rs          # Query → selection learning
│
├── matcher/
│   ├── mod.rs
│   └── fuzzy.rs             # Keep nucleo wrapper
│
├── modes/
│   ├── mod.rs
│   └── apps.rs              # REFACTOR: Use new search engine
│
├── config/                   # NEW MODULE
│   ├── mod.rs
│   ├── keywords.toml        # Default keyword mappings
│   └── loader.rs            # Config file loading
│
└── ui/
    ├── mod.rs
    └── renderer.rs          # ENHANCE: Add match highlighting
```

---

## 🔧 Configuration Files

### Default Keywords
**Location:** `src/config/keywords.toml` (embedded in binary)

```toml
[keywords]
browser = ["firefox", "chrome", "brave", "chromium", "epiphany", "falkon", "qutebrowser"]
editor = ["vim", "nvim", "neovim", "emacs", "code", "vscode", "gedit", "kate", "nano", "sublime"]
terminal = ["alacritty", "kitty", "wezterm", "konsole", "gnome-terminal", "terminator", "tilix"]
files = ["thunar", "nautilus", "dolphin", "pcmanfm", "nemo", "ranger", "mc"]
music = ["spotify", "rhythmbox", "clementine", "audacious", "lollypop", "deadbeef"]
video = ["vlc", "mpv", "celluloid", "totem", "dragon", "smplayer"]
email = ["thunderbird", "evolution", "geary", "mailspring", "kmail"]
office = ["libreoffice", "onlyoffice", "calligra", "abiword", "gnumeric"]
image = ["gimp", "inkscape", "krita", "darktable", "rawtherapee"]
pdf = ["okular", "evince", "zathura", "mupdf", "xreader"]
```

### User Overrides
**Location:** `~/.config/latui/keywords.toml`

Users can add their own mappings or override defaults.

---

## 📈 Performance Targets

### Search Latency
- **Empty query:** < 1ms (return all items)
- **Short query (1-2 chars):** < 5ms
- **Normal query (3-6 chars):** < 10ms
- **Complex query (7+ chars):** < 15ms

### Memory Usage
- **Base:** ~10MB (app data)
- **Index:** ~5MB (trie + inverted index)
- **Cache:** ~2MB (search results)
- **Total:** < 20MB

### Startup Time
- **Cold start:** < 100ms
- **With cache:** < 50ms

---

## 🧪 Testing Strategy

### Unit Tests
- Tokenizer: Test all edge cases
- Scorer: Test each scoring algorithm
- Typo tolerance: Test distance calculations
- Keyword mapper: Test mapping logic

### Integration Tests
- Full search pipeline
- Database operations
- Cache loading/saving

### Benchmark Tests
- Search performance with 100/500/1000 apps
- Parallel vs sequential scoring
- Trie vs linear search

---

## 🎬 Implementation Order

### Step 1: Refactor File Structure
- Create new directories
- Move existing files
- Update module declarations
- Ensure project still compiles

### Step 2: Core Search Infrastructure
- Implement `SearchableItem`
- Implement `Tokenizer`
- Implement `KeywordMapper`
- Load default keywords

### Step 3: Hybrid Scoring
- Implement exact match
- Implement prefix match
- Implement word boundary match
- Implement acronym match
- Integrate fuzzy matcher
- Implement keyword match

### Step 4: Index Enhancement
- Enhance trie for multi-token
- Build index from searchable items
- Integrate trie into search pipeline

### Step 5: Typo Tolerance
- Implement Levenshtein distance
- Add to scoring system
- Test with common typos

### Step 6: Usage Tracking
- Create SQLite schema
- Implement frequency tracking
- Implement recency tracking
- Add boosts to ranking

### Step 7: Parallel Search
- Add rayon dependency
- Implement parallel scoring
- Benchmark performance

### Step 8: Polish
- Add match highlighting
- Implement learning system
- Add context awareness
- Implement search modes

---

## 📚 References & Inspiration

### Similar Projects
- **Rofi** - X11/Wayland launcher
- **Alfred** - macOS launcher
- **Raycast** - Modern macOS launcher
- **Ulauncher** - Linux launcher
- **Albert** - Linux launcher

### Algorithms & Techniques
- **Fuzzy matching:** nucleo-matcher (used by Helix editor)
- **Levenshtein distance:** Edit distance for typo tolerance
- **TF-IDF:** Term frequency for ranking (optional)
- **BM25:** Ranking algorithm (optional, advanced)

### Libraries
- `nucleo-matcher` - Fast fuzzy matching
- `strsim` - String similarity metrics
- `rayon` - Data parallelism
- `rusqlite` - SQLite bindings

---

## 🎯 Success Metrics

### User Experience
- ✅ "browser" matches all web browsers
- ✅ Typos are handled gracefully
- ✅ Frequently used apps appear first
- ✅ Search feels instant (< 10ms)
- ✅ Results are relevant and predictable

### Technical
- ✅ < 20MB memory usage
- ✅ < 100ms cold start
- ✅ < 10ms search latency
- ✅ Scales to 1000+ apps
- ✅ 90%+ test coverage

### Adoption
- ✅ Users prefer it over Rofi/dmenu
- ✅ Positive feedback on speed
- ✅ Positive feedback on relevance
- ✅ Low bug reports
- ✅ Community contributions

---

## 🚀 Future Enhancements (Post-MVP)

### Additional Modes
- **Calculator mode:** Quick calculations
- **File search mode:** Find files quickly
- **Clipboard history:** Recent clipboard items
- **Window switcher:** Switch between open windows
- **SSH hosts:** Quick SSH connections
- **Bookmarks:** Browser bookmarks search

### Advanced Features
- **Plugins system:** User-defined modes
- **Themes:** Customizable UI
- **Hotkeys:** Custom keybindings
- **Multi-monitor:** Per-monitor positioning
- **Wayland protocols:** Better integration

### AI/ML Features
- **Semantic search:** Understand intent
- **Natural language:** "open my browser"
- **Predictive:** Suggest apps before typing
- **Clustering:** Group similar apps

---

## 📝 Notes

- Keep the codebase minimal and focused
- Prioritize performance over features
- Write tests for critical paths
- Document public APIs
- Use Rust idioms and best practices
- Avoid premature optimization
- Profile before optimizing
- Keep dependencies minimal

---

**Last Updated:** 2025-01-XX  
**Status:** Ready for Implementation  
**Next Step:** Refactor file structure
