<script lang="ts">
  import { onMount } from 'svelte';
  import { layoutStore } from './stores/layoutStore';
  import { settingsStore } from './stores/settingsStore';
  import { authStore } from './stores/authStore';
  import LayoutRenderer from './components/layout/LayoutRenderer.svelte';
  import SettingsModal from './components/settings/SettingsModal.svelte';
  import LayoutsModal from './components/layout/LayoutsModal.svelte';
  import LoginModal from './components/auth/LoginModal.svelte';
  import KeyboardManager from './components/KeyboardManager.svelte';
  import { Settings, LayoutTemplate } from 'lucide-svelte';

  let isSettingsOpen = false;
  let isLayoutsModalOpen = false;
  let isLoginModalOpen = false;
  // registerAction removed as we use keyboardManager instance directly
  let keyboardManager: KeyboardManager;

  // Check authentication status on app startup
  onMount(async () => {
    await authStore.checkAuth();

    // Show login modal if not authenticated
    if (!$authStore.isAuthenticated) {
      isLoginModalOpen = true;
    }

    // Setup keyboard shortcuts after authentication check
    // We rely on reactive statement for keyboardManager availability
  });

  // Function to get the currently focused pane ID
  function getFocusedPaneId(): string | null {
    const focusedElement = document.activeElement;
    const paneElement = focusedElement?.closest('[data-pane-id]');
    return paneElement?.getAttribute('data-pane-id') || null;
  }

  // Register keyboard shortcut actions
  function setupKeyboardShortcuts() {
    if (keyboardManager) {
      keyboardManager.registerAction('open-settings', () => {
        console.log('Opening settings via shortcut');
        isSettingsOpen = true;
      });

      keyboardManager.registerAction('split-horizontal', () => {
        console.log('Splitting pane horizontally');
        const paneId = getFocusedPaneId();
        if (paneId) {
          layoutStore.splitPane(paneId, 'horizontal');
        }
      });

      keyboardManager.registerAction('split-vertical', () => {
        console.log('Splitting pane vertically');
        const paneId = getFocusedPaneId();
        if (paneId) {
          layoutStore.splitPane(paneId, 'vertical');
        }
      });

      keyboardManager.registerAction('close-pane', () => {
        console.log('Closing pane');
        const paneId = getFocusedPaneId();
        if (paneId) {
          layoutStore.closePane(paneId);
        }
      });

      keyboardManager.registerAction('toggle-file-explorer', () => {
        console.log('Toggling file explorer');
        // Create a file explorer pane if none exists, or focus the first one
        const rootId = $layoutStore?.id;
        if (rootId) {
          layoutStore.splitPane(rootId, 'horizontal');
          // TODO: Find the new pane ID and change it to file-explorer type
        }
      });
    }
  }

  // Call setup when component mounts and keyboardManager is ready
  $: if (keyboardManager) {
    setupKeyboardShortcuts();
  }

  // Handle successful login
  function handleLoginSuccess() {
    isLoginModalOpen = false;
    // Optionally refresh layout or perform other post-login actions
  }

  // Handle login cancellation (optional - could lock app)
  function handleLoginCancel() {
    // For now, we'll just keep the modal open
    // In production, you might want to show a message or redirect
    isLoginModalOpen = true;
  }
</script>

<main 
  class="flex flex-col h-screen w-screen font-sans bg-primary-bg text-primary-text theme-{$settingsStore.theme}"
>
  <!-- 顶部工具栏 -->
  <header class="h-10 flex items-center justify-between px-4 bg-tertiary-bg border-b border-border-color">
    <div class="flex items-center gap-2">
      <div class="font-bold text-lg tracking-tight text-accent-color">LiteTerm</div>
      <div class="text-xs bg-secondary-bg px-2 py-0.5 rounded text-secondary-text">server: connected</div>
    </div>
    
    <div class="flex items-center gap-3">
      <button class="flex items-center gap-1 text-sm hover:text-primary-text text-secondary-text" on:click={() => isLayoutsModalOpen = true}>
        <LayoutTemplate size={16} />
        <span>Layouts</span>
      </button>
      <button class="text-secondary-text hover:text-primary-text" on:click={() => isSettingsOpen = true}>
        <Settings size={18} />
      </button>
    </div>
  </header>

  <!-- 布局主区域 -->
  <div class="flex-1 overflow-hidden relative">
    {#if $authStore.isAuthenticated}
      {#if $layoutStore}
        <LayoutRenderer node={$layoutStore} />
      {:else}
        <div class="flex items-center justify-center h-full">
          <div class="text-center">
            <p class="text-secondary-text mb-2">Loading Layout...</p>
            <p class="text-xs text-secondary-text/60">Debug: Auth={$authStore.isAuthenticated}, Layout={$layoutStore ? 'exists' : 'null'}</p>
          </div>
        </div>
      {/if}
    {:else}
      <div class="flex items-center justify-center h-full">
        <div class="text-center">
          <h1 class="text-2xl font-bold mb-4 text-primary-text">LiteTerm</h1>
          <p class="text-secondary-text">Authentication required to access the terminal</p>
          <p class="text-xs text-secondary-text/60 mt-2">Debug: Auth={$authStore.isAuthenticated}</p>
        </div>
      </div>
    {/if}
  </div>
  
  <!-- 移动端虚拟按键栏 (示例) -->
  <!-- <div class="h-12 bg-gray-900 border-t border-gray-700 flex items-center justify-around px-2 md:hidden">
    <button>ESC</button>
    <button>TAB</button>
    ...
  </div> -->

  <SettingsModal isOpen={isSettingsOpen} onClose={() => isSettingsOpen = false} />
  <LayoutsModal isOpen={isLayoutsModalOpen} onClose={() => isLayoutsModalOpen = false} />

  <!-- Login Modal for authentication -->
  <LoginModal
    isOpen={isLoginModalOpen}
    onSuccess={handleLoginSuccess}
    onCancel={handleLoginCancel}
  />

  <!-- Keyboard Manager for global shortcuts -->
  <KeyboardManager bind:this={keyboardManager} />
</main>