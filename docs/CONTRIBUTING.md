# Contributing Guide

Thank you for your interest in contributing to LiteTerm-Web! This guide will help you get started with development.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Project Structure](#project-structure)
- [Coding Standards](#coding-standards)
- [Submitting Changes](#submitting-changes)
- [Testing](#testing)
- [Documentation](#documentation)

---

## Getting Started

### Prerequisites

- **Rust** 1.75+ with Cargo
- **Node.js** 18+ with npm/pnpm
- **Git** for version control
- **Operating System**: Linux/macOS/Windows (PTY support required)

### Clone the Repository

```bash
git clone https://github.com/your-username/liteterm-web.git
cd liteterm-web
```

### Quick Start

```bash
# Install frontend dependencies
make install

# Start development servers (both client and server)
make dev
```

This will start:
- Backend server on `http://localhost:3000`
- Frontend dev server on `http://localhost:5173` (proxied to backend)

---

## Development Environment

### Backend Development (Rust)

#### Install Rust

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

#### Development Commands

```bash
# Run server in development mode (with hot reload for code changes)
cd server
cargo run

# Run with debug logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Check code formatting
cargo fmt --all

# Apply code formatting
cargo fmt --all -- --emit=files

# Lint with clippy
cargo clippy --all-targets --all-features -- -D warnings

# Build release version
cargo build --release
```

#### Backend Architecture

```
server/src/
├── main.rs              # Entry point, server initialization
├── config.rs            # Configuration management (TOML)
├── error.rs             # Error types and handling
├── state.rs             # Application state
├── pty.rs               # PTY (pseudo-terminal) management
├── session.rs           # Session management
├── fs.rs                # File system operations
├── http/                # HTTP route handlers
│   ├── mod.rs
│   ├── login.rs         # Authentication endpoints
│   └── fs.rs            # File system endpoints
└── ws/                  # WebSocket handlers
    ├── mod.rs
    ├── terminal.rs      # Terminal WebSocket
    └── system.rs        # System events WebSocket
```

---

### Frontend Development (Svelte)

#### Install Dependencies

```bash
cd client
npm install
# or
pnpm install
```

#### Development Commands

```bash
# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview

# Type checking
npm run check

# Linting
npm run lint
```

#### Frontend Architecture

```
client/src/
├── App.svelte           # Main application shell
├── main.ts              # Entry point (mounts App.svelte)
├── app.css              # Global styles
├── components/          # Svelte components
│   ├── layout/          # Window management system
│   │   ├── LayoutRenderer.svelte  # Recursive layout renderer
│   │   ├── LayoutsModal.svelte    # Layout templates modal
│   │   └── Pane.svelte            # Pane container component
│   ├── panes/           # Functional pane types
│   │   └── TerminalPane.svelte    # Terminal component
│   └── settings/        # Settings UI
│       └── SettingsModal.svelte
├── stores/              # Svelte stores (state management)
│   ├── layoutStore.ts   # Layout state
│   └── settingsStore.ts # Settings state
└── lib/                 # Utilities and types
    ├── types.ts         # TypeScript type definitions
    └── templates.ts     # Layout templates
```

---

## Project Structure

### Root Directory

```
liteterm-web/
├── client/              # Frontend (Svelte + Vite)
├── server/              # Backend (Rust + Axum)
├── docs/                # Documentation
│   ├── design_spec.md   # Design document
│   ├── API.md           # API documentation
│   ├── CONTRIBUTING.md  # This file
│   ├── DEPLOYMENT.md    # Deployment guide
│   └── TROUBLESHOOTING.md # Troubleshooting
├── Makefile             # Build automation
└── README.md            # Project overview
```

### Build System (Makefile)

```bash
make help                # Show available commands
make install             # Install frontend dependencies
make dev                 # Start both client and server
make server              # Start backend only
make client              # Start frontend only
make build               # Build release versions
make clean               # Clean build artifacts
```

---

## Coding Standards

### Rust Backend

#### Code Style

We use `rustfmt` for code formatting. Configuration is in `rustfmt.toml`:

```toml
# Basic formatting
edition = "2021"
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"

# Line width
width_heuristic = "rfour"
print_width = 100
```

#### Formatting Commands

```bash
# Format all code
cargo fmt --all

# Check formatting without applying
cargo fmt --all -- --check

# Format and update imports
cargo fmt --all
cargo clippy --fix
```

#### Linting (Clippy)

We use Clippy for static analysis. Run before submitting PR:

```bash
# Run all lints
cargo clippy --all-targets --all-features -- -D warnings

# Fix automatically fixable lints
cargo clippy --all-targets --all-features --fix
```

#### Error Handling

Use custom error types in `error.rs`:

```rust
// Define error type
#[derive(thiserror::Error, Debug)]
pub enum LiteTermError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Authentication failed")]
    AuthFailed,
}

// Result type alias
pub type Result<T> = std::result::Result<T, LiteTermError>;
```

#### Async/Await Best Practices

```rust
use tokio::time::{sleep, Duration};

// Use spawn for concurrent operations
async fn handle_multiple_connections() -> Result<()> {
    let (conn1, conn2) = tokio::join!(
        handle_connection("conn1"),
        handle_connection("conn2")
    );
    Ok(())
}

// Use proper error propagation
async fn risky_operation() -> Result<String> {
    let result = some_async_call().await
        .map_err(LiteTermError::from)?;
    Ok(result)
}
```

---

### TypeScript/Svelte Frontend

#### Code Style

We use ESLint + Prettier for code formatting.

**Configuration Files:**
- `.eslintrc.cjs` - ESLint rules
- `.prettierrc` - Prettier configuration
- `svelte.config.js` - Svelte preprocessor config

#### TypeScript Guidelines

```typescript
// Use strict TypeScript types
interface PaneConfig {
  id: string;
  type: 'terminal' | 'preview' | 'data';
  title?: string;
  path?: string;
  active: boolean;
}

// Prefer const over let
const PANE_TYPES = ['terminal', 'preview', 'data'] as const;

// Use JSDoc for complex functions
/**
 * Calculate layout size based on split direction
 * @param direction - Split direction (horizontal/vertical)
 * @param containerSize - Total container size in pixels
 * @param ratio - Split ratio (0.0 - 1.0)
 * @returns Size of first pane in pixels
 */
function calculatePaneSize(
  direction: 'horizontal' | 'vertical',
  containerSize: number,
  ratio: number
): number {
  return containerSize * ratio;
}
```

#### Svelte Component Structure

```svelte
<script lang="ts">
  // Imports at top
  import { onMount, onDestroy } from 'svelte';
  import type { PaneConfig } from '$lib/types';

  // Props
  export let pane: PaneConfig;
  export let isActive: boolean = false;

  // Reactive statements
  $: title = pane.title || pane.path || 'Untitled';

  // Lifecycle
  onMount(() => {
    console.log('Pane mounted');
  });

  onDestroy(() => {
    console.log('Pane destroyed');
  });

  // Methods
  function handleClick() {
    // Handle click events
  }
</script>

<!-- Template -->
<div class="pane" class:active={isActive}>
  <h3>{title}</h3>
  <slot />
</div>

<!-- Styles -->
<style>
  .pane {
    padding: 1rem;
  }

  .pane.active {
    border: 2px solid var(--color-primary);
  }
</style>
```

#### Store Patterns

```typescript
// Create writable store
import { writable, derived } from 'svelte/store';

export const panes = writable<PaneConfig[]>([]);

// Derived store
export const activePane = derived(
  panes,
  $panes => $panes.find(p => p.active)
);

// Custom store with methods
export function createPaneStore() {
  const { subscribe, set, update } = writable<PaneConfig[]>([]);

  return {
    subscribe,
    addPane: (pane: PaneConfig) => update(panes => [...panes, pane]),
    removePane: (id: string) => update(panes => panes.filter(p => p.id !== id)),
    updatePane: (id: string, changes: Partial<PaneConfig>) =>
      update(panes => panes.map(p => p.id === id ? { ...p, ...changes } : p))
  };
}

export const paneStore = createPaneStore();
```

---

## Submitting Changes

### Pull Request Process

1. **Fork the repository** on GitHub

2. **Create a feature branch**

   ```bash
   git checkout -b feature/your-feature-name
   # or
   git checkout -b fix/your-bug-fix
   ```

3. **Make your changes**
   - Follow coding standards
   - Add tests for new functionality
   - Update documentation

4. **Run checks locally**

   ```bash
   # Rust backend
   cd server
   cargo fmt --all -- --check
   cargo clippy --all-targets --all-features
   cargo test

   # TypeScript frontend
   cd client
   npm run lint
   npm run check
   npm run build
   ```

5. **Commit changes**

   ```bash
   git add .
   git commit -m "feat: add new terminal feature

   - Add terminal resize handling
   - Update WebSocket protocol
   - Add tests for resize functionality"
   ```

6. **Push to your fork**

   ```bash
   git push origin feature/your-feature-name
   ```

7. **Create Pull Request** on GitHub

### Commit Message Format

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only changes
- `style:` - Changes that don't affect meaning (formatting, etc.)
- `refactor:` - Code change that neither fixes bug nor adds feature
- `test:` - Adding missing tests or correcting existing tests
- `chore:` - Changes to build process or auxiliary tools

**Examples:**

```bash
feat(terminal): add terminal resize support

Implement window resize notifications for terminal panes.
When terminal window size changes, send resize event to PTY.

Closes #123

fix(websocket): prevent connection timeout on idle

Set appropriate keep-alive intervals for WebSocket connections.

docs(api): update API documentation for v2

Add new endpoints to API reference
Update WebSocket protocol examples
```

---

## Testing

### Backend Testing (Rust)

#### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pty_creation() {
        let pty = Pty::new().expect("Failed to create PTY");
        assert!(pty.is_some());
    }

    #[test]
    fn test_path_validation() {
        let valid_paths = vec![
            "/home/user",
            "/var/log",
            "/tmp/test"
        ];

        for path in valid_paths {
            assert!(is_valid_path(path), "Path should be valid: {}", path);
        }
    }
}
```

#### Integration Tests

```rust
#[tokio::test]
async fn test_file_list_api() {
    // Setup test server
    let app = create_test_app().await;
    let client = reqwest::Client::new();

    // Make request
    let response = client
        .get("http://localhost:3000/api/fs/list?path=/tmp")
        .send()
        .await
        .expect("Request failed");

    assert_eq!(response.status(), 200);

    let data: Value = response.json().await.unwrap();
    assert!(data["success"].as_bool().unwrap());
}
```

#### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_pty_creation

# Run with output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration
```

---

### Frontend Testing (Svelte)

#### Unit Tests with Vitest

```typescript
// tests/layoutStore.test.ts
import { describe, it, expect } from 'vitest';
import { paneStore } from '../src/stores/layoutStore';

describe('PaneStore', () => {
  it('should add a pane', () => {
    const initialPanes = get(paneStore);
    expect(initialPanes).toHaveLength(0);

    paneStore.addPane({
      id: 'pane-1',
      type: 'terminal',
      active: true
    });

    const panes = get(paneStore);
    expect(panes).toHaveLength(1);
    expect(panes[0].id).toBe('pane-1');
  });
});
```

#### Component Testing

```typescript
// tests/TerminalPane.test.ts
import { render, screen } from '@testing-library/svelte';
import TerminalPane from '../src/components/panes/TerminalPane.svelte';

describe('TerminalPane', () => {
  it('renders terminal component', () => {
    render(TerminalPane, {
      props: { pane: { id: 'term-1', type: 'terminal' } }
    });

    expect(screen.getByRole('terminal')).toBeInTheDocument();
  });
});
```

#### Running Tests

```bash
# Install test dependencies
cd client
npm install

# Run all tests
npm run test

# Run tests in watch mode
npm run test:watch

# Generate coverage report
npm run test:coverage
```

---

## Documentation

### Documentation Standards

- **Update README.md** for user-facing changes
- **Update docs/API.md** for API changes
- **Add doc comments** to complex functions
- **Update design_spec.md** for architecture changes

### Writing Documentation

#### Code Documentation (Rust)

```rust
/// Create a new PTY (pseudo-terminal) pair.
///
/// This function spawns a new process with a pseudo-terminal attached,
/// allowing WebSocket connections to interact with shell commands.
///
/// # Arguments
///
/// * `shell` - Optional shell path. Defaults to `$SHELL` or `/bin/bash`
///
/// # Returns
///
/// Returns a `Result` containing the PTY pair on success, or an error
/// on failure.
///
/// # Examples
///
/// ```
/// let pty = create_pty(Some("/bin/zsh")).await?;
/// pty.write(b"ls -la\n")?;
/// let output = pty.read().await?;
/// ```
///
/// # Errors
///
/// Returns `LiteTermError::PtyCreationFailed` if:
/// - The shell path is invalid
/// - The system lacks PTY support
/// - Memory allocation fails
pub async fn create_pty(shell: Option<&str>) -> Result<PtyPair> {
    // Implementation
}
```

#### Code Documentation (TypeScript)

```typescript
/**
 * Splits a pane horizontally or vertically and creates two child panes.
 *
 * This function modifies the layout tree by converting a pane node into
 * a split node with two children. The original pane's content is moved
 * to the first child pane.
 *
 * @param paneId - ID of the pane to split
 * @param direction - Split direction: 'horizontal' or 'vertical'
 * @param ratio - Size ratio for the split (0.0 to 1.0), defaults to 0.5
 * @returns Promise that resolves when the layout is updated
 *
 * @example
 * // Split horizontally with default 50/50 ratio
 * await splitPane('pane-1', 'horizontal');
 *
 * @example
 * // Split vertically with 70/30 ratio
 * await splitPane('pane-2', 'vertical', 0.7);
 *
 * @throws {Error} If the pane ID doesn't exist
 * @throws {Error} If the ratio is not between 0.0 and 1.0
 */
export async function splitPane(
  paneId: string,
  direction: 'horizontal' | 'vertical',
  ratio: number = 0.5
): Promise<void> {
  // Implementation
}
```

### Building Documentation

```bash
# Generate Rust documentation
cd server
cargo doc --open

# Serve docs locally
cargo doc --no-deps --open
```

---

## Performance Guidelines

### Backend (Rust)

1. **Use appropriate async runtime**
   ```rust
   // For I/O-bound operations
   async fn handle_request() -> Result<Response> { ... }

   // For CPU-bound operations
   tokio::task::spawn_blocking(|| cpu_intensive_operation())
   ```

2. **Minimize allocations**
   ```rust
   // Bad: Multiple allocations
   let mut data = String::new();
   data.push_str(&prefix);
   data.push_str(&content);

   // Good: Single allocation
   let data = format!("{}{}", prefix, content);
   ```

3. **Use streaming for large files**
   ```rust
   let file = tokio::fs::File::open(path).await?;
   let stream = tokio::io::BufReader::new(file);
   let bytes = stream.into_bytes();
   ```

### Frontend (Svelte)

1. **Optimize reactive statements**
   ```svelte
   <!-- Bad: Runs on every render -->
   $: computedValue = expensiveOperation(data);

   <!-- Good: Use function call -->
   $: computedValue = computeValue(data);

   <!-- Or -->
   $: {
     if (shouldCompute) {
       computedValue = expensiveOperation(data);
     }
   }
   ```

2. **Use svelte:component for dynamic components**
   ```svelte
   <!-- Good -->
   <svelte:component this={componentMap[type]} {props} />
   ```

3. **Minimize DOM updates**
   ```svelte
   <!-- Good: Use key directive -->
   {#each items as item (item.id)}
     <Component {item} />
   {/each}
   ```

---

## Debugging

### Backend Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Trace all modules
RUST_LOG=trace cargo run

# Debug specific module
RUST_LOG=liteterm_web::ws::terminal=debug cargo run
```

### Frontend Debugging

```bash
# Enable debug mode
VITE_DEBUG=true npm run dev

# Analyze bundle size
npm run build
npm run analyze
```

### Common Issues

#### PTY Not Working

```bash
# Check PTY support
ls -la /dev/ptmx
# Should output: crw-rw-rw- 1 root root 5, 2 ...
```

#### WebSocket Connection Fails

1. Check firewall settings
2. Verify session authentication
3. Check browser console for errors
4. Enable debug logging on server

#### File Permissions

```bash
# Ensure server has read access to root directory
chmod +rx /path/to/workspace
chmod 755 /path/to/workspace/subdirs
```

---

## Questions?

If you have questions about contributing:

1. Check existing [Issues](https://github.com/your-repo/liteterm-web/issues)
2. Create a new issue with:
   - Clear description
   - Steps to reproduce (for bugs)
   - Your environment (OS, Rust version, etc.)
   - Screenshots if applicable

---

## Recognition

Contributors will be recognized in:
- README.md Contributors section
- Release notes for significant contributions
- Annual contributor highlights

Thank you for contributing to LiteTerm-Web! 🎉
