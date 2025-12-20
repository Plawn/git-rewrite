# Git Rewrite - Justfile

# Default recipe: show available commands
default:
    @just --list

# Install dependencies
install:
    bun install

# Run development server
dev:
    bun run tauri dev

# Build the application for production
build:
    bun run tauri build

# Build debug version
build-debug:
    bun run tauri build --debug

# Clean build artifacts
clean:
    rm -rf build
    rm -rf src-tauri/target

# Clean and rebuild
rebuild: clean build

# Check Rust code
check:
    cd src-tauri && cargo check

# Format Rust code
fmt:
    cd src-tauri && cargo fmt

# Lint Rust code
clippy:
    cd src-tauri && cargo clippy

# Run all checks (format, clippy, check)
lint: fmt clippy check
