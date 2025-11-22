这是一份为您量身定制的**LiteTerm-Web 项目全案文档**。您可以直接使用这份文档作为后续开发的蓝图，或者将其喂给 AI 编程助手（如 Cursor, Github Copilot）来分步生成代码。

---

# 第一部分：项目需求文档 (PRD)

## 1.1 项目概述
**项目名称：** LiteTerm-Web
**项目愿景：** 构建一个基于 Web 的、高性能的个人远程工作台。
**核心价值：** 结合了终端模拟器与平铺式窗口管理器（Tiling Window Manager）的特点，让用户在移动设备（平板/手机）上也能获得类似 Tmux 的高效分屏体验，实现“一边写代码/编译，一边实时预览结果”的闭环工作流。

## 1.2 用户角色与核心故事
*   **用户：** 熟悉 Linux 命令行、有移动办公需求的开发者或科研人员。
*   **核心故事：**
    *   作为用户，我希望通过浏览器访问服务器 IP，输入密码后直接进入工作台。
    *   作为用户，我希望将屏幕切分为左右两部分，左边运行 Vim 编辑 LaTeX，右边显示 PDF 预览。
    *   作为用户，当我在终端执行编译命令后，右边的 PDF 窗口能自动刷新，无需手动重载。
    *   作为用户，我在手机上操作时，需要一排虚拟功能键（Esc, Ctrl, Tab）来辅助输入。
    *   作为用户，我希望查看服务器上的 HDF5 数据文件，通过专门的查看器窗口进行分析。

## 1.3 功能需求详述

### 1.3.1 窗口管理系统 (Tmux-like Core)
| 功能点 | 详细描述 |
| :--- | :--- |
| **自由分屏** | 用户可以将当前窗口（Pane）水平或垂直切分。支持无限嵌套（递归切分）。 |
| **内容指派** | 每个切分出的 Pane 可以独立选择加载的组件类型（终端、PDF viewer、文件树等）。 |
| **布局调整** | 支持拖拽调整分屏比例。支持最大化某个 Pane，再次点击恢复原状。 |
| **工作区预设** | 提供默认布局模板（如：Default [全终端]、Writing [左终端+右预览]、Data [上终端+下数据]）。 |

### 1.3.2 组件类型与行为
该系统是一个容器，支持加载以下类型的组件：

1.  **终端组件 (Terminal Pane)**
    *   **核心：** 基于 WebSocket 连接后端 PTY (伪终端)。
    *   **移动端辅助：** 底部或键盘上方常驻“虚拟按键栏” (Esc, Tab, Ctrl, Alt, Arrows, PgUp/Dn)。
    *   **字体：** 混合字体策略（优先本地字体 + 远程 Nerd Font 符号）。
    *   **交互：** 支持鼠标点击、滚轮滚动、剪贴板粘贴（浏览器原生）。

2.  **增强预览组件 (Preview Pane)**
    *   **支持格式：** PDF, 图片 (PNG/JPG/SVG), Markdown (渲染为HTML)。
    *   **文件监听 (Watch Mode)：** 当显示的文件的 `mtime` 发生变化（由后端推送通知），组件自动重新加载内容。
    *   **操作：** 支持手势缩放 (Pinch-to-Zoom)、平移。

3.  **数据查看组件 (Data Viewer Pane)**
    *   **HDFView：** 支持加载 HDF5 文件，以树状结构查看 Dataset/Group，并展示简单的表格或图表数据。

4.  **文件管理组件 (File Explorer Pane)**
    *   以树状图展示服务器文件，双击文件可在新 Pane 中打开（或替换当前 Pane 内容）。

### 1.3.3 系统与非功能需求
*   **认证：** HTTP Basic Auth 或简单的表单密码验证（Session/Cookie 维持）。
*   **网络：** 纯 HTTP + WebSocket。不强制 HTTPS。
*   **性能：** 极低内存占用，除了加载查看大文件外，尽量减少带宽消耗。

---

# 第二部分：技术架构与设计文档

## 2.1 技术栈选型

| 领域 | 选型 | 理由 |
| :--- | :--- | :--- |
| **后端语言** | **Rust** | 内存安全，单文件部署，极高的并发性能（适合 WebSocket 和文件监听）。 |
| **后端框架** | **Axum** | 极其轻量、模块化，对 WebSocket 支持极好，基于 Tokio 异步运行时。 |
| **终端库** | **portable-pty** | Rust 跨平台 PTY 处理库，稳定可靠。 |
| **文件监听** | **notify** | Rust 库，用于高效监听文件系统变动事件。 |
| **前端框架** | **Svelte (v4/v5)** | 编译型框架，无 Virtual DOM，生成的 JS 包极小，运行极快，适合移动端。 |
| **UI 样式** | **TailwindCSS** | 原子化 CSS，快速构建响应式布局和暗黑模式。 |
| **核心组件** | **xterm.js** (终端), **pdf.js** (PDF渲染), **h5wasm** (HDF5读取) | 业界标准的前端组件库。 |

## 2.2 系统架构图

