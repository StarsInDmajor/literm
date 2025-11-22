<script lang="ts">
  import { settingsStore, type KeyboardShortcut, defaultSettings } from '../../stores/settingsStore';
  import { X, Palette, Keyboard, RotateCcw, Check, AlertTriangle } from 'lucide-svelte';
  import { fade, scale } from 'svelte/transition';
  import ShortcutRecorder from '../ShortcutRecorder.svelte';

  export let isOpen = false;
  export let onClose: () => void;

  let activeTab: 'appearance' | 'shortcuts' = 'appearance';
  let editingShortcutId: string | null = null;
  let editingShortcut: Partial<KeyboardShortcut> | null = null;
  let conflictWarning: string | null = null;

  function save() {
    // Settings are auto-saved by the store, but we could add explicit save logic here if needed.
    onClose();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }

  function startEditingShortcut(id: string, shortcut: Partial<KeyboardShortcut>) {
    editingShortcutId = id;
    editingShortcut = { ...shortcut };
    conflictWarning = null;
  }

  function cancelEditing() {
    editingShortcutId = null;
    editingShortcut = null;
    conflictWarning = null;
  }

  function checkForConflict(newShortcut: Partial<KeyboardShortcut>, excludeId?: string): string | null {
    if (!newShortcut.key) return null;

    const shortcuts = $settingsStore.keyboardShortcuts || {};

    for (const [id, existingShortcut] of Object.entries(shortcuts)) {
      if (id === excludeId) continue;

      if (existingShortcut.key?.toLowerCase() === newShortcut.key?.toLowerCase() &&
          Boolean(existingShortcut.ctrlKey) === Boolean(newShortcut.ctrlKey) &&
          Boolean(existingShortcut.altKey) === Boolean(newShortcut.altKey) &&
          Boolean(existingShortcut.shiftKey) === Boolean(newShortcut.shiftKey)) {
        return `This shortcut is already used by: ${existingShortcut.description}`;
      }
    }

    return null;
  }

  function handleShortcutRecorded(e: CustomEvent<Partial<KeyboardShortcut>>) {
    if (editingShortcutId) {
      const newShortcut = e.detail;
      conflictWarning = checkForConflict(newShortcut, editingShortcutId);
      if (!conflictWarning) {
        editingShortcut = { ...editingShortcut, ...newShortcut };
      }
    }
  }

  function saveShortcut() {
    if (editingShortcutId && editingShortcut) {
      if (!conflictWarning) {
        settingsStore.updateShortcut(editingShortcutId, editingShortcut);
        cancelEditing();
      }
    }
  }

  function resetShortcut(id: string) {
    // Get default shortcut from the default settings
    const defaultShortcuts = defaultSettings.keyboardShortcuts as Record<string, Partial<KeyboardShortcut>>;

    const defaultShortcut = defaultShortcuts[id];
    if (defaultShortcut) {
      settingsStore.updateShortcut(id, defaultShortcut);
      if (editingShortcutId === id) {
        cancelEditing();
      }
    }
  }

  function resetAllShortcuts() {
    const defaultShortcuts = defaultSettings.keyboardShortcuts as Record<string, Partial<KeyboardShortcut>>;

    for (const [id, shortcut] of Object.entries(defaultShortcuts)) {
      settingsStore.updateShortcut(id, shortcut);
    }
    cancelEditing();
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
    on:click={handleBackdropClick}
    on:keydown={(e) => { if (e.key === 'Escape') onClose(); }}
    role="button" tabindex="0"
    transition:fade={{ duration: 200 }}
  >
    <div
      class="w-full max-w-2xl bg-secondary-bg border border-border-color rounded-lg shadow-xl overflow-hidden"
      transition:scale={{ duration: 200, start: 0.95 }}
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-border-color bg-tertiary-bg">
        <h2 class="text-lg font-semibold text-primary-text">Settings</h2>
        <button class="text-secondary-text hover:text-primary-text transition-colors" on:click={onClose}>
          <X size={20} />
        </button>
      </div>

      <!-- Tab Navigation -->
      <div class="flex border-b border-border-color bg-tertiary-bg">
        <button
          class="flex items-center gap-2 px-6 py-3 text-sm font-medium transition-colors border-b-2 {activeTab === 'appearance' ? 'text-accent-color border-accent-color bg-primary-bg' : 'text-secondary-text hover:text-primary-text border-transparent'}"
          on:click={() => activeTab = 'appearance'}
        >
          <Palette size={16} />
          Appearance
        </button>
        <button
          class="flex items-center gap-2 px-6 py-3 text-sm font-medium transition-colors border-b-2 {activeTab === 'shortcuts' ? 'text-accent-color border-accent-color bg-primary-bg' : 'text-secondary-text hover:text-primary-text border-transparent'}"
          on:click={() => activeTab = 'shortcuts'}
        >
          <Keyboard size={16} />
          Keyboard Shortcuts
        </button>
      </div>

      <!-- Body with Scrolling -->
      <div class="max-h-[500px] overflow-y-auto">
        {#if activeTab === 'appearance'}
          <!-- Appearance Tab Content -->
          <div class="p-6 space-y-6">
            <!-- Theme Selection -->
            <div class="space-y-2">
              <label for="theme" class="block text-sm font-medium text-secondary-text">Theme</label>
              <select
                id="theme"
                bind:value={$settingsStore.theme}
                on:change={() => settingsStore.updateSetting('theme', $settingsStore.theme)}
                class="w-full px-3 py-2 bg-primary-bg border border-border-color rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent-color text-sm"
              >
                <option value="dark">Dark</option>
                <option value="light">Light</option>
              </select>
            </div>

            <!-- Font Preview -->
            <div
              class="bg-primary-bg p-4 rounded-md border border-border-color font-mono text-primary-text text-left overflow-x-auto"
              style="
                font-size: {$settingsStore.fontSize}px;
                font-family: {$settingsStore.fontFamily};
                font-weight: {$settingsStore.fontWeight};
                line-height: {$settingsStore.lineHeight};
                font-feature-settings: {$settingsStore.enableLigatures ? '"liga" on, "calt" on' : '"liga" off, "calt" off'};
              "
            >
              <pre><code>{@html `// Optional comment -> true
const x = 10; x === 10; x !== 20;
a >= b && c <= d || e => f;
/* multiline */ /** doc */`}</code></pre>
            </div>

            <!-- Font Size -->
            <div class="space-y-2">
              <label for="fontSize" class="block text-sm font-medium text-secondary-text">
                Font Size ({$settingsStore.fontSize}px)
              </label>
              <input
                id="fontSize"
                type="range"
                min="10"
                max="24"
                step="1"
                bind:value={$settingsStore.fontSize}
                on:change={() => settingsStore.updateSetting('fontSize', $settingsStore.fontSize)}
                class="w-full h-2 bg-border-color rounded-lg appearance-none cursor-pointer accent-accent-color"
              />
            </div>

            <!-- Font Family -->
            <div class="space-y-2">
              <label for="fontFamily" class="block text-sm font-medium text-secondary-text">Font Family</label>
              <input
                id="fontFamily"
                type="text"
                bind:value={$settingsStore.fontFamily}
                on:change={() => settingsStore.updateSetting('fontFamily', $settingsStore.fontFamily)}
                class="w-full px-3 py-2 bg-primary-bg border border-border-color rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent-color text-sm"
                placeholder='e.g. "Fira Code", monospace'
              />
            </div>

            <!-- Font Weight -->
            <div class="space-y-2">
              <label for="fontWeight" class="block text-sm font-medium text-secondary-text">
                Font Weight ({$settingsStore.fontWeight})
              </label>
              <input
                id="fontWeight"
                type="range"
                min="100"
                max="900"
                step="100"
                bind:value={$settingsStore.fontWeight}
                on:change={() => settingsStore.updateSetting('fontWeight', $settingsStore.fontWeight)}
                class="w-full h-2 bg-border-color rounded-lg appearance-none cursor-pointer accent-accent-color"
              />
            </div>

            <!-- Line Height -->
            <div class="space-y-2">
              <label for="lineHeight" class="block text-sm font-medium text-secondary-text">
                Line Height ({$settingsStore.lineHeight})
              </label>
              <input
                id="lineHeight"
                type="range"
                min="0.8"
                max="2.0"
                step="0.1"
                bind:value={$settingsStore.lineHeight}
                on:change={() => settingsStore.updateSetting('lineHeight', $settingsStore.lineHeight)}
                class="w-full h-2 bg-border-color rounded-lg appearance-none cursor-pointer accent-accent-color"
              />
            </div>

            <!-- Ligatures -->
            <div class="flex items-center justify-between">
              <label for="ligatures" class="text-sm font-medium text-secondary-text">Enable Ligatures</label>
              <input
                id="ligatures"
                type="checkbox"
                bind:checked={$settingsStore.enableLigatures}
                on:change={() => settingsStore.updateSetting('enableLigatures', $settingsStore.enableLigatures)}
                class="w-4 h-4 text-accent-color bg-border-color border-border-color rounded focus:ring-accent-color focus:ring-2"
              />
            </div>

            <!-- Cursor Style -->
            <div class="space-y-2">
              <label for="cursorStyle" class="block text-sm font-medium text-secondary-text">Cursor Style</label>
              <select
                id="cursorStyle"
                bind:value={$settingsStore.cursorStyle}
                on:change={() => settingsStore.updateSetting('cursorStyle', $settingsStore.cursorStyle)}
                class="w-full px-3 py-2 bg-primary-bg border border-border-color rounded text-primary-text focus:outline-none focus:ring-2 focus:ring-accent-color text-sm"
              >
                <option value="block">Block</option>
                <option value="bar">Bar</option>
                <option value="underline">Underline</option>
              </select>
            </div>

            <div class="pt-4 border-t border-border-color">
              <button
                class="text-xs text-secondary-text hover:text-accent-color underline"
                on:click={() => settingsStore.reset()}
              >
                Reset to Defaults
              </button>
            </div>
          </div>
        {:else if activeTab === 'shortcuts'}
          <!-- Keyboard Shortcuts Tab Content -->
          <div class="p-6">
            <div class="space-y-6">
              <!-- Shortcuts Overview -->
              <div>
                <h3 class="text-lg font-semibold text-primary-text mb-4">Keyboard Shortcuts</h3>
                <p class="text-sm text-secondary-text mb-6">
                  Here are all the keyboard shortcuts available in LiteTerm. Click on any shortcut to see more details.
                </p>
              </div>

              
              <!-- Shortcuts Grid -->
              <div class="space-y-4">
                {#each Object.entries($settingsStore.keyboardShortcuts || {}) as [id, shortcut]}
                  {#if editingShortcutId === id}
                    <!-- Editing Mode -->
                    <div class="bg-primary-bg border-2 border-accent-color rounded-lg p-4">
                      <div class="space-y-4">
                        <div class="flex items-start justify-between">
                          <div class="flex-1 min-w-0">
                            <h4 class="font-medium text-primary-text mb-1">
                              {shortcut.description || 'No description'}
                            </h4>
                            <p class="text-xs text-secondary-text font-mono bg-secondary-bg px-2 py-1 rounded inline-block">
                              ID: {id}
                            </p>
                          </div>
                          <div class="flex items-center gap-2">
                            <button
                              class="p-1 text-secondary-text hover:text-accent-color transition-colors"
                              on:click={() => resetShortcut(id)}
                              title="Reset to default"
                            >
                              <RotateCcw size={16} />
                            </button>
                          </div>
                        </div>

                        <!-- Shortcut Recorder -->
                        <ShortcutRecorder
                          shortcut={editingShortcut}
                          isRecording={false}
                          on:shortcut-recorded={handleShortcutRecorded}
                        />

                        <!-- Conflict Warning -->
                        {#if conflictWarning}
                          <div class="flex items-start gap-2 p-3 bg-red-500/10 border border-red-500/30 rounded-lg">
                            <AlertTriangle size={16} class="text-red-400 mt-0.5" />
                            <div class="text-sm text-red-400">{conflictWarning}</div>
                          </div>
                        {/if}

                        <!-- Action Buttons -->
                        <div class="flex items-center justify-end gap-2">
                          <button
                            class="px-3 py-1 text-sm bg-secondary-bg hover:bg-tertiary-bg border border-border-color rounded transition-colors"
                            on:click={cancelEditing}
                          >
                            Cancel
                          </button>
                          <button
                            class="px-3 py-1 text-sm bg-accent-color hover:bg-blue-500 text-white border border-accent-color rounded transition-colors flex items-center gap-1"
                            class:opacity-50={!!conflictWarning}
                            class:cursor-not-allowed={!!conflictWarning}
                            on:click={saveShortcut}
                            disabled={!!conflictWarning}
                          >
                            <Check size={14} />
                            Save
                          </button>
                        </div>
                      </div>
                    </div>
                  {:else}
                    <!-- Normal Display Mode -->
                    <div class="bg-primary-bg border border-border-color rounded-lg p-4 hover:border-accent-color/50 transition-colors group {shortcut.enabled === false ? 'opacity-60' : ''}">
                      <div class="flex items-start justify-between">
                        <div class="flex-1 min-w-0">
                          <h4 class="font-medium text-primary-text mb-1 flex items-center gap-2">
                            {shortcut.description || 'No description'}
                            {#if !shortcut.enabled}
                              <span class="px-2 py-0.5 bg-gray-500 text-white text-xs rounded-full">Disabled</span>
                            {/if}
                          </h4>
                          <p class="text-xs text-secondary-text font-mono bg-secondary-bg px-2 py-1 rounded inline-block">
                            ID: {id}
                          </p>
                        </div>
                        <div class="flex items-center gap-2 ml-4">
                          <!-- Toggle Switch -->
                          <label class="relative inline-flex items-center cursor-pointer">
                            <input
                              type="checkbox"
                              bind:checked={shortcut.enabled}
                              on:change={() => {
                                const updatedShortcut = { ...shortcut, enabled: shortcut.enabled };
                                settingsStore.updateShortcut(id, updatedShortcut);
                              }}
                              class="sr-only peer"
                            />
                            <div class="w-9 h-5 bg-gray-200 peer-focus:outline-none peer-focus:ring-2 peer-focus:ring-accent-color/30 rounded-full peer dark:bg-gray-700 peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-accent-color"></div>
                          </label>

                          {#if shortcut.ctrlKey}
                            <span class="px-2 py-1 bg-tertiary-bg border border-border-color rounded text-xs font-medium text-secondary-text">Ctrl</span>
                          {/if}
                          {#if shortcut.altKey}
                            <span class="px-2 py-1 bg-tertiary-bg border border-border-color rounded text-xs font-medium text-secondary-text">Alt</span>
                          {/if}
                          {#if shortcut.shiftKey}
                            <span class="px-2 py-1 bg-tertiary-bg border border-border-color rounded text-xs font-medium text-secondary-text">Shift</span>
                          {/if}
                          <span class="px-3 py-1 {shortcut.enabled ? 'bg-accent-color text-white' : 'bg-gray-500 text-gray-300'} rounded text-sm font-medium border min-w-[40px] text-center">
                            {(shortcut.key || '?').toUpperCase()}
                          </span>
                          <button
                            class="p-1 text-secondary-text hover:text-accent-color opacity-0 group-hover:opacity-100 transition-all"
                            on:click={() => startEditingShortcut(id, shortcut)}
                            title="Edit shortcut"
                          >
                            <Keyboard size={14} />
                          </button>
                        </div>
                      </div>
                    </div>
                  {/if}
                {/each}
              </div>

              <!-- Additional Info -->
              <div class="bg-tertiary-bg border border-border-color rounded-lg p-4">
                <div class="flex items-start justify-between mb-3">
                  <h4 class="font-medium text-primary-text flex items-center gap-2">
                    <Keyboard size={16} class="text-accent-color" />
                    Tips & Tricks
                  </h4>
                  <button
                    class="px-3 py-1 text-xs bg-secondary-bg hover:bg-primary-bg border border-border-color rounded transition-colors flex items-center gap-1"
                    on:click={resetAllShortcuts}
                    title="Reset all shortcuts to defaults"
                  >
                    <RotateCcw size={12} />
                    Reset All
                  </button>
                </div>
                <ul class="text-sm text-secondary-text space-y-1">
                  <li>• Toggle individual shortcuts on/off using the switch next to each one</li>
                  <li>• Keyboard shortcuts work everywhere except when typing in input fields</li>
                  <li>• Use Ctrl+, to quickly open this settings panel</li>
                  <li>• Click the keyboard icon on any shortcut to customize it</li>
                  <li>• Most shortcuts follow common editor conventions</li>
                  <li>• Conflicts are automatically detected and prevented</li>
                  <li>• Disabled shortcuts show a gray label and reduced opacity</li>
                </ul>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 bg-tertiary-bg flex justify-end">
        <button 
          class="px-4 py-2 bg-accent-color hover:bg-blue-500 text-white text-sm font-medium rounded transition-colors"
          on:click={save}
        >
          Done
        </button>
      </div>
    </div>
  </div>
{/if}