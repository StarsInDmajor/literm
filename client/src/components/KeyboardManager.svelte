<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { settingsStore, type KeyboardShortcut } from '../stores/settingsStore';

  let keydownHandler: (e: KeyboardEvent) => void;

  // Store for actions that can be triggered by shortcuts
  const actions: Record<string, () => void> = {};

  onMount(() => {
    console.log('KeyboardManager mounted');

    keydownHandler = (e: KeyboardEvent) => {
      // Check if keyboard shortcuts are enabled in settings
      if (!$settingsStore.keyboardShortcutsEnabled) {
        return;
      }

      // Debug: Log all key events for troubleshooting
      console.log(`Keyboard event: ${e.key} (Ctrl: ${e.ctrlKey}, Alt: ${e.altKey}, Shift: ${e.shiftKey})`);

      // Don't trigger shortcuts when typing in input fields, unless it's the terminal
      const target = e.target as HTMLElement;
      const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.contentEditable === 'true';
      // xterm.js uses a textarea with this class for input
      const isTerminal = target.classList.contains('xterm-helper-textarea');

      if (isInput && !isTerminal) {
        console.log('Ignoring shortcut - typing in input field');
        return;
      }

      const shortcuts = $settingsStore.keyboardShortcuts;
      console.log('Available shortcuts:', shortcuts);

      // Find matching shortcut
      for (const [id, shortcut] of Object.entries(shortcuts)) {
        console.log(`Checking ${id}:`, shortcut);
        if (matchesShortcut(e, shortcut as KeyboardShortcut)) {
          // Check if this specific shortcut is enabled
          if (!(shortcut as any).enabled) {
            console.log(`Shortcut ${id} is disabled, ignoring`);
            break;
          }

          console.log(`Matched shortcut: ${id}`);
          e.preventDefault();
          e.stopPropagation();

          const action = actions[id];
          if (action) {
            console.log(`Executing action for ${id}`);
            action();
          } else {
            console.log(`No action found for ${id}`);
          }
          break;
        } else {
          console.log(`No match for ${id}`);
        }
      }
    };

    document.addEventListener('keydown', keydownHandler, true); // Use capture phase
  });

  onDestroy(() => {
    if (keydownHandler) {
      document.removeEventListener('keydown', keydownHandler);
    }
  });

  function matchesShortcut(e: KeyboardEvent, shortcut: Partial<KeyboardShortcut>): boolean {
    if (!shortcut.key) return false;

    // Normalize key
    const eventKey = e.key.toLowerCase();
    const shortcutKey = shortcut.key.toLowerCase();

    // Check if key matches
    if (eventKey !== shortcutKey) return false;

    // Check modifier keys - Strict matching (undefined means false)
    if (!!shortcut.ctrlKey !== e.ctrlKey) return false;
    if (!!shortcut.altKey !== e.altKey) return false;
    if (!!shortcut.shiftKey !== e.shiftKey) return false;

    return true;
  }

  // These functions will be exposed through the slot prop and exported for bind:this
  export function registerAction(id: string, action: () => void) {
    console.log(`Registering action for ${id}`);
    actions[id] = action;
    console.log('Current actions:', Object.keys(actions));
  }

  export function unregisterAction(id: string) {
    delete actions[id];
  }

  </script>

<slot {registerAction} {unregisterAction} />