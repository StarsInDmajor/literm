<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Terminal } from 'xterm';
  import { FitAddon } from 'xterm-addon-fit';
  import { settingsStore } from '../../stores/settingsStore';

  export let id: string; // Pane ID

  let termContainer: HTMLDivElement;
  let term: Terminal;
  let fitAddon: FitAddon;
  let resizeObserver: ResizeObserver | undefined;
  let socket: WebSocket;
  let isManualResizeMode = false; // Flag to prevent automatic resize during pane operations
  let lastTabTime = 0; // Track tab presses to prevent rapid firing

  // Define xterm.js themes
  const darkTheme = {
    background: '#1a1b26', // Primary background color
    foreground: '#c0caf5', // Primary text color
    cursor: '#7aa2f7', // Accent color
    cursorAccent: '#1a1b26', // Text color under cursor (matches background)
    selectionBackground: '#545c7e',
    black: '#1a1b26',
    red: '#f7768e',
    green: '#9ece6a',
    yellow: '#e0af68',
    blue: '#7aa2f7',
    magenta: '#ad8ee6',
    cyan: '#4fd6ed',
    white: '#c0caf5',
    brightBlack: '#414868',
    brightRed: '#f7768e',
    brightGreen: '#9ece6a',
    brightYellow: '#e0af68',
    brightBlue: '#7aa2f7',
    brightMagenta: '#ad8ee6',
    brightCyan: '#4fd6ed',
    brightWhite: '#ffffff',
  };

  const lightTheme = {
    background: '#ffffff', // Primary background color
    foreground: '#333333', // Primary text color
    cursor: '#007bff', // Accent color
    cursorAccent: '#ffffff', // Text color under cursor (matches background)
    selectionBackground: '#d4e0ff',
    black: '#000000',
    red: '#dc3545',
    green: '#28a745',
    yellow: '#ffc107',
    blue: '#007bff',
    magenta: '#6f42c1',
    cyan: '#17a2b8',
    white: '#cccccc',
    brightBlack: '#6c757d',
    brightRed: '#dc3545',
    brightGreen: '#28a745',
    brightYellow: '#ffc107',
    brightBlue: '#007bff',
    brightMagenta: '#6f42c1',
    brightCyan: '#17a2b8',
    brightWhite: '#ffffff',
  };

  // Reactively update terminal options and theme when settings change
  $: if (term && $settingsStore) {
    term.options.fontSize = $settingsStore.fontSize;
    term.options.fontFamily = $settingsStore.fontFamily;
    term.options.fontWeight = $settingsStore.fontWeight;
    term.options.lineHeight = $settingsStore.lineHeight;
    term.options.cursorStyle = $settingsStore.cursorStyle;

    // Apply theme
    term.options.theme = $settingsStore.theme === 'dark' ? darkTheme : lightTheme;

    // Resize might be needed if font size changes
    fitAddon?.fit();
  }

  onMount(() => {
    console.log(`[TerminalPane ${id}] Mounting terminal pane`);

    // 1. Initialize xterm
    term = new Terminal({
      cursorBlink: true,
      fontSize: $settingsStore.fontSize,
      fontFamily: $settingsStore.fontFamily,
      fontWeight: $settingsStore.fontWeight,
      lineHeight: $settingsStore.lineHeight,
      cursorStyle: $settingsStore.cursorStyle, // Initial cursor style
      theme: $settingsStore.theme === 'dark' ? darkTheme : lightTheme, // Initial theme
      allowProposedApi: true
    });

    fitAddon = new FitAddon();
    term.loadAddon(fitAddon);

    // 2. Mount to DOM
    term.open(termContainer);

    // 3. Connect to WebSocket
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    socket = new WebSocket(`${protocol}//${window.location.host}/ws/term`);
    socket.binaryType = 'arraybuffer';

    socket.onopen = () => {
      term.writeln('\x1b[1;32mLiteTerm-Web\x1b[0m Connected.');
      fitAddon.fit(); // Initial fit on connection open
      sendResize(term.rows, term.cols); // Send initial resize to backend
      term.focus();
    };

    socket.onmessage = (event) => {
      if (event.data instanceof ArrayBuffer) {
        // Received raw binary output from PTY
        const array = new Uint8Array(event.data);
        term.write(array);
      }
    };

    socket.onclose = () => {
      term.writeln('\r\n\x1b[1;31mConnection closed.\x1b[0m');
    };

    socket.onerror = (err) => {
      console.error('WebSocket error:', err);
      term.writeln('\r\n\x1b[1;31mConnection error.\x1b[0m');
    };

    // 4. Handle Input
    term.onData((data) => {
      if (socket && socket.readyState === WebSocket.OPEN) {
        // Handle tab character with debouncing to prevent excessive scrolling
        if (data === '\t') {
          const now = Date.now();

          // Prevent rapid tab presses (within 200ms) that can cause scroll issues
          if (now - lastTabTime < 200) {
            return;
          }

          lastTabTime = now;

          // Send tab character with a small delay to allow terminal to process properly
          setTimeout(() => {
            const encoder = new TextEncoder();
            const rawData = encoder.encode('\t');
            const payload = new Uint8Array(rawData.length + 1);
            payload[0] = 0x01;
            payload.set(rawData, 1);
            socket.send(payload);
          }, 50);

          return;
        }

        // Normal data handling
        const encoder = new TextEncoder();
        const rawData = encoder.encode(data);
        const payload = new Uint8Array(rawData.length + 1);
        payload[0] = 0x01;
        payload.set(rawData, 1);
        socket.send(payload);
      }
    });

    // 5. Handle Resize - DISABLED to prevent refresh during split
    console.log(`[TerminalPane ${id}] ResizeObserver disabled to preserve terminal during split`);
  });

  function sendResize(rows: number, cols: number) {
    // Protocol: 0x02 + rows(u16) + cols(u16) - Big Endian
    const buffer = new ArrayBuffer(5);
    const view = new DataView(buffer);
    view.setUint8(0, 0x02);
    view.setUint16(1, rows, false); // Big Endian
    view.setUint16(3, cols, false); // Big Endian
    socket.send(buffer);
  }

  // Expose a refresh function for parent components
  export function refreshTerminalSize() {
    if (term && fitAddon) {
      isManualResizeMode = true; // Enable manual mode for this operation
      fitAddon.fit();
      if (socket && socket.readyState === WebSocket.OPEN) {
        sendResize(term.rows, term.cols);
      }
      // Disable manual mode after a short delay
      setTimeout(() => {
        isManualResizeMode = false;
      }, 500);
    }
  }

  // Enable manual resize mode for pane operations
  export function enableManualResize() {
    isManualResizeMode = true;
  }

  // Disable manual resize mode
  export function disableManualResize() {
    isManualResizeMode = false;
  }

  onDestroy(() => {
    console.log(`[TerminalPane ${id}] Destroying terminal pane`);
    socket?.close();
    term?.dispose();
    resizeObserver?.disconnect();
  });
</script>

<div class="w-full h-full bg-[#1a1b26]" bind:this={termContainer}></div>