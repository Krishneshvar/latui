# Contributing to LaTUI

First off, thank you for considering contributing to LaTUI! It's people like you that make LaTUI such a great tool.

Following these guidelines helps to communicate that you respect the time of the developers managing and developing this open-source project. In return, they should reciprocate that respect in addressing your issue, assessing changes, and helping you finalize your pull requests.

## How Can I Contribute?

### Reporting Bugs

This section guides you through submitting a bug report for LaTUI. Following these guidelines helps maintainers and the community understand your report, reproduce the behavior, and find related reports.

Before creating bug reports, please check [this list](#before-submitting-a-bug-report) to be sure that you need to create one. When you are creating a bug report, please [include as many details as possible](#how-do-i-submit-a-good-bug-report).

### Suggesting Enhancements

This section guides you through submitting an enhancement suggestion for LaTUI, including entirely new features and minor improvements to existing functionality. Following these guidelines helps maintainers and the community understand your suggestion and find related suggestions.

When you are creating an enhancement suggestion, please [include as many details as possible](#how-do-i-submit-a-good-enhancement-suggestion).

### Pull Requests

*   Fill in [the pull request template](.github/PULL_REQUEST_TEMPLATE.md) (if available).
*   Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/README.html).
*   Include tests if applicable.
*   Document new functionality.
*   Ensure that any new logic follows the **Strategy Pattern** for modes.

## Styleguides

### Rust Styleguide

All Rust code should be formatted with `rustfmt`. You can run it with:

```bash
cargo fmt
```

### Git Commit Messages

*   Use the present tense ("Add feature" not "Added feature").
*   Use the imperative mood ("Move cursor to..." not "Moves cursor to...").
*   Limit the first line to 72 characters or less.
*   Reference issues and pull requests liberally after the first line.

## Core Architecture Rules

If you are adding a new mode, please ensure:
1.  It implements the `Mode` trait.
2.  It uses the `tracking` module if it needs to rank results by frequency.
3.  It handles errors gracefully using `thiserror` and `anyhow`.
4.  It doesn't block the main UI thread with heavy I/O (use `rayon` or async if necessary).

Thank you for your contributions!
