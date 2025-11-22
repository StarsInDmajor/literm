export type NodeType = 'container' | 'pane';
export type Direction = 'horizontal' | 'vertical';
export type ContentType = 'terminal' | 'file-explorer' | 'preview' | 'empty';

export interface PaneContentConfig {
  title?: string;
  filePath?: string;
  terminalId?: string;
  // Session sharing data
  sharedTerminal?: any;
  sharedSocket?: any;
  isSharedSession?: boolean;
}

export interface LayoutNode {
  id: string;
  type: NodeType;
  parent?: string | null; // 方便向上查找

  // Container 属性
  direction?: Direction;
  children?: LayoutNode[];
  
  // Pane 属性
  contentType?: ContentType;
  config?: PaneContentConfig;
}