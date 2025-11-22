<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { Keyboard } from 'lucide-svelte';
  import type { KeyboardShortcut } from '../stores/settingsStore';

  export let shortcut: Partial<KeyboardShortcut> | null = null;
  export let isRecording = false;

  const dispatch = createEventDispatcher();

  let recordingKeys: string[] = [];
  let modifierKeys = {
    ctrl: false,
    alt: false,
    shift: false
  };

  function startRecording() {
    isRecording = true;
    recordingKeys = [];
    modifierKeys = { ctrl: false, alt: false, shift: false };
  }

  function stopRecording() {
    isRecording = false;
    if (recordingKeys.length > 0) {
      const newShortcut = {
        key: recordingKeys[recordingKeys.length - 1], // Use the last key as primary
        ctrlKey: modifierKeys.ctrl,
        altKey: modifierKeys.alt,
        shiftKey: modifierKeys.shift
      };
      dispatch('shortcut-recorded', newShortcut);
    }
    recordingKeys = [];
    modifierKeys = { ctrl: false, alt: false, shift: false };
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!isRecording) return;

    e.preventDefault();
    e.stopPropagation();

    // Check for modifier keys
    if (e.key === 'Control') {
      modifierKeys.ctrl = true;
    } else if (e.key === 'Alt') {
      modifierKeys.alt = true;
    } else if (e.key === 'Shift') {
      modifierKeys.shift = true;
    } else if (e.key === 'Meta') {
      // Treat Meta (Cmd) as Ctrl for cross-platform compatibility
      modifierKeys.ctrl = true;
    } else {
      // Regular key
      if (!recordingKeys.includes(e.key)) {
        recordingKeys.push(e.key);
      }
    }
  }

  function handleKeyup(e: KeyboardEvent) {
    if (!isRecording) return;

    // Stop recording when modifier keys are released
    if (e.key === 'Control' || e.key === 'Meta') {
      modifierKeys.ctrl = false;
      if (recordingKeys.length > 0) {
        stopRecording();
      }
    } else if (e.key === 'Alt') {
      modifierKeys.alt = false;
      if (recordingKeys.length > 0) {
        stopRecording();
      }
    } else if (e.key === 'Shift') {
      modifierKeys.shift = false;
      if (recordingKeys.length > 0) {
        stopRecording();
      }
    } else {
      // For regular keys, stop recording immediately
      if (recordingKeys.length > 0) {
        stopRecording();
      }
    }
  }

  function formatCurrentShortcut() {
    const parts: string[] = [];
    if (modifierKeys.ctrl) parts.push('Ctrl');
    if (modifierKeys.alt) parts.push('Alt');
    if (modifierKeys.shift) parts.push('Shift');
    if (recordingKeys.length > 0) {
      parts.push(recordingKeys[recordingKeys.length - 1].toUpperCase());
    }
    return parts.join(' + ') || 'Press keys...';
  }

  function cancelRecording() {
    isRecording = false;
    recordingKeys = [];
    modifierKeys = { ctrl: false, alt: false, shift: false };
  }

  $: if (isRecording) {
    // Add event listeners when recording starts
    document.addEventListener('keydown', handleKeydown, true);
    document.addEventListener('keyup', handleKeyup, true);
  } else {
    // Remove event listeners when recording stops
    document.removeEventListener('keydown', handleKeydown, true);
    document.removeEventListener('keyup', handleKeyup, true);
  }

  $: displayShortcut = shortcut ? [
    shortcut.ctrlKey ? 'Ctrl' : null,
    shortcut.altKey ? 'Alt' : null,
    shortcut.shiftKey ? 'Shift' : null,
    shortcut.key?.toUpperCase()
  ].filter(Boolean).join(' + ') : 'Not set';
</script>

<div class="shortcut-recorder">
  {#if isRecording}
    <div class="flex items-center gap-3 p-3 bg-accent-color/10 border-2 border-accent-color rounded-lg">
      <Keyboard size={20} class="text-accent-color animate-pulse" />
      <div class="flex-1">
        <div class="font-medium text-accent-color">Recording shortcut...</div>
        <div class="text-sm text-secondary-text">{formatCurrentShortcut()}</div>
      </div>
      <button
        class="px-3 py-1 text-sm bg-secondary-bg hover:bg-tertiary-bg border border-border-color rounded transition-colors"
        on:click={cancelRecording}
      >
        Cancel
      </button>
    </div>
  {:else}
    <div class="flex items-center gap-3 p-3 bg-primary-bg border border-border-color rounded-lg">
      <Keyboard size={20} class="text-secondary-text" />
      <div class="flex-1">
        <div class="font-mono text-sm text-primary-text">{displayShortcut}</div>
        <div class="text-xs text-secondary-text">Click to record new shortcut</div>
      </div>
      <button
        class="px-3 py-1 text-sm bg-accent-color hover:bg-blue-500 text-white border border-accent-color rounded transition-colors"
        on:click={startRecording}
      >
        Record
      </button>
    </div>
  {/if}
</div>

<style>
  .shortcut-recorder :global(.animate-pulse) {
    animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
  }

  @keyframes pulse {
    0%, 100% {
      opacity: 1;
    }
    50% {
      opacity: .5;
    }
  }
</style>