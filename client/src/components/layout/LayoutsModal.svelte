<script lang="ts">
  import { layoutStore } from '../../stores/layoutStore';
  import { templates } from '../../lib/templates';
  import { X } from 'lucide-svelte';
  import { fade, scale } from 'svelte/transition';
  import type { LayoutNode } from '../../lib/types';

  export let isOpen = false;
  export let onClose: () => void;

  function applyTemplate(createFn: () => LayoutNode) {
    layoutStore.setLayout(createFn());
    onClose();
  }

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      onClose();
    }
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
        <h2 class="text-lg font-semibold text-primary-text">Choose Layout</h2>
        <button class="text-secondary-text hover:text-primary-text transition-colors" on:click={onClose}>
          <X size={20} />
        </button>
      </div>

      <!-- Body -->
      <div class="p-6 grid grid-cols-1 md:grid-cols-2 gap-4 max-h-[70vh] overflow-y-auto">
        {#each templates as template}
          <button 
            type="button"
            class="flex flex-col items-start p-4 border border-border-color rounded-lg 
                   bg-primary-bg hover:bg-tertiary-bg transition-colors duration-200 text-left group"
            on:click={() => applyTemplate(template.create)}
          >
            <h3 class="text-md font-semibold text-accent-color group-hover:text-accent-color">{template.name}</h3>
            <p class="text-sm text-secondary-text mt-1">{template.description}</p>
          </button>
        {/each}
      </div>

      <!-- Footer -->
      <div class="px-6 py-4 bg-tertiary-bg flex justify-end">
        <button 
          class="px-4 py-2 bg-secondary-bg hover:bg-tertiary-bg text-primary-text text-sm font-medium rounded transition-colors"
          on:click={onClose}
        >
          Close
        </button>
      </div>
    </div>
  </div>
{/if}
