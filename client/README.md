# LiteTerm-Web Client

LiteTerm-Web 的前端部分，基于 **Svelte 5** 和 **Vite** 构建，旨在提供高性能、低延迟的终端和窗口管理体验。

## 技术栈

*   **Framework**: [Svelte 5](https://svelte.dev/) - 编译型前端框架，无 Virtual DOM，高性能。
*   **Build Tool**: [Vite](https://vitejs.dev/) - 极速的开发服务器和构建工具。
*   **Styling**: [TailwindCSS](https://tailwindcss.com/) - 原子化 CSS 框架。
*   **Terminal**: [xterm.js](https://xtermjs.org/) - 业界标准的 Web 终端组件。
*   **Icons**: [Lucide Svelte](https://lucide.dev/) - 漂亮的矢量图标库。

## 目录结构

*   `src/components/layout/`: 核心布局系统 (LayoutRenderer, Pane)。
*   `src/components/panes/`: 具体的功能组件 (TerminalPane, PreviewPane 等)。
*   `src/stores/`: Svelte Stores (布局状态管理)。
*   `src/lib/`: 类型定义和工具函数。

## 开发指南

### 安装依赖

```bash
npm install
```

### 启动开发服务器

```bash
npm run dev
```

默认运行在 `http://localhost:5173`。需要配合后端服务器运行以获得完整功能。

### 构建生产版本

```bash
npm run build
```

构建产物将输出到 `dist/` 目录。

## 核心功能实现

*   **布局系统**: 使用递归组件 (`LayoutRenderer`) 渲染分屏树。
*   **终端连接**: 每个 `TerminalPane` 独立建立 WebSocket 连接到后端 PTY。
*   **自适应**: 监听窗口大小变化，自动调整 xterm.js 尺寸并通知后端。