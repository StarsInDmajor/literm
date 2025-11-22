# API Documentation

This document provides detailed specifications for all LiteTerm-Web API endpoints, WebSocket protocols, and data structures.

## Table of Contents

- [HTTP API](#http-api)
  - [Authentication](#authentication)
  - [File System Operations](#file-system-operations)
- [WebSocket API](#websocket-api)
  - [Terminal WebSocket](#terminal-websocket)
  - [System WebSocket](#system-websocket)
- [Data Structures](#data-structures)
- [Error Handling](#error-handling)
- [Authentication Flow](#authentication-flow)

---

## HTTP API

### Base URL

```
Development: http://localhost:3000
Production:  http://your-server:3000
```

All API endpoints are prefixed with `/api/`.

### Authentication

#### POST `/api/login`

Authenticate user and establish session.

**Request:**

```json
{
  "password": "your-password"
}
```

**Response:**

```json
{
  "success": true,
  "message": "Authentication successful",
  "session_id": "uuid-string"
}
```

**Status Codes:**

- `200 OK` - Authentication successful
- `401 Unauthorized` - Invalid password
- `400 Bad Request` - Invalid request body

**CURL Example:**

```bash
curl -X POST http://localhost:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"password": "your-password"}'
```

---

### File System Operations

#### GET `/api/fs/list`

List directory contents.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | Absolute path to directory |

**Response:**

```json
{
  "success": true,
  "path": "/path/to/directory",
  "entries": [
    {
      "name": "file.txt",
      "path": "/path/to/directory/file.txt",
      "is_dir": false,
      "size": 1024,
      "mtime": 1703123456
    },
    {
      "name": "subdir",
      "path": "/path/to/directory/subdir",
      "is_dir": true,
      "size": 4096,
      "mtime": 1703123000
    }
  ]
}
```

**Status Codes:**

- `200 OK` - Success
- `400 Bad Request` - Invalid path
- `401 Unauthorized` - Not authenticated
- `403 Forbidden` - Access denied to path
- `404 Not Found` - Path does not exist

**CURL Example:**

```bash
curl "http://localhost:3000/api/fs/list?path=/home/user/projects"
```

---

#### GET `/api/fs/content`

Get text file contents.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | Absolute path to text file |

**Response:**

```
Raw text content of file
```

**Headers:**

```
Content-Type: text/plain; charset=utf-8
```

**Status Codes:**

- `200 OK` - Success
- `400 Bad Request` - Invalid path
- `401 Unauthorized` - Not authenticated
- `403 Forbidden` - Access denied
- `404 Not Found` - File does not exist
- `413 Payload Too Large` - File too large to display

**CURL Example:**

```bash
curl "http://localhost:3000/api/fs/content?path=/home/user/README.md"
```

---

#### GET `/api/fs/raw`

Get binary file stream (for images, PDFs, etc.).

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `path` | string | Yes | Absolute path to file |

**Response:**

Binary file content with appropriate `Content-Type` header.

**Headers:**

```
Content-Type: application/pdf          # for PDF files
Content-Type: image/png                # for PNG files
Content-Type: image/jpeg               # for JPG files
Content-Type: image/svg+xml            # for SVG files
Content-Type: application/octet-stream # for unknown types
```

**Status Codes:**

- `200 OK` - Success
- `400 Bad Request` - Invalid path
- `401 Unauthorized` - Not authenticated
- `403 Forbidden` - Access denied
- `404 Not Found` - File does not exist

**CURL Example:**

```bash
# Save PDF to file
curl "http://localhost:3000/api/fs/raw?path=/home/user/document.pdf" \
  -o document.pdf

# View image in browser
# http://localhost:3000/api/fs/raw?path=/home/user/image.png
```

---

## WebSocket API

WebSocket connections provide real-time communication for terminal I/O and system events.

### Connection URL

```
ws://localhost:3000/ws/{endpoint}
wss://your-server:3000/ws/{endpoint}  # for HTTPS
```

### Authentication

WebSocket connections require session authentication. Include session cookie or pass `session_id` in query string:

```
ws://localhost:3000/ws/term?session_id=uuid-string
```

---

### Terminal WebSocket

**Endpoint:** `/ws/term`

**Purpose:** Bidirectional communication for terminal I/O

**Protocol:** Mixed (Binary + Text)

- **Binary data**: Terminal input/output (stdin/stdout)
- **Text messages**: Control commands

#### Connection Flow

1. Client establishes WebSocket connection
2. Server creates PTY (pseudo-terminal) process
3. Server begins forwarding PTY output to client
4. Client sends input data to server
5. Server forwards input to PTY stdin

#### Client → Server (Input)

Send binary terminal input:

```
┌─────────────┐
│ Binary Data │  ← Raw terminal bytes (keys, commands)
└─────────────┘
```

**Example (JavaScript):**

```javascript
const ws = new WebSocket('ws://localhost:3000/ws/term?session_id=xxx');

// Send keyboard input
ws.send('ls -la\n');

// Send special keys (ANSI escape sequences)
ws.send('\x1b[A'); // Up arrow
ws.send('\x1b[B'); // Down arrow
ws.send('\x03');    // Ctrl+C
```

#### Server → Client (Output)

Receive binary terminal output:

```
┌─────────────┐
│ Binary Data │  ← PTY stdout/stderr
└─────────────┘
```

**Example (JavaScript):**

```javascript
ws.onmessage = (event) => {
  if (event.data instanceof Blob) {
    // Binary terminal output
    event.data.arrayBuffer().then(buffer => {
      const bytes = new Uint8Array(buffer);
      // Display in xterm.js
      terminal.write(bytes);
    });
  }
};
```

#### Terminal Resize

**Client → Server (Text Message):**

Send window resize notification:

```json
{
  "type": "resize",
  "cols": 80,
  "rows": 24
}
```

**Example:**

```javascript
ws.send(JSON.stringify({
  type: 'resize',
  cols: 120,
  rows: 40
}));
```

#### Terminal Close

**Client → Server (Text Message):**

Close terminal session:

```json
{
  "type": "close"
}
```

**Example:**

```javascript
ws.send(JSON.stringify({ type: 'close' }));
ws.close();
```

---

### System WebSocket

**Endpoint:** `/ws/system`

**Purpose:** System notifications and file watcher events

**Protocol:** JSON

#### Message Types

##### File Changed Event

Server → Client notification when watched file is modified:

```json
{
  "type": "file_changed",
  "path": "/home/user/document.pdf",
  "mtime": 1703123456,
  "event": "modify"
}
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Always "file_changed" |
| `path` | string | Absolute path to changed file |
| `mtime` | number | New modification timestamp |
| `event` | string | Event type: "create", "modify", "delete", "rename" |

**Example:**

```javascript
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);

  if (data.type === 'file_changed') {
    console.log(`File changed: ${data.path}`);
    // Auto-refresh preview component
    refreshPreview(data.path);
  }
};
```

##### Error Event

Server → Client error notification:

```json
{
  "type": "error",
  "message": "Terminal process exited",
  "code": 1
}
```

**Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | Always "error" |
| `message` | string | Error description |
| `code` | number | Optional error code |

##### Pong Response

**Client → Server:**

```json
{
  "type": "ping"
}
```

**Server → Client:**

```json
{
  "type": "pong",
  "timestamp": 1703123456
}
```

---

## Data Structures

### Layout Structure

Layout tree for window management:

```typescript
interface LayoutNode {
  id: string;
  type: 'split' | 'pane';
  direction?: 'horizontal' | 'vertical';
  size?: number;  // 0.0 - 1.0, proportional size
  children?: [LayoutNode, LayoutNode];  // for splits
  pane?: PaneConfig;
}

interface PaneConfig {
  id: string;
  type: 'terminal' | 'preview' | 'data' | 'file';
  title?: string;
  path?: string;  // File/folder path
  active?: boolean;
}
```

### File Entry

```typescript
interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  mtime: number;
}
```

### Session

```typescript
interface Session {
  id: string;
  created_at: number;
  last_activity: number;
  authenticated: boolean;
  pty_processes: Map<string, PtyHandle>;
}
```

---

## Error Handling

### HTTP Error Responses

All HTTP endpoints return consistent error format:

```json
{
  "success": false,
  "error": "ERROR_CODE",
  "message": "Human-readable error message"
}
```

**Error Codes:**

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Authentication required or failed |
| `FORBIDDEN` | 403 | Access denied to resource |
| `NOT_FOUND` | 404 | Resource not found |
| `INVALID_PATH` | 400 | Invalid file path |
| `TOO_LARGE` | 413 | File too large |
| `INTERNAL_ERROR` | 500 | Server internal error |

### WebSocket Error Handling

WebSocket connections can encounter:

1. **Connection Error** - Failed to establish connection
2. **Protocol Error** - Invalid message format
3. **Authentication Error** - Invalid or expired session
4. **Process Error** - PTY process terminated

**Error Event Example:**

```javascript
ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = (event) => {
  if (event.code !== 1000) {
    console.error('Connection closed with code:', event.code);
  }
};
```

---

## Authentication Flow

### Session-Based Authentication

1. User sends password to `/api/login`
2. Server validates password against hash
3. Server creates session (stored in memory or Redis)
4. Server returns success response
5. Session cookie is set in response
6. All subsequent API calls include session cookie

### Session Cookie

```
Cookie: session_id=uuid-string
```

### Session Timeout

Default timeout: 60 minutes (configurable)

Sessions are automatically invalidated after `session_timeout_minutes` of inactivity.

### Session Management

```bash
# List active sessions (debug endpoint - development only)
curl -H "Cookie: session_id=admin-session" \
  http://localhost:3000/api/sessions
```

---

## Rate Limiting

API endpoints are subject to rate limiting:

- **Authentication**: 5 requests per minute per IP
- **File operations**: 100 requests per minute per session
- **WebSocket**: No limit (subject to connection limits)

Rate limit headers:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 99
X-RateLimit-Reset: 1703123456
```

---

## Examples

### Complete Workflow

```javascript
// 1. Authenticate
const loginResponse = await fetch('http://localhost:3000/api/login', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ password: 'your-password' }),
  credentials: 'include'  // Include cookies
});

const { session_id } = await loginResponse.json();

// 2. List directory
const listResponse = await fetch(
  'http://localhost:3000/api/fs/list?path=/home/user',
  { credentials: 'include' }
);
const { entries } = await listResponse.json();

// 3. Open terminal
const termWs = new WebSocket(
  `ws://localhost:3000/ws/term?session_id=${session_id}`
);

// 4. Watch file changes
const sysWs = new WebSocket(
  `ws://localhost:3000/ws/system?session_id=${session_id}`
);

sysWs.onmessage = (event) => {
  const data = JSON.parse(event.data);
  if (data.type === 'file_changed') {
    // Auto-refresh preview
    refreshPreview(data.path);
  }
};
```

### Using with xterm.js

```javascript
import { Terminal } from 'xterm';
import 'xterm/css/xterm.css';

// Initialize terminal
const term = new Terminal({
  cursorBlink: true,
  fontSize: 14,
  convertEol: true
});

term.open(document.getElementById('terminal'));

// Connect to WebSocket
const ws = new WebSocket('ws://localhost:3000/ws/term?session_id=xxx');

// Send data from terminal to WebSocket
term.onData(data => {
  ws.send(data);
});

// Receive data from WebSocket and write to terminal
ws.onmessage = (event) => {
  if (event.data instanceof Blob) {
    event.data.arrayBuffer().then(buffer => {
      term.write(new Uint8Array(buffer));
    });
  }
};

// Handle terminal resize
term.onResize(({ cols, rows }) => {
  ws.send(JSON.stringify({ type: 'resize', cols, rows }));
});
```

---

## SDK / Client Libraries

### JavaScript/TypeScript

```typescript
// Official TypeScript client (hypothetical)
import { LiteTermClient } from '@liteterm/client';

const client = new LiteTermClient('http://localhost:3000');

// Authenticate
await client.login('your-password');

// List files
const files = await client.fs.list('/home/user');

// Open terminal
const terminal = await client.terminal.create();
terminal.onOutput((data) => {
  console.log('Terminal output:', data);
});
terminal.write('ls\n');
```

---

## Versioning

**Current API Version:** v1

API versioning is applied via URL prefix: `/api/v1/`

Future versions will increment the version number, e.g., `/api/v2/`.

---

## Changelog

### v1.0.0 (Initial Release)

- Initial API implementation
- HTTP endpoints for authentication and file operations
- WebSocket endpoints for terminal I/O and system events
- Session-based authentication
- File watcher notifications
