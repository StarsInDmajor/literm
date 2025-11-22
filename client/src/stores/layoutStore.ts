import { writable } from 'svelte/store';
import type { LayoutNode, Direction, ContentType } from '../lib/types';
import { v4 as uuidv4 } from 'uuid';

// 初始状态：一个全屏的空 Pane
const initialLayout: LayoutNode = {
  id: uuidv4(),
  type: 'pane',
  contentType: 'terminal', // 默认显示终端
  config: { title: 'Terminal' }
};

function createLayoutStore() {
  const { subscribe, update, set } = writable<LayoutNode>(initialLayout);

  // 辅助函数：在树中递归查找节点并执行操作
  const updateNodeInTree = (root: LayoutNode, targetId: string, callback: (node: LayoutNode) => LayoutNode): LayoutNode => {
    if (root.id === targetId) {
      return callback(root);
    }
    if (root.children) {
      return {
        ...root,
        children: root.children.map(child => updateNodeInTree(child, targetId, callback))
      };
    }
    return root;
  };

  // 辅助函数：查找节点的父节点
  const findParentAndIndex = (root: LayoutNode, targetId: string, parent: LayoutNode | null = null): { parent: LayoutNode | null; index: number } | null => {
    if (root.children) {
      for (let i = 0; i < root.children.length; i++) {
        const child = root.children[i];
        if (child.id === targetId) {
          return { parent: root, index: i };
        }
        const result = findParentAndIndex(child, targetId, root);
        if (result) {
          return result;
        }
      }
    }
    return null;
  };

  // 辅助函数：关闭Pane并重新填充布局
  const closePaneAndRefill = (root: LayoutNode, targetId: string): LayoutNode => {
    // 如果根节点就是要关闭的Pane
    if (root.id === targetId) {
      // 创建一个新的Terminal Pane作为替代
      return {
        id: uuidv4(),
        type: 'pane',
        contentType: 'terminal',
        config: { title: 'Terminal' }
      };
    }

    // 查找要关闭节点的父节点
    const parentInfo = findParentAndIndex(root, targetId);
    if (!parentInfo || !parentInfo.parent) {
      // 如果没找到父节点，fallback到简单替换
      return updateNodeInTree(root, targetId, () => ({
        id: uuidv4(),
        type: 'pane',
        contentType: 'terminal',
        config: { title: 'Terminal' }
      }));
    }

    const { parent, index } = parentInfo;

    // 从父节点的children中移除要关闭的节点
    const newChildren = parent.children!.filter((_, i) => i !== index);

    if (newChildren.length === 0) {
      // 如果没有其他子节点，父容器应该变成一个Pane
      return updateNodeInTree(root, parent.id, () => ({
        id: uuidv4(),
        type: 'pane',
        contentType: 'terminal',
        config: { title: 'Terminal' }
      }));
    } else if (newChildren.length === 1) {
      // 如果只剩下一个子节点，提升这个子节点
      const onlyChild = newChildren[0];
      return updateNodeInTree(root, parent.id, () => onlyChild);
    } else {
      // 多个子节点，保持容器不变
      return updateNodeInTree(root, parent.id, () => ({
        ...parent,
        children: newChildren
      }));
    }
  };

  return {
    subscribe,

    // 核心功能：拆分 Pane
    splitPane: (targetId: string, direction: Direction) => {
      console.log(`[LayoutStore] Splitting pane ${targetId} in direction ${direction}`);
      update(root => {
        return updateNodeInTree(root, targetId, (node) => {
          // 只有 Pane 才能被拆分
          if (node.type !== 'pane') {
            console.log(`[LayoutStore] Node ${targetId} is not a pane, it's a ${node.type}`);
            return node;
          }

          const newContainerId = uuidv4();
          const newPaneId = uuidv4();
          console.log(`[LayoutStore] Creating new container ${newContainerId} with new pane ${newPaneId}`);

          // 创建一个新的容器，包含原有的 Pane 和一个新的 Terminal Pane
          const newContainer: LayoutNode = {
            id: newContainerId,
            type: 'container',
            direction: direction,
            children: [
              { ...node }, // 保留原有 Pane 状态
              {
                id: newPaneId,
                type: 'pane',
                contentType: 'terminal', // New pane defaults to terminal
                config: {
                  title: 'Terminal'
                }
              }
            ]
          };
          console.log(`[LayoutStore] Split complete, returning new container with ${newContainer.children?.length} children`);
          return newContainer;
        });
      });
    },

    // 关闭 Pane 并重新填充布局
    closePane: (targetId: string) => {
      update(root => {
        console.log(`[LayoutStore] Closing pane: ${targetId}`);
        const newRoot = closePaneAndRefill(root, targetId);
        console.log('[LayoutStore] New layout after close:', newRoot);
        return newRoot;
      });
    },

    // 改变 Pane 类型
    changePaneType: (targetId: string, newContentType: ContentType) => {
      update(root => {
        return updateNodeInTree(root, targetId, (node) => {
          // 只有 Pane 才能改变类型
          if (node.type !== 'pane') return node;

          const titleMap = {
            'terminal': 'Terminal',
            'file-explorer': 'File Explorer',
            'preview': 'Preview',
            'empty': 'Empty'
          };

          return {
            ...node,
            contentType: newContentType,
            config: {
              ...node.config,
              title: titleMap[newContentType]
            }
          };
        });
      });
    },

    // 设置整个布局（用于模板）
    setLayout: (newRoot: LayoutNode) => {
      set(newRoot);
    }
  };
}

export const layoutStore = createLayoutStore();

// View state separate from the tree structure
interface LayoutViewState {
  maximizedPaneId: string | null;
}

function createLayoutViewStore() {
  const { subscribe, update } = writable<LayoutViewState>({ maximizedPaneId: null });

  return {
    subscribe,
    toggleMaximize: (id: string) => {
      update(state => ({
        maximizedPaneId: state.maximizedPaneId === id ? null : id
      }));
    }
  };
}

export const layoutViewStore = createLayoutViewStore();