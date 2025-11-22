<script lang="ts">
  import { onMount } from 'svelte';
  import { File, Folder, FolderOpen, ChevronRight, ChevronDown, Home, ArrowLeft, Download, Eye, EyeOff, FolderInput } from 'lucide-svelte';
  import { layoutStore } from '../../stores/layoutStore';
  import { fileStore, filePathStore, explorerPathStore } from '../../stores/fileStore';

  export let id: string;

  interface FileEntry {
    name: string;
    entry_type: 'file' | 'dir';
    size: number;
    mtime: number;
  }

  interface FileSystemNode {
    name: string;
    path: string;
    type: 'file' | 'dir';
    size?: number;
    mtime?: number;
    children?: FileSystemNode[];
    isExpanded?: boolean;
  }

  let currentPath = '';
  let fileSystem: FileSystemNode | null = null;
  let selectedNode: FileSystemNode | null = null;
  let loading = false;
  let error: string | null = null;
  let showHidden = false;
  let isChangeRootModalOpen = false;
  let newRootPath = '';

  onMount(() => {
    console.log(`[FileExplorerPane ${id}] Mounted`);
    const savedPath = explorerPathStore.getPath(id);
    if (savedPath !== undefined) {
        currentPath = savedPath;
    }
    loadDirectory(currentPath);
  });

  async function loadDirectory(path: string) {
    loading = true;
    error = null;

    try {
      const response = await fetch(`/api/fs/list?path=${encodeURIComponent(path)}&show_hidden=${showHidden}`);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      const data = await response.json();

      if (data.ok) {
        currentPath = data.path || path;
        explorerPathStore.setPath(id, currentPath);
        const entries = data.entries || [];

        // Sort directories first, then files, both alphabetically
        entries.sort((a: FileEntry, b: FileEntry) => {
          if (a.entry_type !== b.entry_type) {
            return a.entry_type === 'dir' ? -1 : 1;
          }
          return a.name.localeCompare(b.name);
        });

        fileSystem = {
          name: getCurrentDirName(),
          path: currentPath,
          type: 'dir',
          children: entries.map((entry: FileEntry) => ({
            name: entry.name,
            path: currentPath === '' ? entry.name : `${currentPath}/${entry.name}`,
            type: entry.entry_type,
            size: entry.size,
            mtime: entry.mtime,
            children: entry.entry_type === 'dir' ? [] : undefined,
            isExpanded: false
          }))
        };
      } else {
        throw new Error('Failed to load directory');
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unknown error';
      console.error(`[FileExplorerPane ${id}] Error loading directory:`, err);
    } finally {
      loading = false;
    }
  }

  function getCurrentDirName(): string {
    if (currentPath === '' || currentPath === '/') return 'Root';
    const parts = currentPath.split('/').filter(Boolean);
    return parts[parts.length - 1] || 'Root';
  }

  function formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${Math.round((bytes / Math.pow(k, i)) * 100) / 100} ${sizes[i]}`;
  }

  function formatDate(timestamp: number): string {
    return new Date(timestamp * 1000).toLocaleString();
  }

  async function handleNodeClick(node: FileSystemNode) {
    selectedNode = node;

    if (node.type === 'dir') {
      // Enter the folder in the same pane
      await loadDirectory(node.path);
    } else {
      // Handle file click - could open in preview pane or terminal
      handleFileClick(node);
    }
  }

  function handleFileClick(node: FileSystemNode) {
    // Single click just selects the file
    selectedNode = node;
    console.log(`[FileExplorerPane ${id}] File selected:`, node.path);
  }

  function handleFileDoubleClick(node: FileSystemNode) {
    console.log(`[FileExplorerPane ${id}] File double-clicked:`, node.path);

    // Set file path for this pane
    filePathStore.setFilePath(id, node.path);

    // Change pane type to preview and trigger file loading
    layoutStore.changePaneType(id, 'preview');
    fileStore.requestPreview(id, node.path);
  }

  async function downloadFile(node: FileSystemNode) {
    try {
      const response = await fetch(`/api/fs/raw?path=${encodeURIComponent(node.path)}`);
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      // Get the blob and create download link
      const blob = await response.blob();
      const url = window.URL.createObjectURL(blob);

      // Create download link and trigger download
      const link = document.createElement('a');
      link.href = url;
      link.download = node.name;
      link.target = '_blank';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);

      // Clean up the object URL
      window.URL.revokeObjectURL(url);

      console.log(`[FileExplorerPane ${id}] Downloaded file: ${node.name}`);
    } catch (err) {
      console.error(`[FileExplorerPane ${id}] Error downloading file:`, err);
      // TODO: Show error message to user
    }
  }

  async function goToParent() {
    if (currentPath === '' || currentPath === '/') return;

    const parts = currentPath.split('/').filter(Boolean);
    parts.pop();
    const parentPath = parts.length > 0 ? parts.join('/') : '';

    await loadDirectory(parentPath);
  }

  async function goHome() {
    await loadDirectory('');
  }

  async function refresh() {
    console.log(`[FileExplorerPane ${id}] Refreshing directory: ${currentPath}`);

    try {
      // Add timeout to prevent hanging
      const timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Refresh timeout')), 10000);
      });

      await Promise.race([loadDirectory(currentPath), timeoutPromise]);
    } catch (err) {
      console.error(`[FileExplorerPane ${id}] Refresh failed:`, err);
      error = err instanceof Error ? err.message : 'Refresh failed';
      loading = false;
    }
  }

  async function toggleHiddenFiles() {
    showHidden = !showHidden;
    await refresh();
  }

  function openChangeRootModal() {
    isChangeRootModalOpen = true;
    newRootPath = '';
  }

  async function changeRootDirectory() {
    if (!newRootPath) return;

    try {
        const response = await fetch('/api/fs/root', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ new_root: newRootPath })
        });

        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        const data = await response.json();
        if (data.ok) {
            isChangeRootModalOpen = false;
            // Refresh completely to go to new root
            currentPath = '';
            await loadDirectory('');
        } else {
            throw new Error('Failed to change root directory');
        }
    } catch (err) {
        console.error(`[FileExplorerPane ${id}] Error changing root directory:`, err);
        error = err instanceof Error ? err.message : 'Failed to change root directory';
    }
  }

  // Tree rendering function
  function renderTree(node: FileSystemNode, depth: number = 0): string {
    if (!node.children || node.children.length === 0) {
      return `<div class="tree-item" data-path="${node.path}">
        <span class="tree-indent" style="padding-left: ${depth * 16}px"></span>
        ${node.type === 'dir' ?
          '<Folder size={14} class="text-yellow-600" />' :
          '<File size={14} class="text-blue-600" />'}
        <span class="tree-name">${node.name}</span>
      </div>`;
    }

    return `<div class="tree-node" data-path="${node.path}">
      <div class="tree-item" data-path="${node.path}">
        <span class="tree-indent" style="padding-left: ${depth * 16}px"></span>
        ${node.isExpanded ?
          '<ChevronDown size={12} class="text-gray-500" />' :
          '<ChevronRight size={12} class="text-gray-500" />'}
        ${node.type === 'dir' ?
          (node.isExpanded ?
            '<FolderOpen size={14} class="text-yellow-600" />' :
            '<Folder size={14} class="text-yellow-600" />') :
          '<File size={14} class="text-blue-600" />'}
        <span class="tree-name font-medium">${node.name}</span>
      </div>
      ${node.isExpanded && node.children ?
        node.children.map(child => renderTree(child, depth + 1)).join('') :
        ''}
    </div>`;
  }
</script>

<div class="flex flex-col h-full bg-primary-bg relative">
  <!-- Toolbar -->
  <div class="flex items-center justify-between px-3 py-2 border-b border-border-color bg-tertiary-bg">
    <div class="flex items-center space-x-2">
      <button
        class="p-1 hover:bg-secondary-bg rounded transition-colors"
        on:click={goHome}
        title="Go to root"
      >
        <Home size={14} />
      </button>
      <button
        class="p-1 hover:bg-secondary-bg rounded transition-colors"
        on:click={goToParent}
        title="Go to parent directory"
        disabled={currentPath === '' || currentPath === '/'}
        class:opacity-50={currentPath === '' || currentPath === '/'}
        class:cursor-not-allowed={currentPath === '' || currentPath === '/'}
      >
        <ArrowLeft size={14} />
      </button>
      <button
        class="p-1 hover:bg-secondary-bg rounded transition-colors"
        class:animate-spin={loading}
        on:click={refresh}
        title="Refresh"
        disabled={loading}
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M23 4v6h-6M1 20v-6h6M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0 0 20.49 15"/>
        </svg>
      </button>
      <div class="w-px h-4 bg-border-color mx-1"></div>
      <button
        class="p-1 hover:bg-secondary-bg rounded transition-colors"
        class:text-accent-color={showHidden}
        on:click={toggleHiddenFiles}
        title={showHidden ? "Hide hidden files" : "Show hidden files"}
      >
        {#if showHidden}
            <Eye size={14} />
        {:else}
            <EyeOff size={14} />
        {/if}
      </button>
      <button
        class="p-1 hover:bg-secondary-bg rounded transition-colors"
        on:click={openChangeRootModal}
        title="Change root directory"
      >
        <FolderInput size={14} />
      </button>
    </div>
    <div class="text-sm text-secondary-text font-medium">
      {currentPath === '' ? '/' : currentPath}
    </div>
  </div>

  <!-- Content Area -->
  <div class="flex-1 overflow-auto">
    {#if loading}
      <div class="flex items-center justify-center h-32">
        <div class="text-secondary-text">Loading...</div>
      </div>
    {:else if error}
      <div class="flex flex-col items-center justify-center h-32 p-4">
        <div class="text-red-400 mb-2">Error</div>
        <div class="text-sm text-secondary-text text-center">{error}</div>
      </div>
    {:else if fileSystem}
      <!-- List View -->
      <div class="min-h-full">
        {#if fileSystem.children && fileSystem.children.length > 0}
          {#each fileSystem.children as node}
            <div
              role="button"
              tabindex="0"
              aria-label={`${node.type === 'dir' ? 'Directory' : 'File'}: ${node.name}`}
              class="flex items-center px-3 py-2 hover:bg-secondary-bg cursor-pointer border-b border-border-color/50"
              class:bg-secondary-bg={selectedNode?.path === node.path}
              on:click={() => handleNodeClick(node)}
              on:dblclick={() => handleFileDoubleClick(node)}
              on:keydown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  handleNodeClick(node);
                }
              }}
            >
              {#if node.type === 'dir'}
                {#if node.isExpanded}
                  <ChevronDown size={12} class="text-gray-500 mr-1" />
                  <FolderOpen size={16} class="text-yellow-600 mr-2" />
                {:else}
                  <ChevronRight size={12} class="text-gray-500 mr-1" />
                  <Folder size={16} class="text-yellow-600 mr-2" />
                {/if}
              {:else}
                <div class="w-4 mr-3"></div>
                <File size={16} class="text-blue-600 mr-2" />
              {/if}

              <div class="flex-1 min-w-0">
                <div class="font-medium text-primary-text truncate">{node.name}</div>
              </div>

              <div class="text-right text-xs text-secondary-text ml-4 flex items-center gap-2">
                {#if node.size !== undefined}
                  <div>{formatFileSize(node.size)}</div>
                {/if}
                {#if node.mtime}
                  <div>{formatDate(node.mtime)}</div>
                {/if}
                {#if node.type === 'file'}
                  <button
                    class="p-1 hover:bg-secondary-bg rounded transition-colors"
                    on:click|stopPropagation={() => downloadFile(node)}
                    title="Download file"
                  >
                    <Download size={12} />
                  </button>
                {/if}
              </div>
            </div>

            <!-- Expanded children -->
            {#if node.isExpanded && node.children}
              {#each node.children as child}
                <div
                  role="button"
                  tabindex="0"
                  aria-label={`${child.type === 'dir' ? 'Directory' : 'File'}: ${child.name}`}
                  class="flex items-center px-3 py-2 hover:bg-secondary-bg cursor-pointer border-b border-border-color/50"
                  class:bg-secondary-bg={selectedNode?.path === child.path}
                  style="padding-left: {32 + 16}px"
                  on:click={() => handleNodeClick(child)}
                  on:dblclick={() => handleFileDoubleClick(child)}
                  on:keydown={(e) => {
                    if (e.key === 'Enter' || e.key === ' ') {
                      e.preventDefault();
                      handleNodeClick(child);
                    }
                  }}
                >
                  {#if child.type === 'dir'}
                    {#if child.isExpanded}
                      <ChevronDown size={12} class="text-gray-500 mr-1" />
                      <FolderOpen size={16} class="text-yellow-600 mr-2" />
                    {:else}
                      <ChevronRight size={12} class="text-gray-500 mr-1" />
                      <Folder size={16} class="text-yellow-600 mr-2" />
                    {/if}
                  {:else}
                    <div class="w-4 mr-3"></div>
                    <File size={16} class="text-blue-600 mr-2" />
                  {/if}

                  <div class="flex-1 min-w-0">
                    <div class="text-primary-text truncate">{child.name}</div>
                  </div>

                  <div class="text-right text-xs text-secondary-text ml-4 flex items-center gap-2">
                    {#if child.size !== undefined}
                      <div>{formatFileSize(child.size)}</div>
                    {/if}
                    {#if child.mtime}
                      <div>{formatDate(child.mtime)}</div>
                    {/if}
                    {#if child.type === 'file'}
                      <button
                        class="p-1 hover:bg-secondary-bg rounded transition-colors"
                        on:click|stopPropagation={() => downloadFile(child)}
                        title="Download file"
                      >
                        <Download size={12} />
                      </button>
                    {/if}
                  </div>
                </div>
              {/each}
            {/if}
          {/each}
        {:else}
          <div class="flex items-center justify-center h-32">
            <div class="text-secondary-text">Directory is empty</div>
          </div>
        {/if}
      </div>
    {:else}
      <div class="flex items-center justify-center h-32">
        <div class="text-secondary-text">No directory loaded</div>
      </div>
    {/if}
  </div>

  <!-- Change Root Modal -->
  {#if isChangeRootModalOpen}
    <div class="absolute inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm p-4">
        <div class="bg-secondary-bg border border-border-color rounded-lg shadow-xl w-full max-w-sm overflow-hidden">
            <div class="px-4 py-3 border-b border-border-color bg-tertiary-bg flex justify-between items-center">
                <h3 class="font-semibold text-primary-text">Change Root Directory</h3>
                <button 
                    class="text-secondary-text hover:text-primary-text"
                    on:click={() => isChangeRootModalOpen = false}
                >
                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <path d="M18 6L6 18M6 6l12 12"/>
                    </svg>
                </button>
            </div>
            <div class="p-4">
                <p class="text-xs text-secondary-text mb-2">Enter absolute path for new root:</p>
                <input 
                    type="text" 
                    bind:value={newRootPath}
                    class="w-full px-3 py-2 bg-primary-bg border border-border-color rounded text-primary-text focus:outline-none focus:ring-1 focus:ring-accent-color text-sm"
                    placeholder="/home/user/..."
                    on:keydown={(e) => e.key === 'Enter' && changeRootDirectory()}
                />
            </div>
            <div class="px-4 py-3 bg-tertiary-bg flex justify-end gap-2">
                <button 
                    class="px-3 py-1.5 text-xs bg-secondary-bg hover:bg-primary-bg border border-border-color rounded transition-colors"
                    on:click={() => isChangeRootModalOpen = false}
                >
                    Cancel
                </button>
                <button 
                    class="px-3 py-1.5 text-xs bg-accent-color hover:bg-blue-600 text-white rounded transition-colors"
                    on:click={changeRootDirectory}
                >
                    Change
                </button>
            </div>
        </div>
    </div>
  {/if}
</div>