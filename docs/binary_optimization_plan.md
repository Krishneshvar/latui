# Binary Size Optimization Plan

This document outlines the strategy for reducing LaTUI's binary size. Currently, the binary is >6MB, while traditional C-based launchers like Rofi are significantly smaller. Our goal is to reach a footprint of **2MB - 3MB** without compromising on the safety or performance that Rust provides.

---

## 🏗️ 1. Compiler-Level Optimizations (Items 2 & 3)

The most immediate wins come from telling the Rust compiler (`rustc`) to prioritize binary size and perform more aggressive dead-code elimination.

### Configuration ([Cargo.toml](file:///home/krish/Desktop/programming/self-projects/latui/Cargo.toml))
We will add a `[profile.release]` section to your [Cargo.toml](file:///home/krish/Desktop/programming/self-projects/latui/Cargo.toml):

```toml
[profile.release]
# Link-Time Optimization (LTO)
# Aggressively removes unused code across all dependencies (crates).
lto = true

# Codegen Units
# Setting this to 1 allows for better optimization at the cost of slower build times.
codegen-units = 1

# Optimization Level
# "z" optimizes specifically for binary size. "s" is a slightly less aggressive alternative.
opt-level = "z"

# Strip Symbols
# Removes debug info and symbol tables, which often account for 1-2MB of the binary.
strip = true

# Panic behavior
# Optional: aborting on panic can save space by removing stack-unwinding code.
# panic = "abort"
```

---

## 📦 2. Dependency Refinement (Item 4)

Reducing the "bloat" from our dependencies is the second major step.

### A. Refining `tokio`
Currently, we use `features = ["full"]`. This pulls in networking, signal handling, and a heavy-duty multi-threaded scheduler that LaTUI likely doesn't need for a TUI interface.

**Target Action:**
Limit `tokio` to just the essential features (e.g., `rt`, `sync`, `macros`).

### B. Decoupling `rusqlite`
We are currently using `features = ["bundled"]`. This compiles the entire SQLite C source into our binary (adding ~2MB).

**Target Action:**
1. Use the system's SQLite library instead of bundling it.
2. In [Cargo.toml](file:///home/krish/Desktop/programming/self-projects/latui/Cargo.toml), change `rusqlite` to use the default features or `libsqlite3-sys`.

---

## 🔍 3. Analysis & Measurement

To ensure our changes are effective, we should use specialized tools to see "who" is taking up space.

### Tools to use:
1. **`cargo bloat`**: Lists the largest functions and dependencies in your binary.
   ```bash
   cargo install cargo-bloat
   cargo bloat --release
   ```
2. **`twiggy`**: (For Wasm/Generic) or **`nm -S --size-sort`**: To inspect symbol sizes manually.

---

## 📈 4. Expected Impact

| Step | Current Size | Expected Reduction | New Total (Est) |
| :--- | :--- | :--- | :--- |
| **Baseline** | ~6.5 MB | - | 6.5 MB |
| **LTO + Strip** | - | -2.5 MB | 4.0 MB |
| **Opt-level "z"** | - | -0.5 MB | 3.5 MB |
| **Un-bundle SQLite** | - | -1.5 MB | 2.0 MB |
| **Tokio Refinement** | - | -0.5 MB | **1.5 MB** |

---

> [!IMPORTANT]
> Some optimizations (like `panic = "abort"`) might change how the application handles fatal errors. We should verify if we want to retain full backtraces for production debugging before enabling that specific flag.
