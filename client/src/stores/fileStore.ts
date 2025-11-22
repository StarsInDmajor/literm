import { writable } from 'svelte/store';

interface FilePreviewRequest {
  paneId: string;
  filePath: string;
  timestamp: number;
}

function createFileStore() {
  const { subscribe, set, update } = writable<FilePreviewRequest | null>(null);

  return {
    subscribe,

    // Request file preview in a specific pane
    requestPreview: (paneId: string, filePath: string) => {
      set({
        paneId,
        filePath,
        timestamp: Date.now()
      });
    },

    // Clear preview request
    clearPreview: () => {
      set(null);
    },

    // Get current preview request
    getCurrent: () => {
      let current: FilePreviewRequest | null = null;
      subscribe(value => current = value)();
      return current;
    }
  };
}

export const fileStore = createFileStore();

// Store for file paths by pane ID
function createFilePathStore() {
  const { subscribe, set, update } = writable<Map<string, string>>(new Map());

  return {
    subscribe,

    // Set file path for a pane
    setFilePath: (paneId: string, filePath: string) => {
      update(paths => {
        const newPaths = new Map(paths);
        newPaths.set(paneId, filePath);
        return newPaths;
      });
    },

    // Get file path for a pane
    getFilePath: (paneId: string): string | undefined => {
      let paths: Map<string, string> = new Map();
      subscribe(value => paths = value)();
      return paths.get(paneId);
    },

    // Clear file path for a pane
    clearFilePath: (paneId: string) => {
      update(paths => {
        const newPaths = new Map(paths);
        newPaths.delete(paneId);
        return newPaths;
      });
    }
  };
}

export const filePathStore = createFilePathStore();