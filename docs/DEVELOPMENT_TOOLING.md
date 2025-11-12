# Development Tooling Setup

This document describes the industry-standard development tooling that has been added to the project to improve code quality, catch issues early, and save time in the future.

## Overview

The following tooling has been added:
1. **Pre-commit hooks** (Husky + lint-staged) - Automatically run linters and formatters before commits
2. **CI/CD pipeline** (GitHub Actions) - Automated testing and linting on push/PR
3. **Makefile** - Common development tasks in one place
4. **EditorConfig** - Consistent code formatting across editors

## Pre-commit Hooks

### What it does
- Automatically runs linters and formatters on staged files before each commit
- Prevents committing code with lint errors or formatting issues
- Only checks files that are actually being committed (via lint-staged)

### How it works
- Uses **Husky** to manage Git hooks
- Uses **lint-staged** to run commands only on staged files
- Configured in `package.json` under `lint-staged`

### What gets checked
- **Backend (Rust)**: `cargo fmt --check` and `cargo clippy`
- **Frontend (TypeScript)**: ESLint with auto-fix and TypeScript type checking

### Usage
The hooks run automatically when you commit. If checks fail, the commit is blocked.

To bypass (not recommended):
```bash
git commit --no-verify
```

## CI/CD Pipeline

### What it does
- Runs on every push to `main`/`develop` branches
- Runs on every pull request
- Ensures code quality before merging

### What gets tested
- **Backend**:
  - Code formatting (`cargo fmt --check`)
  - Linting (`cargo clippy`)
  - Build (`cargo build --release`)
  - Tests (`cargo test --release`)
- **Frontend**:
  - Linting (`npm run lint`)
  - Type checking (`npm run type-check`)
  - Tests (`npm test`)
  - Build (`npm run build`)

### Location
`.github/workflows/ci.yml`

## Makefile

### Purpose
Provides a single interface for common development tasks, making it easier to remember commands and maintain consistency.

### Available Commands

```bash
make help          # Show all available commands
make install       # Install all dependencies
make dev           # Start development servers
make build         # Build both backend and frontend
make test          # Run all tests
make lint          # Run all linters
make format        # Format all code
make type-check    # Run TypeScript type checking
make check         # Run all checks (lint + type-check + test)
make clean         # Clean build artifacts
```

### Individual Commands
You can also run commands for specific parts:
- `make test-backend` / `make test-frontend`
- `make lint-backend` / `make lint-frontend`
- `make format-backend` / `make format-frontend`

## EditorConfig

### What it does
Ensures consistent code formatting across different editors and IDEs.

### Settings
- UTF-8 encoding
- LF line endings
- Trailing whitespace removal
- Final newline insertion
- Indentation: 2 spaces for JS/TS/JSON, 4 spaces for Rust

### Location
`.editorconfig` (root directory)

## Root Package.json Scripts

The root `package.json` provides convenient scripts for running checks across the entire project:

```bash
npm run lint:backend        # Lint backend only
npm run lint:frontend       # Lint frontend only
npm run type-check:frontend # Type check frontend
npm run test:backend        # Test backend only
npm run test:frontend       # Test frontend only
npm run format:backend      # Format backend
npm run format:frontend     # Format frontend
npm run check:all           # Run all checks
```

## Benefits

1. **Catch issues early**: Pre-commit hooks prevent bad code from being committed
2. **Consistency**: EditorConfig ensures everyone uses the same formatting
3. **Automation**: CI/CD runs tests automatically, no manual checking needed
4. **Time savings**: Makefile provides quick access to common tasks
5. **Quality**: Automated checks ensure code quality standards are maintained

## Setup for New Developers

1. Clone the repository
2. Run `npm install` (installs Husky and sets up hooks)
3. Run `make install` (installs all dependencies)
4. Start coding! Hooks will run automatically on commit.

## Modularity

All tooling is kept modular:
- Pre-commit hooks are separate from CI/CD
- Makefile commands can be run independently
- Each tool can be configured separately
- Easy to extend or modify individual components

## Future Enhancements

Potential additions (when needed):
- Pre-push hooks for running tests
- Commit message linting (conventional commits)
- Dependency vulnerability scanning
- Code coverage reporting
- Automated dependency updates (Dependabot)

