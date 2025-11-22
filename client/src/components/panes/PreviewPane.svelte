<script lang="ts">
  import { onMount } from 'svelte';
  import { Download, Eye, FileText, Image as ImageIcon, ArrowLeft, Database } from 'lucide-svelte';
  import { fileStore, filePathStore } from '../../stores/fileStore';
  import { layoutStore } from '../../stores/layoutStore';
  import Hdf5Viewer from './Hdf5Viewer.svelte';

  export let id: string;
  let filePath: string | null = null;

  let loading = false;
  let error: string | null = null;
  let content: string | null = null;
  let contentType: string | null = null;
  let fileUrl: string | null = null;
  let hdf5Data: any = null;

  // React to file changes from the file store
  $: if ($fileStore && $fileStore.paneId === id) {
    filePath = $fileStore.filePath;
  }

  // Also check the filePathStore
  $: filePath = filePathStore.getFilePath(id) || null;

  $: if (filePath) {
    loadFile(filePath);
  }

  async function loadFile(path: string) {
    if (!path) return;

    loading = true;
    error = null;
    content = null;
    contentType = null;
    fileUrl = null;
    hdf5Data = null;

    try {
      const extension = path.split('.').pop()?.toLowerCase();

      // Determine file type and load accordingly
      if (['jpg', 'jpeg', 'png', 'gif', 'svg', 'webp'].includes(extension || '')) {
        // Image file
        contentType = 'image';
        fileUrl = `/api/fs/raw?path=${encodeURIComponent(path)}`;
      } else if (extension === 'pdf') {
        // PDF file
        contentType = 'pdf';
        fileUrl = `/api/fs/raw?path=${encodeURIComponent(path)}`;
      } else if (['md', 'markdown'].includes(extension || '')) {
        // Markdown file
        contentType = 'markdown';
        const response = await fetch(`/api/fs/content?path=${encodeURIComponent(path)}`);
        if (response.ok) {
          const data = await response.json();
          if (data.ok) {
            content = data.content;
          }
        }
      } else if (['h5', 'hdf5'].includes(extension || '')) {
        // HDF5 file
        contentType = 'hdf5';
        const response = await fetch(`/api/fs/hdf5/preview?path=${encodeURIComponent(path)}`);
        if (response.ok) {
            const data = await response.json();
            if (data.ok) {
                hdf5Data = data.root;
            } else {
                error = "Failed to load HDF5 data";
            }
        } else {
            error = `Server returned ${response.status}`;
        }
      } else {
        // Text file
        contentType = 'text';
        const response = await fetch(`/api/fs/content?path=${encodeURIComponent(path)}`);
        if (response.ok) {
          const data = await response.json();
          if (data.ok) {
            content = data.content;
          }
        }
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Unknown error';
      console.error(`[PreviewPane ${id}] Error loading file:`, err);
    } finally {
      loading = false;
    }
  }

  function getFileName(): string {
    if (!filePath) return 'No file selected';
    const parts = filePath.split('/');
    return parts[parts.length - 1] || filePath;
  }

  function renderMarkdown(markdown: string): string {
    let html = markdown;

    // Process code blocks first (to avoid conflicts with inline code)
    html = html.replace(/```(\w*)\n([\s\S]*?)```/g, (match, lang, code) => {
      return `<pre class="bg-gray-800 dark:bg-gray-900 text-gray-100 p-4 rounded-lg overflow-x-auto my-3 border border-gray-600 dark:border-gray-700"><code class="text-sm font-mono">${code.trim()}</code></pre>`;
    });

    // Headers
    html = html.replace(/^### (.*$)/gim, '<h3 class="text-lg font-medium mb-2">$1</h3>');
    html = html.replace(/^## (.*$)/gim, '<h2 class="text-xl font-semibold mb-3">$1</h2>');
    html = html.replace(/^# (.*$)/gim, '<h1 class="text-2xl font-bold mb-4">$1</h1>');

    // Blockquotes
    html = html.replace(/^> (.+)$/gm, '<blockquote class="border-l-4 border-gray-300 pl-4 italic my-2">$1</blockquote>');

    // Unordered lists
    html = html.replace(/^\* (.+)$/gm, '<li class="ml-4">• $1</li>');
    html = html.replace(/(<li.*>.*<\/li>)/s, '<ul class="list-disc list-inside my-2">$1</ul>');

    // Bold and italic
    html = html.replace(/\*\*(.+?)\*\*/g, '<strong class="font-semibold">$1</strong>');
    html = html.replace(/\*(.+?)\*/g, '<em class="italic">$1</em>');

    // Inline code (after processing code blocks to avoid conflicts)
    html = html.replace(/`([^`]+)`/g, '<code class="bg-gray-700 dark:bg-gray-600 text-gray-100 px-1 py-0.5 rounded text-sm font-mono">$1</code>');

    // Links
    html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank" class="text-blue-600 hover:text-blue-800 underline">$1</a>');

    // Line breaks and paragraphs
    html = html.replace(/\n\n+/g, '</p><p class="mb-3">');
    html = html.replace(/\n/g, '<br>');

    // Wrap in paragraphs (but don't wrap elements that are already in HTML tags)
    const lines = html.split('\n');
    let result = '';
    let inParagraph = false;

    for (let line of lines) {
      line = line.trim();
      if (line.startsWith('<h') || line.startsWith('<ul') || line.startsWith('<pre') || line.startsWith('<blockquote')) {
        if (inParagraph) {
          result += '</p>';
          inParagraph = false;
        }
        result += line;
      } else if (line && !line.startsWith('<')) {
        if (!inParagraph) {
          result += '<p class="mb-3">';
          inParagraph = true;
        }
        result += line;
      } else if (line === '') {
        if (inParagraph) {
          result += '</p>';
          inParagraph = false;
        }
      } else {
        result += line;
      }
    }

    if (inParagraph) {
      result += '</p>';
    }

    return result;
  }

  function downloadFile() {
    if (fileUrl) {
      const link = document.createElement('a');
      link.href = fileUrl;
      link.download = getFileName();
      link.target = '_blank';
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    }
  }

  function backToFileExplorer() {
    // Clear file path and change back to file explorer
    filePathStore.clearFilePath(id);
    layoutStore.changePaneType(id, 'file-explorer');
  }
</script>

<div class="flex flex-col h-full bg-primary-bg">
  <!-- Header -->
  <div class="flex items-center justify-between px-3 py-2 border-b border-border-color bg-tertiary-bg">
    <div class="flex items-center space-x-2">
      <button
        class="p-1 hover:bg-secondary-bg rounded transition-colors"
        on:click={backToFileExplorer}
        title="Back to File Explorer"
      >
        <ArrowLeft size={14} />
      </button>
      {#if contentType === 'image'}
        <ImageIcon size={16} class="text-blue-500" />
      {:else if contentType === 'pdf'}
        <FileText size={16} class="text-red-500" />
      {:else if contentType === 'hdf5'}
        <Database size={16} class="text-green-500" />
      {:else}
        <Eye size={16} class="text-green-500" />
      {/if}
      <span class="text-sm font-medium text-primary-text truncate max-w-[200px]">
        {getFileName()}
      </span>
    </div>
    {#if fileUrl}
      <button
        class="p-1 hover:bg-secondary-bg rounded transition-colors"
        on:click={downloadFile}
        title="Download file"
      >
        <Download size={14} />
      </button>
    {/if}
  </div>

  <!-- Content Area -->
  <div class="flex-1 overflow-auto p-4">
    {#if loading}
      <div class="flex items-center justify-center h-full">
        <div class="text-secondary-text">Loading file...</div>
      </div>
    {:else if error}
      <div class="flex flex-col items-center justify-center h-full">
        <div class="text-red-400 mb-2">Error loading file</div>
        <div class="text-sm text-secondary-text text-center">{error}</div>
      </div>
    {:else if !filePath}
      <div class="flex items-center justify-center h-full">
        <div class="text-secondary-text text-center">
          <Eye size={48} class="mx-auto mb-4 opacity-50" />
          <div>No file selected</div>
          <div class="text-sm opacity-60 mt-1">Select a file to preview</div>
        </div>
      </div>
    {:else if contentType === 'image' && fileUrl}
      <div class="flex items-center justify-center h-full">
        <img
          src={fileUrl}
          alt={getFileName()}
          class="max-w-full max-h-full object-contain"
        />
      </div>
    {:else if contentType === 'pdf' && fileUrl}
      <div class="h-full">
        <iframe
          src={fileUrl}
          class="w-full h-full border-0"
          title={getFileName()}
        ></iframe>
      </div>
    {:else if contentType === 'markdown' && content}
      <div class="h-full p-4 overflow-auto">
        <div class="prose prose-base max-w-none text-primary-text">
          {@html renderMarkdown(content)}
        </div>
      </div>
    {:else if contentType === 'hdf5' && hdf5Data}
      <div class="h-full overflow-auto p-2">
        <div class="mb-4 pb-2 border-b border-border-color">
            <h3 class="text-lg font-semibold">HDF5 Structure</h3>
            <p class="text-xs text-secondary-text">Hierarchy of groups and datasets</p>
        </div>
        <Hdf5Viewer node={hdf5Data} />
      </div>
    {:else if contentType === 'text' && content}
      <div class="h-full p-4 overflow-auto">
        <pre class="text-base text-gray-100 bg-gray-900 font-mono whitespace-pre-wrap leading-relaxed p-4 rounded-lg border border-gray-700">{content}</pre>
      </div>
    {:else}
      <div class="flex items-center justify-center h-full">
        <div class="text-secondary-text">Cannot preview this file type</div>
      </div>
    {/if}
  </div>
</div>
