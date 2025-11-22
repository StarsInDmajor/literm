# LiteTerm-Web Server

LiteTerm-Web 的后端服务器，基于 **Rust** 和 **Axum** 构建，提供高性能的 WebSocket 服务和文件系统 API。

## 技术栈

*   **Language**: Rust (Edition 2021)
*   **Web Framework**: [Axum](https://github.com/tokio-rs/axum) - 模块化、高性能的 Web 框架。
*   **Async Runtime**: [Tokio](https://tokio.rs/) - Rust 异步运行时标准。
*   **PTY**: [portable-pty](https://github.com/wez/wezterm/tree/main/pty) - 跨平台伪终端处理。
*   **File Watcher**: [notify](https://github.com/notify-rs/notify) - 高效的文件系统监听。
*   **Serialization**: Serde + TOML/JSON。

## 快速开始

### 运行

```bash
cargo run
```

默认监听 `0.0.0.0:3000`。

### 配置

配置文件位于 `config/config.toml` (如果不存在请创建)。

```toml
[server]
bind_addr = "0.0.0.0"
port = 3000
root_dir = "/home/user"  # 文件浏览器的根目录
session_timeout_minutes = 60

[auth]
password_hash = "..." # Argon2 hash

[features]
enable_hdf5 = true
enable_watch = true
```

## API 接口

### HTTP API

| 方法 | 路径 | 描述 |
| :--- | :--- | :--- |
| POST | `/api/login` | 用户登录 |
| GET | `/api/fs/list` | 获取文件列表 (`?path=/...`) |
| GET | `/api/fs/content` | 获取文本文件内容 |
| GET | `/api/fs/raw` | 获取文件原始流 (用于图片/PDF) |

### WebSocket API

*   `/ws/term`: 终端交互 (Binary/Text 协议)
*   `/ws/system`: 系统通知与文件监听 (JSON 协议)

## 核心模块

*   `src/pty.rs`: 封装 PTY 进程管理，处理 stdin/stdout 转发。
*   `src/ws/`: WebSocket 处理器，分发终端输入和系统事件。
*   `src/fs.rs`: 文件系统操作封装。