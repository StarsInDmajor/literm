# LiteTerm-Web

**LiteTerm-Web** is a web-based terminal with tiling window manager (like Tmux). Optimized for mobile devices, enables split-screen workflow for coding and real-time preview.

[![Rust](https://img.shields.io/badge/Rust-1.75+-blue?logo=rust)](https://www.rust-lang.org/)
[![Svelte](https://img.shields.io/badge/Svelte-5-orange?logo=svelte)](https://svelte.dev/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

## ✨ Features

* **Tiling Window Manager**: Horizontal/vertical splits with unlimited nesting
* **Terminal**: Full-featured via WebSocket + PTY with xterm.js
* **Preview**: Real-time preview for PDF, images, and Markdown
* **Mobile Optimized**: Virtual keyboard support (Esc, Ctrl, Tab, Arrow keys)
* **High Performance**: Rust backend + Svelte frontend

## 🚀 Quick Start

### Prerequisites
- **Rust** 1.75+ (backend)
- **Node.js** 18+ (frontend)

### Development

```bash
# Using Makefile (recommended)
make install  # Install dependencies
make dev      # Start both servers

# Or manually
cd server && cargo run                    # Backend on :3000
cd client && npm install && npm run dev   # Frontend on :5173
```

### Production

```bash
make build
./server/target/release/liteterm-web
```

## 🏗️ Architecture

```
Frontend (Svelte 5) ←→ Backend (Rust + Axum) ←→ System
      │                        │                  │
   Layout System           WebSocket API     PTY + FS
   Terminal (xterm.js)      File Watcher     Shell Process
   Preview Components       Static Files     File System
```

## 📁 Project Structure

```
liteterm-web/
├── client/          # Frontend (Svelte 5 + Vite)
│   ├── src/components/
│   │   ├── layout/  # Window management
│   │   ├── panes/   # Terminal, Preview, etc.
│   │   └── settings/
│   └── package.json
├── server/          # Backend (Rust + Axum)
│   ├── src/
│   │   ├── http/    # HTTP handlers
│   │   ├── ws/      # WebSocket handlers
│   │   └── pty.rs   # PTY management
│   └── Cargo.toml
└── docs/            # Documentation
```

## 🛠️ Tech Stack

**Frontend**: Svelte 5, Vite, TailwindCSS, xterm.js
**Backend**: Rust, Axum, Tokio, portable-pty
**Communication**: HTTP (static/auth) + WebSocket (real-time)

## 📚 Documentation

- **[Architecture](./docs/design_spec.md)** - Comprehensive design
- **[API Reference](./docs/API.md)** - Detailed API docs
- **[Contributing](./docs/CONTRIBUTING.md)** - Development guide
- **[Deployment](./docs/DEPLOYMENT.md)** - Production setup

## 🎯 Usage

Split your workspace for efficient workflow:

```
┌─────────────┬─────────────┐    ┌─────────────────────────────┐
│   Terminal  │   Preview   │    │         Terminal            │
│   (Vim)     │   (PDF)     │    │      (Python/R)             │
│             │             │    ├─────────────────────────────┤
│             │             │    │    Data Viewer (HDF5)       │
└─────────────┴─────────────┘    └─────────────────────────────┘
```

## 📝 License

MIT License - see [LICENSE](LICENSE) file.