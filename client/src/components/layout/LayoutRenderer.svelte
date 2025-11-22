<script lang="ts">
  import type { LayoutNode } from '../../lib/types';
  import Pane from './Pane.svelte';
  // 引用自身实现递归
  import LayoutRenderer from './LayoutRenderer.svelte';
  import { layoutViewStore } from '../../stores/layoutStore';

  export let node: LayoutNode;

  let effectiveNodeToRender: LayoutNode;

  // Recursive function to find a node by ID
  function findNodeById(root: LayoutNode, id: string): LayoutNode | undefined {
    if (root.id === id) {
      return root;
    }
    if (root.children) {
      for (const child of root.children) {
        const found = findNodeById(child, id);
        if (found) {
          return found;
        }
      }
    }
    return undefined;
  }

  $: {
    const maximizedId = $layoutViewStore.maximizedPaneId;
    if (maximizedId) {
      const foundNode = findNodeById(node, maximizedId);
      effectiveNodeToRender = foundNode || node; // Fallback to original node if not found
    } else {
      effectiveNodeToRender = node;
    }
  }
</script>

{#if effectiveNodeToRender.type === 'container'}
  <!-- 容器节点：使用 Flexbox 布局 -->
  <div 
    class="flex w-full h-full overflow-hidden"
    class:flex-row={effectiveNodeToRender.direction === 'horizontal'}
    class:flex-col={effectiveNodeToRender.direction === 'vertical'}
  >
    {#each effectiveNodeToRender.children || [] as child (child.id)}
      <!-- 
        简单的平分策略。
        进阶：可以给 node 添加 flex-basis 或 flex-grow 属性来实现可拖拽调整大小。
      -->
      <div class="flex-1 min-w-0 min-h-0 relative border-r border-b border-gray-800 last:border-0">
        <LayoutRenderer node={child} />
      </div>
    {/each}
  </div>
{:else}
  <!-- 叶子节点：渲染具体 Pane -->
  <Pane node={effectiveNodeToRender} />
{/if}