```mermaid
graph TD
    User[用户设备 (Browser)]
    
    subgraph Frontend [Svelte Client]
        WM[窗口管理器 (Layout System)]
        C_Term[终端组件 (xterm.js)]
        C_Prev[预览组件 (PDF/Img)]
        C_Data[数据组件 (HDF/File)]
        WS_Client[WS Client]
        
        WM --> C_Term
        WM --> C_Prev
        WM --> C_Data
        C_Term <--> WS_Client
        C_Prev <--> WS_Client
    end
    
    subgraph Backend [Rust Server (Axum)]
        Auth[身份认证]
        Router[HTTP Router]
        WS_Handler[WebSocket Handler]
        PTY_Mgr[PTY Manager]
        FS_Watcher[File Watcher (notify)]
        Static[静态文件服务]
        
        Router --> Auth
        Auth --> WS_Handler
        Auth --> Static
        WS_Handler <--> PTY_Mgr
        WS_Handler <--> FS_Watcher
    end
    
    subgraph System [Ubuntu OS]
        Shell[Bash/Zsh Process]
        FileSystem[文件系统]
    end
    
    User <-->|HTTP (Assets/Auth)| Router
    User <-->|WebSocket (Data)| WS_Handler
    
    PTY_Mgr <-->|Stdin/Stdout| Shell
    FS_Watcher -.->|Watch Events| FileSystem
    Static --> FileSystem
```

## 2.3 核心数据结构设计

### 2.3.1 前端：递归布局树 (Layout Tree)
为了实现 Tmux 风格的分屏，前端必须维护一个树状数据结构。

```typescript
// 定义节点类型
type NodeType = 'container' | 'pane';

// 定义分割方向
type Direction = 'horizontal' | 'vertical';

// 定义窗格内容的类型
type ContentType = 'terminal' | 'file-manager' | 'pdf-viewer' | 'image-viewer' | 'hdf-viewer' | 'empty';

interface LayoutNode {
  id: string;
  type: NodeType;
  
  // 仅当 type === 'container' 时有效
  direction?: Direction;
  children?: LayoutNode[]; 
  splitRatio?: number[]; // 记录子节点的比例，如 [0.5, 0.5]
  
  // 仅当 type === 'pane' 时有效
  contentType?: ContentType;
  contentConfig?: {
    filePath?: string; // 如果是预览器，记录文件路径
    terminalId?: string; // 如果是终端，记录会话ID
  };
}
```

### 2.3.2 后端：WebSocket 消息协议
为了保持连接复用，所有控制信令和数据传输共用一个 WS 连接（或者每个 Pane 一个连接，**推荐每个 Terminal Pane 一个独立连接**以简化逻辑，而文件监听使用全局共享连接）。

**Terminal WS Protocol (Binary/Text):**
1.  **Input (Client -> Server):** `[type: byte, ...payload]`
    *   `0x01` + `data`: 终端输入 (Stdin)
    *   `0x02` + `rows(u16), cols(u16)`: 调整窗口大小 (Resize)
2.  **Output (Server -> Client):**
    *   直接发送二进制流 (Stdout)

**System/Watcher WS Protocol (JSON):**
1.  **Client -> Server:**
    *   `{ "action": "watch", "path": "/home/user/project/output.pdf" }`
2.  **Server -> Client:**
    *   `{ "event": "change", "path": "/home/user/project/output.pdf", "timestamp": 123456789 }`

## 2.4 数据库/存储设计
由于是轻量级工具，**不使用传统数据库**。
*   **用户配置 (Layout, Settings):** 存储在浏览器 `localStorage` 中。
*   **服务器配置:** 简单的 `config.toml` 文件或环境变量（存储端口、密码 hash、根目录）。

## 2.5 API 接口定义 (HTTP)

| 方法 | URL | 描述 | 参数/Body |
| :--- | :--- | :--- | :--- |
| POST | `/api/login` | 用户登录，获取 Token/Cookie | `{password: "xxx"}` |
| GET | `/api/fs/list` | 获取文件列表 | `?path=/home/user` |
| GET | `/api/fs/content` | 获取文件内容（小文件/代码） | `?path=...` |
| GET | `/api/fs/raw` | 获取文件原始流（用于 img/pdf src） | `?path=...` |
| WS | `/ws/term` | 建立终端 WebSocket 连接 | - |
| WS | `/ws/system` | 建立系统通知 WebSocket 连接 | - |

## 2.6 字体与样式规范
*   **Font Family:**
    ```css
    font-family: "JetBrainsMono Nerd Font", "Apple Color Emoji", "Segoe UI Emoji", "Noto Color Emoji", monospace;
    ```
    *注：需要在 CSS 中通过 `@font-face` 引入裁剪过的、仅包含 Icon 的 Nerd Font 字体文件。*
*   **Theme:** 默认 Dark Mode，背景色建议使用 `#1e1e1e` 或 `#000000` 以适配 OLED 屏幕省电。

## 2.7 开发路线图 (Roadmap)

1.  **Phase 1: Core Terminal (MVP)**
    *   搭建 Rust Axum 服务器。
    *   实现 PTY + WebSocket + xterm.js。
    *   实现移动端虚拟键盘栏。
2.  **Phase 2: Layout System**
    *   实现 Svelte 的递归布局组件。
    *   实现分屏逻辑（Split H/V, Close）。
3.  **Phase 3: File System & Preview**
    *   实现文件列表 API。
    *   集成 PDF.js 和图片查看器。
    *   实现后端 `notify` 文件监听与前端自动刷新。
4.  **Phase 4: Advanced Features**
    *   集成 HDF5 查看器。
    *   完善手势操作和 UI 美化。

---

**您可以将本文档的特定章节复制给 AI，例如：**
*   *"请基于 Rust 和 Axum 为我编写后端代码，按照文档 2.3.2 中的 WebSocket 协议实现 PTY 转发..."*
*   *"请使用 Svelte 编写前端组件，数据结构参考 2.3.1，实现递归分屏布局..."*