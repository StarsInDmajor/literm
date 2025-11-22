<script lang="ts">
  import { onMount } from 'svelte';
  import type { LayoutNode, ContentType } from '../../lib/types';
  import { layoutStore, layoutViewStore } from '../../stores/layoutStore';
  import { X, SplitSquareHorizontal, SplitSquareVertical, Maximize2, Minimize2, RefreshCw, Settings, ChevronDown } from 'lucide-svelte';

  // 具体的内容组件
  import TerminalPane from '../panes/TerminalPane.svelte';
  import FileExplorerPane from '../panes/FileExplorerPane.svelte';
  import PreviewPane from '../panes/PreviewPane.svelte';

  export let node: LayoutNode;

  $: isMaximized = $layoutViewStore.maximizedPaneId === node.id;

  let terminalPaneRef: TerminalPane; // Reference to the TerminalPane component
  let paneContainer: HTMLDivElement; // Reference to this pane's root div
  let showTypeDropdown = false; // Toggle for dropdown visibility

  // Available pane types with labels
  const paneTypes: { value: ContentType; label: string; icon: string }[] = [
    { value: 'terminal', label: 'Terminal', icon: '⏻' },
    { value: 'file-explorer', label: 'File Explorer', icon: '📁' },
    { value: 'preview', label: 'Preview', icon: '👁️' },
    { value: 'empty', label: 'Empty', icon: '⭕' }
  ];

  onMount(() => {
    console.log(`[Pane ${node.id}] Mounted. Initial dimensions: ${paneContainer.offsetWidth}x${paneContainer.offsetHeight}`);
  });

  function handleSplit(direction: 'horizontal' | 'vertical') {
    console.log(`[Pane ${node.id}] Splitting pane, direction: ${direction}`);
    layoutStore.splitPane(node.id, direction);
    console.log(`[Pane ${node.id}] Split completed`);
  }

  function handleClose() {
    console.log(`[Pane ${node.id}] Closing pane`);
    layoutStore.closePane(node.id);
    console.log(`[Pane ${node.id}] Close completed`);
  }

  function toggleMaximize() {
    console.log(`[Pane ${node.id}] Toggling maximize`);
    layoutViewStore.toggleMaximize(node.id);
    console.log(`[Pane ${node.id}] Maximize toggle completed`);
  }

  function handleRefresh() {
    console.log(`[Pane ${node.id}] Refresh clicked (placeholder)`);
    // TODO: Implement terminal refresh if needed
  }

  function handleTypeChange(newType: ContentType) {
    layoutStore.changePaneType(node.id, newType);
    showTypeDropdown = false; // Close dropdown after selection
  }

  function toggleDropdown() {
    showTypeDropdown = !showTypeDropdown;
  }

  // Get the current pane type info
  $: currentPaneType = paneTypes.find(pt => pt.value === node.contentType) || paneTypes[0];
</script>

<div 
  class="flex flex-col border border-border-color bg-primary-bg overflow-hidden transition-all duration-200"
  class:w-full={!isMaximized}
  class:h-full={!isMaximized}
  class:fixed={isMaximized}
  class:inset-0={isMaximized}
  class:z-[100]={isMaximized}
  bind:this={paneContainer}
  data-pane-id={node.id}
>
  <!-- 1. 标题栏 -->
  <div class="flex items-center justify-between bg-tertiary-bg h-8 px-2 border-b border-border-color select-none">
    <div class="flex items-center gap-2 text-xs text-secondary-text">
      <!-- Pane Type Selector Dropdown -->
      <div class="relative">
        <button
          class="flex items-center gap-1 px-2 py-0.5 hover:bg-secondary-bg rounded transition-colors"
          on:click={toggleDropdown}
          title="Change Pane Type"
        >
          <span class="text-sm">{currentPaneType.icon}</span>
          <span class="font-medium">{currentPaneType.label}</span>
          <ChevronDown size={12} class="transition-transform {showTypeDropdown ? 'rotate-180' : ''}" />
        </button>

        <!-- Dropdown Menu -->
        {#if showTypeDropdown}
          <div class="absolute top-full left-0 mt-1 bg-tertiary-bg border border-border-color rounded shadow-lg z-50 min-w-[140px]">
            {#each paneTypes as paneType}
              <button
                class="w-full text-left px-3 py-1.5 text-xs hover:bg-secondary-bg flex items-center gap-2 transition-colors
                       {node.contentType === paneType.value ? 'bg-secondary-bg text-primary-text font-medium' : 'text-secondary-text'}"
                on:click={() => handleTypeChange(paneType.value)}
              >
                <span>{paneType.icon}</span>
                <span>{paneType.label}</span>
                {#if node.contentType === paneType.value}
                  <span class="ml-auto text-primary-text">✓</span>
                {/if}
              </button>
            {/each}
          </div>
        {/if}
      </div>

      <span class="text-secondary-text/60">|</span>
      <span class="text-secondary-text/70">{node.config?.title || 'Untitled'}</span>
    </div>

    <!-- 窗口控制按钮 -->
    <div class="flex items-center gap-1">
      {#if node.contentType === 'terminal'}
        <button class="p-1 hover:bg-secondary-bg rounded" on:click={handleRefresh} title="Refresh Terminal Size">
          <RefreshCw size={14} />
        </button>
      {/if}
      <button class="p-1 hover:bg-secondary-bg rounded" on:click={() => handleSplit('horizontal')} title="Split Horizontal">
        <SplitSquareHorizontal size={14} />
      </button>
      <button class="p-1 hover:bg-secondary-bg rounded" on:click={() => handleSplit('vertical')} title="Split Vertical">
        <SplitSquareVertical size={14} />
      </button>
      <button class="p-1 hover:bg-secondary-bg rounded" on:click={toggleMaximize} title={isMaximized ? "Restore" : "Maximize"}>
        {#if isMaximized}
          <Minimize2 size={14} />
        {:else}
          <Maximize2 size={14} />
        {/if}
      </button>
      <button class="p-1 hover:bg-red-900 text-red-400 rounded" on:click={handleClose} title="Close">
        <X size={14} />
      </button>
    </div>
  </div>

  <!-- Click outside to close dropdown -->
  {#if showTypeDropdown}
    <button
      class="fixed inset-0 z-40 bg-transparent border-none"
      on:click={() => showTypeDropdown = false}
      aria-label="Close dropdown"
      type="button"
    ></button>
  {/if}

  <!-- 2. 内容区域 -->
  <div class="flex-1 min-h-0 relative">
    {#if node.contentType === 'terminal'}
      <TerminalPane
        id={node.id}
        bind:this={terminalPaneRef}
      />
    {:else if node.contentType === 'file-explorer'}
      <FileExplorerPane id={node.id} />
    {:else if node.contentType === 'preview'}
      <PreviewPane id={node.id} />
    {:else if node.contentType === 'empty'}
      <div class="flex items-center justify-center h-full text-secondary-text text-sm">
        <div class="flex flex-col items-center">
          <div class="text-4xl mb-2 opacity-30">⭕</div>
          <div class="text-xs text-secondary-text/60">Empty Pane</div>
        </div>
      </div>
    {:else}
      <div class="p-4 text-red-400">Unknown content type: {node.contentType}</div>
    {/if}
  </div>
</div>