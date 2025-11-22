import { writable } from 'svelte/store';

export interface KeyboardShortcut {
  key: string;
  ctrlKey?: boolean;
  altKey?: boolean;
  shiftKey?: boolean;
  description: string;
  enabled: boolean; // Individual enable/disable flag
  action?: () => void; // Made optional since it's not stored
}

export interface Settings {
  theme: 'dark' | 'light'; // Added for theme setting
  fontSize: number;
  fontFamily: string;
  fontWeight: number;
  lineHeight: number;
  enableLigatures: boolean;
  cursorStyle: 'block' | 'bar' | 'underline';
  keyboardShortcutsEnabled: boolean; // Added to enable/disable keyboard shortcuts
  keyboardShortcuts: Record<string, Partial<KeyboardShortcut>>;
}

export const defaultSettings: Settings = {
  theme: 'dark', // Default theme
  fontSize: 14,
  fontFamily: '"Maple Mono", "JetBrains Mono", monospace',
  fontWeight: 400, // Default to Regular
  lineHeight: 1.2, // Default line height
  enableLigatures: true,
  cursorStyle: 'block', // Default cursor style
  keyboardShortcutsEnabled: true, // Default to enabled
  keyboardShortcuts: {
    'split-horizontal': {
      key: 'h',
      altKey: true,
      description: 'Split pane horizontally',
      enabled: true
    },
    'split-vertical': {
      key: 'v',
      altKey: true,
      description: 'Split pane vertically',
      enabled: true
    },
    'close-pane': {
      key: 'w',
      altKey: true,
      description: 'Close current pane',
      enabled: true
    },
    'toggle-file-explorer': {
      key: 'e',
      ctrlKey: true,
      description: 'Toggle file explorer',
      enabled: true
    },
    'open-settings': {
      key: ',',
      ctrlKey: true,
      description: 'Open settings',
      enabled: true
    },
    'focus-next-pane': {
      key: 'Tab',
      ctrlKey: true,
      description: 'Focus next pane',
      enabled: true
    },
    'focus-prev-pane': {
      key: 'Tab',
      ctrlKey: true,
      shiftKey: true,
      description: 'Focus previous pane',
      enabled: true
    },
    'refresh': {
      key: 'r',
      ctrlKey: true,
      description: 'Refresh current pane',
      enabled: true
    }
  }
};

function createSettingsStore() {
  // Try to load from localStorage
  const stored = localStorage.getItem('liteterm-settings');
  let initial: Settings;

  try {
    if (stored) {
      const parsed = JSON.parse(stored);
      // Validate the loaded data has the required structure
      if (parsed && typeof parsed === 'object') {
        // Migrate keyboard shortcuts to include enabled flag
        const migratedShortcuts = parsed.keyboardShortcuts || defaultSettings.keyboardShortcuts;
        for (const [id, shortcut] of Object.entries(migratedShortcuts)) {
          if (typeof shortcut === 'object' && shortcut !== null) {
            // Add enabled flag if it doesn't exist
            if ((shortcut as any).enabled === undefined) {
              (shortcut as any).enabled = true;
            }
            // Remove action property if it exists (it shouldn't be stored)
            if ((shortcut as any).action !== undefined) {
              delete (shortcut as any).action;
            }
          }
        }

        initial = {
          theme: parsed.theme || defaultSettings.theme,
          fontSize: parsed.fontSize || defaultSettings.fontSize,
          fontFamily: parsed.fontFamily || defaultSettings.fontFamily,
          fontWeight: parsed.fontWeight || defaultSettings.fontWeight,
          lineHeight: parsed.lineHeight || defaultSettings.lineHeight,
          enableLigatures: parsed.enableLigatures !== undefined ? parsed.enableLigatures : defaultSettings.enableLigatures,
          cursorStyle: parsed.cursorStyle || defaultSettings.cursorStyle,
          keyboardShortcutsEnabled: parsed.keyboardShortcutsEnabled !== undefined ? parsed.keyboardShortcutsEnabled : defaultSettings.keyboardShortcutsEnabled,
          keyboardShortcuts: migratedShortcuts
        };
      } else {
        throw new Error('Invalid settings data');
      }
    } else {
      initial = defaultSettings;
    }
  } catch (error) {
    console.error('Error loading settings, using defaults:', error);
    initial = defaultSettings;
    // Clear corrupted localStorage
    localStorage.removeItem('liteterm-settings');
  }

  const { subscribe, update, set } = writable<Settings>(initial);

  return {
    subscribe,
    update,
    set: (settings: Settings) => {
        set(settings);
        localStorage.setItem('liteterm-settings', JSON.stringify(settings));
    },
    updateSetting: <K extends keyof Settings>(key: K, value: Settings[K]) => {
      update(s => {
        const newSettings = { ...s, [key]: value };
        localStorage.setItem('liteterm-settings', JSON.stringify(newSettings));
        return newSettings;
      });
    },
    reset: () => {
        set(defaultSettings);
        localStorage.setItem('liteterm-settings', JSON.stringify(defaultSettings));
    },
    updateShortcut: (id: string, shortcut: Partial<KeyboardShortcut>) => {
      update(s => {
        const newSettings = {
          ...s,
          keyboardShortcuts: {
            ...s.keyboardShortcuts,
            [id]: shortcut
          }
        };
        localStorage.setItem('liteterm-settings', JSON.stringify(newSettings));
        return newSettings;
      });
    },
    getShortcut: (id: string): Partial<KeyboardShortcut> | undefined => {
      let current: Settings | undefined;
      const unsubscribe = subscribe(settings => {
        current = settings;
      });
      unsubscribe();
      return current?.keyboardShortcuts[id];
    }
  };
}

export const settingsStore = createSettingsStore();
