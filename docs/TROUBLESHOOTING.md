# Troubleshooting Guide

This guide helps you diagnose and solve common issues with LiteTerm-Web.

## Table of Contents

- [Common Issues](#common-issues)
  - [Server Won't Start](#server-wont-start)
  - [WebSocket Connection Failed](#websocket-connection-failed)
  - [Terminal Not Working](#terminal-not-working)
  - [Authentication Fails](#authentication-fails)
  - [File Not Loading](#file-not-loading)
  - [Preview Not Refreshing](#preview-not-refreshing)
- [Development Issues](#development-issues)
- [Performance Issues](#performance-issues)
- [Platform-Specific Issues](#platform-specific-issues)
- [Debugging](#debugging)
- [Getting Help](#getting-help)

---

## Common Issues

### Server Won't Start

#### Symptom: Server fails to start or crashes immediately

**Check 1: Port Already in Use**

```bash
# Check if port 3000 is in use
sudo lsof -i :3000
# or
netstat -tulnp | grep 3000

# Kill process using port
sudo kill -9 <PID>
```

**Check 2: Invalid Configuration**

```bash
# Validate TOML syntax
toml-verify server/config/config.toml

# Or run server with verbose output
RUST_LOG=debug ./server/target/release/liteterm-web --config server/config/config.toml
```

Common config errors:
```toml
# ❌ Invalid: Missing quotes
password_hash = $argon2id$...

# ✅ Correct:
password_hash = "$argon2id$..."

# ❌ Invalid: Invalid path
root_dir = ~/workspace

# ✅ Correct:
root_dir = "/home/user/workspace"
```

**Check 3: Permission Denied**

```bash
# Check if user has read access to root_dir
sudo -u liteterm ls /path/to/root_dir

# Check log file permissions
sudo touch /var/log/liteterm.log
sudo chown liteterm:liteterm /var/log/liteterm.log
```

**Check 4: Missing Dependencies**

```bash
# Check if PTY is available
ls -la /dev/ptmx
# Should output: crw-rw-rw- 1 root root 5, 2 ...

# If missing:
sudo mknod /dev/ptmx c 5 2
sudo chmod 666 /dev/ptmx
```

---

### WebSocket Connection Failed

#### Symptom: "WebSocket connection failed" error in browser

**Check 1: Session Authentication**

```bash
# Inspect browser DevTools -> Network tab
# Check if login request returns success

# Manually test with curl
curl -X POST http://localhost:3000/api/login \
  -H "Content-Type: application/json" \
  -d '{"password": "your-password"}'

# Should return: {"success": true, ...}
```

**Check 2: WebSocket URL**

Browser console error:
```
WebSocket connection to 'ws://localhost:3000/ws/term?session_id=xxx' failed
```

Verify:
- Session ID is present in URL
- Using `ws://` for HTTP or `wss://` for HTTPS
- Firewall allows WebSocket connections

**Check 3: Backend WebSocket Handler**

```bash
# Check if WebSocket endpoint is accessible
curl -i -N -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Key: SGVsbG8gV29ybGQ=" \
  -H "Sec-WebSocket-Version: 13" \
  http://localhost:3000/ws/term

# Should return: 101 Switching Protocols
```

**Check 4: Browser Network Tab**

Open DevTools (F12) -> Network tab:
- Filter by "WS" (WebSocket)
- Click on WebSocket connection
- Check "Frames" tab for messages
- Look for errors in "Messages" section

---

### Terminal Not Working

#### Symptom: Terminal pane shows blank screen or doesn't respond

**Check 1: PTY Process**

```bash
# Check if PTY process exists
ps aux | grep -E "bash|zsh|sh"

# Check server logs
sudo journalctl -u liteterm -f
```

**Check 2: WebSocket Binary Transfer**

```javascript
// In browser console
const ws = new WebSocket('ws://localhost:3000/ws/term?session_id=xxx');
ws.onmessage = (event) => {
  console.log('Received:', event.data);
};
ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

// Try sending test data
ws.onopen = () => {
  ws.send('echo "test"\n');
};
```

Expected output in console:
```
Received: Blob { size: ..., type: "application/octet-stream" }
```

**Check 3: Terminal Font/Styling**

```css
/* xterm.js font issues */
.xterm {
  font-family: "Fira Code", "JetBrains Mono", monospace !important;
  font-size: 14px;
}
```

**Check 4: Terminal Size**

```javascript
// In browser console
term.resize(80, 24); // Try different sizes
```

If terminal works after resize, the issue is initial size calculation.

**Check 5: Terminal Permissions**

```bash
# Check shell permissions
ls -la $(which bash)
ls -la $(which zsh)

# Ensure shell is executable
chmod +x /bin/bash
chmod +x /bin/zsh
```

---

### Authentication Fails

#### Symptom: "Invalid password" despite correct password

**Check 1: Password Hash**

```bash
# Verify hash format (should start with $argon2id$)
grep password_hash server/config/config.toml

# Test password with argon2-cli
echo "your-password" | argon2 $(openssl rand -base64 32) -e -l 16 -k 65536 -p 4 -t 3
# Compare with config hash
```

**Check 2: Session Cookie**

Browser DevTools -> Application -> Cookies:
- Check if `session_id` cookie is set after login
- Verify cookie has correct domain/path
- Check if cookie is marked as "Secure" for HTTPS

```bash
# Check session storage in browser console
localStorage.getItem('session_id')
// or
sessionStorage.getItem('session_id')
```

**Check 3: Server Session Management**

```bash
# Check server logs for session creation
grep "Session created" /var/log/liteterm.log

# Check for session timeout
grep "Session expired" /var/log/liteterm.log
```

**Check 4: Authentication Middleware**

```rust
// Debug: Enable auth logging in server/src/http/login.rs
debug!("Login attempt from IP: {}", ip);
debug!("Session created: {}", session_id);
```

---

### File Not Loading

#### Symptom: "File not found" or "Permission denied" errors

**Check 1: Path Validation**

```bash
# Test API endpoint manually
curl "http://localhost:3000/api/fs/list?path=/home/user"

# Check response
{
  "success": true,
  "path": "/home/user",
  "entries": [...]
}

# If error:
{
  "success": false,
  "error": "INVALID_PATH",
  "message": "Invalid path"
}
```

**Check 2: File Permissions**

```bash
# Check file permissions
ls -la /path/to/file

# Check directory permissions
ls -lad /path/to/directory

# Server must have read access
sudo -u liteterm test -r /path/to/file && echo "Readable" || echo "Not readable"
```

**Check 3: Root Directory Configuration**

```toml
[server]
root_dir = "/home/user"
# File path: /home/user/projects/file.txt
# API path: /api/fs/list?path=/projects/file.txt
# Note: root_dir is stripped from API paths
```

**Check 4: Symlinks**

```bash
# Check if path contains symlinks
readlink -f /path/to/file

# Server may resolve symlinks
# Ensure final path is within root_dir
```

---

### Preview Not Refreshing

#### Symptom: File changes don't trigger preview refresh

**Check 1: File Watcher Status**

```javascript
// Browser console
const ws = new WebSocket('ws://localhost:3000/ws/system?session_id=xxx');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('System event:', data);
};

// Expected event when file changes:
{
  "type": "file_changed",
  "path": "/workspace/file.pdf",
  "mtime": 1703123456,
  "event": "modify"
}
```

**Check 2: File Watcher Configuration**

```toml
[features]
enable_watch = true  # Must be enabled

[server]
root_dir = "/path/to/workspace"  # Watch this directory
```

**Check 3: Backend File Watch**

```bash
# Check notify-rs is working
# Run in server directory
RUST_LOG=debug cargo run 2>&1 | grep -i watch

# Expected output:
# DEBUG notify: watching "/path/to/workspace"
```

**Check 4: Frontend Event Handling**

```javascript
// In pane component
$: if (data.type === 'file_changed' && data.path === filePath) {
  // Auto-refresh preview
  refreshPreview();
}
```

**Check 5: File Mtime**

```bash
# Check file modification time
stat /path/to/file

# Watcher uses mtime to detect changes
# If mtime doesn't change, file won't refresh
# Fix: Touch file to update mtime
touch /path/to/file
```

---

## Development Issues

### Build Failures

#### Rust Build Errors

**Error: `linker 'cc' not found`**

```bash
# Ubuntu/Debian
sudo apt install build-essential

# CentOS/RHEL
sudo yum groupinstall "Development Tools"

# macOS
xcode-select --install
```

**Error: `cannot find -l...` (missing libraries)**

```bash
# Install development libraries
sudo apt install -y \
  libssl-dev \
  pkg-config \
  libudev-dev
```

**Error: `error[E0554]: #![feature] may not be used on the stable channel`**

```bash
# Use nightly Rust or remove unstable features
rustup install stable
rustup default stable

# Check Rust version
rustc --version  # Should be 1.75+
```

---

#### Node.js Build Errors

**Error: `node-gyp` fails**

```bash
# Install build tools
# Ubuntu/Debian
sudo apt install -y python3 make g++

# macOS
xcode-select --install

# Windows
npm install -g windows-build-tools
```

**Error: `peer dep missing`**

```bash
# Clear npm cache
npm cache clean --force

# Delete node_modules
rm -rf node_modules package-lock.json

# Reinstall
npm install
```

**Error: `pnpm install` fails**

```bash
# Use npm instead
npm install
```

---

### Hot Reload Not Working

#### Frontend Changes Not Reflected

**Check Vite Config**

```typescript
// client/vite.config.ts
export default defineConfig({
  plugins: [svelte()],
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      '/api': 'http://127.0.0.1:3000',
      '/ws': {
        target: 'ws://127.0.0.1:3000',
        ws: true
      }
    }
  }
});
```

**Check Browser Cache**

```javascript
// Hard refresh
// Windows/Linux: Ctrl + Shift + R
// macOS: Cmd + Shift + R

// Or disable cache in DevTools
// DevTools -> Network tab -> Disable cache ✓
```

---

### TypeScript Errors

**Error: `Cannot find module './Component.svelte'`**

```typescript
// Add to tsconfig.json
{
  "compilerOptions": {
    "moduleResolution": "bundler",
    "types": ["svelte"]
  }
}
```

**Error: `Property does not exist on type`**

```typescript
// Use type assertion
import Component from './Component.svelte';

const comp = new Component({
  target: document.getElementById('app')
}) as unknown as Component;
```

---

## Performance Issues

### High CPU Usage

**Check 1: WebSocket Connections**

```bash
# Count active WebSocket connections
sudo netstat -an | grep 3000 | grep ESTABLISHED | wc -l

# If too many:
# - Implement connection limits
# - Check for memory leaks
# - Increase session timeout
```

**Check 2: File Watcher**

```bash
# Check watched directories
sudo lsof +D /path/to/workspace  # May be slow for large directories

# If too many files:
# - Exclude node_modules
# - Use .watchmanconfig
# - Increase debounce interval
```

**Check 3: PTY Processes**

```bash
# Check PTY count
ps aux | grep -E "bash|zsh" | wc -l

# If too many:
# - Limit concurrent terminals
# - Kill idle sessions
# - Reduce session timeout
```

**Solution: Optimize Configuration**

```toml
[server]
max_connections = 100        # Limit concurrent connections
session_timeout_minutes = 30 # Shorter timeout
worker_threads = 4           # Match CPU cores

[features]
enable_watch = true          # But exclude large directories
```

---

### High Memory Usage

**Check 1: Memory Leaks**

```bash
# Monitor memory usage
watch -n 1 'ps aux | grep liteterm'

# Check memory growth over time
# If constantly increasing, there's a leak
```

**Check 2: Large File Handling**

```toml
# Limit file sizes
[server]
max_file_size_mb = 100  # Reject files larger than 100MB
```

**Solution: Streaming**

```rust
// Stream large files instead of loading into memory
let file = tokio::fs::File::open(path).await?;
let stream = tokio::io::BufReader::new(file);
let body = Body::from_stream(stream);
```

---

### Slow File System Access

**Check 1: Network File Systems**

```bash
# Avoid NFS/SMB for root_dir
# These are slower and may have latency issues

# If using NFS:
# - Increase timeouts
# - Enable caching
# - Monitor network latency
```

**Check 2: Disk I/O**

```bash
# Check disk usage
df -h

# Check I/O wait
iostat -x 1

# If high I/O wait:
# - Upgrade to SSD
# - Reduce concurrent operations
# - Implement caching
```

---

## Platform-Specific Issues

### Linux

#### Permission Denied on PTY

```bash
# Check PTY permissions
ls -la /dev/ptmx
# Should be: crw-rw-rw- 1 root root 5, 2 ...

# Fix permissions
sudo chmod 666 /dev/ptmx
```

#### Missing PTY Support

```bash
# In container or chroot, PTY may not be available

# Check kernel support
cat /proc/sys/kernel/pty/max
# If file doesn't exist, PTY not supported

# Enable PTY in container:
# docker run --privileged ...
# or
docker run -v /dev/pts:/dev/pts ...
```

---

### macOS

#### PTY Path Differences

```rust
// macOS uses /dev/ttys* instead of /dev/pts/*
use portable_pty::{CommandBuilder, Pty, native_pty_system};

let pty_system = native_pty_system();
let (mut master, _) = pty_system.openpty().unwrap();
```

#### File Path Case Sensitivity

```bash
# macOS filesystem is case-insensitive by default
# Ensure consistent casing in paths

# Use realpath to resolve
realpath /path/to/File.txt
```

---

### Windows

#### PTY Support Limited

```bash
# Windows PTY support is experimental

# Use Windows Terminal (WT) instead
# Or run under WSL (Windows Subsystem for Linux)

# Enable WSL:
dism.exe /online /enable-feature /featurename:Microsoft-Windows-Subsystem-Linux /all /norestart
```

#### Path Separators

```toml
[server]
# Use forward slashes even on Windows
root_dir = "C:/Users/user/workspace"
# Or
root_dir = "C:\\Users\\user\\workspace"
```

---

### Docker

#### PTY Not Available in Container

```dockerfile
# In Dockerfile
FROM debian:bookworm-slim

# Install PTY support
RUN apt-get update && apt-get install -y \
    procps \
    util-linux \
    && rm -rf /var/lib/apt/lists/*

# Run with PTY
CMD ["./liteterm-web", "--config", "config.toml"]
```

```bash
# Run container with PTY
docker run -it liteterm-web
# -i: Interactive
# -t: Allocate TTY
```

#### Permission Issues

```dockerfile
# Use same UID/GID as host
RUN groupadd -r liteterm && useradd -r -g liteterm liteterm
USER liteterm

# Or mount with correct permissions
docker run \
  -v /host/workspace:/workspace \
  -u $(id -u):$(id -g) \
  liteterm-web
```

---

## Debugging

### Enable Debug Logging

#### Backend

```bash
# Runtime debug logging
RUST_LOG=debug ./server/target/release/liteterm-web

# Trace logging (most verbose)
RUST_LOG=trace ./server/target/release/liteterm-web

# Specific module
RUST_LOG=liteterm_web::ws::terminal=debug ./server/target/release/liteterm-web

# Save to file
RUST_LOG=debug ./server/target/release/liteterm-web 2>&1 | tee debug.log
```

```toml
# config.toml
[logging]
level = "debug"
file = "/var/log/liteterm-debug.log"
```

#### Frontend

```typescript
// In component
console.log('Debug info:', data);

// Enable verbose logging in browser DevTools console:
localStorage.setItem('debug', 'true');
```

---

### Network Debugging

#### Capture Traffic

```bash
# Using tcpdump
sudo tcpdump -i any -w traffic.pcap port 3000

# Using Wireshark
# Filter: tcp.port == 3000
```

#### WebSocket Testing

```javascript
// Simple WebSocket test in browser console
const testWS = () => {
  const ws = new WebSocket('ws://localhost:3000/ws/system?session_id=' + getSessionId());
  ws.onopen = () => console.log('WS connected');
  ws.onerror = (e) => console.error('WS error', e);
  ws.onmessage = (m) => console.log('WS message:', m.data);
};

testWS();
```

---

### Memory Debugging

#### Rust Memory Usage

```bash
# Install valgrind
sudo apt install valgrind

# Check memory leaks
valgrind --leak-check=full ./server/target/debug/liteterm-web

# Or use sanitizers
RUSTFLAGS="-Z sanitizer=address" cargo run
```

#### Frontend Memory Usage

```javascript
// Chrome DevTools -> Performance tab
// Record while using app
// Look for memory leaks in "Memory" chart

// Force garbage collection
// Chrome DevTools -> Console:
window.gc();
```

---

### CPU Profiling

#### Backend

```bash
# Install flamegraph
cargo install flamegraph

# Generate flamegraph
cargo flamegraph --bin liteterm-web

# Output: flamegraph.svg (open in browser)
```

#### Frontend

```javascript
// Chrome DevTools -> Performance tab
// Click Record
// Perform actions
// Stop and analyze
// Look for:
# - Long tasks (>50ms)
# - Expensive reflows
# - Memory leaks
```

---

## Getting Help

### Before Reporting an Issue

1. **Check this guide** - Search for your error
2. **Search existing issues** - Someone may have solved it
3. **Gather information**:
   - Operating system and version
   - Rust version (`rustc --version`)
   - Node.js version (`node --version`)
   - Full error message
   - Steps to reproduce

### Report Issue

#### GitHub Issue Template

```markdown
**Bug Description**
A clear description of the bug.

**Environment**
- OS: Ubuntu 22.04
- Rust: 1.75.0
- Node.js: 18.17.0
- Browser: Chrome 120

**Steps to Reproduce**
1. Start server
2. Open browser
3. Login with password
4. See error

**Expected Behavior**
What should happen.

**Actual Behavior**
What actually happens.

**Error Logs**
```
Paste relevant logs here
```

**Configuration**
```toml
Paste config.toml here
```
```

### Community Resources

- **GitHub Discussions**: [https://github.com/your-repo/liteterm-web/discussions](https://github.com/your-repo/liteterm-web/discussions)
- **Discord**: [https://discord.gg/your-server](https://discord.gg/your-server)
- **Documentation**: [README.md](../README.md)

### Diagnostic Script

Run this to collect system info:

```bash
#!/bin/bash
# collect-diagnostics.sh

echo "=== System Information ==="
uname -a
cat /etc/os-release 2>/dev/null || echo "OS info not available"

echo -e "\n=== Software Versions ==="
rustc --version 2>/dev/null || echo "Rust not installed"
node --version 2>/dev/null || echo "Node.js not installed"
npm --version 2>/dev/null || echo "npm not available"

echo -e "\n=== Port Usage ==="
netstat -tulnp 2>/dev/null | grep 3000 || echo "Port 3000 not in use"

echo -e "\n=== PTY Availability ==="
ls -la /dev/ptmx 2>/dev/null || echo "PTY not available"

echo -e "\n=== Log Files ==="
ls -la /var/log/liteterm.log 2>/dev/null || echo "Log file not found"

echo -e "\n=== Configuration ==="
cat server/config/config.toml 2>/dev/null || echo "Config file not found"

echo -e "\n=== Process Status ==="
ps aux | grep -E "liteterm|bash|zsh" | grep -v grep || echo "No processes found"
```

---

## Quick Reference

### Common Commands

```bash
# Start development
make dev

# Build release
make build

# Check logs
sudo journalctl -u liteterm -f

# Restart service
sudo systemctl restart liteterm

# Test API
curl http://localhost:3000/api/fs/list?path=/tmp

# Check port usage
sudo lsof -i :3000

# Test WebSocket
wscat -c ws://localhost:3000/ws/system?session_id=xxx
```

### Config Validation

```bash
# Validate TOML
python3 -c "import toml; toml.load('server/config/config.toml')"

# Or
cargo run --example validate_config
```

### Useful Links

- [Rust Book](https://doc.rust-lang.org/book/)
- [Svelte Tutorial](https://svelte.dev/tutorial)
- [Axum Docs](https://docs.rs/axum/)
- [WebSocket API](./API.md)

---

## Conclusion

Still stuck? Don't hesitate to:
1. Check [GitHub Issues](https://github.com/your-repo/liteterm-web/issues)
2. Create a new issue with diagnostic information
3. Join our [Discord community](https://discord.gg/your-server)

Happy coding! 🚀
