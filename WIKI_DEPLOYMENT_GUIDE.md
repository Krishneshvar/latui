# GitHub Wiki Deployment Guide

This guide outlines the exact, step-by-step procedure to deploy the newly created LaTUI Markdown documentation (`ARCHITECTURE.md` and `CONFIGURATION.md`) into your GitHub Repository's Wiki.

## Prerequisites

Ensure that the Wiki feature is enabled on your GitHub repository:
1. Go to your `Krishneshvar/latui` repository on GitHub.
2. Click **Settings** > **Features**.
3. Ensure the **Wikis** checkbox is checked.

## Step-by-Step Deployment

Git allows you to treat the GitHub Wiki as its own standalone repository. This makes bulk uploading documentation incredibly easy.

### 1. Clone the Wiki Repository
Clone the `.wiki.git` repository adjacent to your main project folder.
```bash
cd ~/Desktop/programming/self-projects/latui
git clone https://github.com/Krishneshvar/latui.wiki.git
```

### 2. Copy the Documentation
Copy the markdown files we drafted from your main repository into the wiki directory:
```bash
cp latui/ARCHITECTURE.md latui.wiki/
cp latui/CONFIGURATION.md latui.wiki/
```

### 3. Setup Navigation Sidebar (Optional)
To provide a structured sidebar on the right side of the Wiki pages, create a specially named `_Sidebar.md` file inside the `latui.wiki` directory:
```bash
cat << 'EOF' > latui.wiki/_Sidebar.md
## LaTUI Documentation
- [Home](Home)
- [Configuration & Theming](CONFIGURATION)
- [Architecture & Internals](ARCHITECTURE)
EOF
```

### 4. Commit and Push
Navigate to the wiki folder, commit the new files, and push them to GitHub.
```bash
cd latui.wiki
git add .
git commit -m "docs: Add Architecture and Configuration Guides"
git push origin master
```

**Note:** GitHub Wikis traditionally use `master` as the default branch. If GitHub prompts you to use `main`, adjust the command accordingly.

## Linking from the Main README

In Phase 1, the `README.md` was already updated to gently direct users to the Wiki. Once the push above succeeds, the links in the repo (or directly pointing to `https://github.com/Krishneshvar/latui/wiki`) will become populated.
