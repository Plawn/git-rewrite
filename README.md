# Git Rewrite

A modern Git history viewer and editor built with Tauri, SvelteKit, and TypeScript.

## Features

- View commit history with a visual git graph
- Edit commit messages
- View commit diffs with syntax highlighting
- Search within diffs
- Squash commits
- Light/dark mode with iOS-style transparency effects

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Development

```bash
# Install dependencies
bun install

# Run in development mode
bun run tauri dev

# Build for production
bun run tauri build
```

## Updating the App Icon

To change the application icon:

1. Place your icon file in the project root as `icon.png`
   - Recommended size: 1024x1024 pixels or larger
   - Format: PNG with transparency support

2. Run the Tauri icon generator:
   ```bash
   bun run tauri icon icon.png
   ```

3. This will generate all required icon formats in `src-tauri/icons/`:
   - `icon.icns` - macOS
   - `icon.ico` - Windows
   - Various PNG sizes for different platforms
   - iOS and Android icons

4. Rebuild the app to apply the new icon:
   ```bash
   bun run tauri build
   ```

The icons are automatically configured in `src-tauri/tauri.conf.json`.
