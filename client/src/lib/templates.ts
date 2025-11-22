import type { LayoutNode } from './types';
import { v4 as uuidv4 } from 'uuid';

export const templates: { name: string; description: string; create: () => LayoutNode }[] = [
  {
    name: 'Single Terminal',
    description: 'A single full-screen terminal.',
    create: () => ({
      id: uuidv4(),
      type: 'pane',
      contentType: 'terminal',
      config: { title: 'Terminal' }
    })
  },
  {
    name: 'Split Horizontal',
    description: 'Two terminals side by side.',
    create: () => ({
      id: uuidv4(),
      type: 'container',
      direction: 'horizontal',
      children: [
        {
          id: uuidv4(),
          type: 'pane',
          contentType: 'terminal',
          config: { title: 'Terminal 1' }
        },
        {
          id: uuidv4(),
          type: 'pane',
          contentType: 'terminal',
          config: { title: 'Terminal 2' }
        }
      ]
    })
  },
  {
    name: 'Grid 2x2',
    description: 'Four terminals in a grid.',
    create: () => ({
      id: uuidv4(),
      type: 'container',
      direction: 'vertical',
      children: [
        {
          id: uuidv4(),
          type: 'container',
          direction: 'horizontal',
          children: [
            { id: uuidv4(), type: 'pane', contentType: 'terminal', config: { title: 'Term TL' } },
            { id: uuidv4(), type: 'pane', contentType: 'terminal', config: { title: 'Term TR' } }
          ]
        },
        {
          id: uuidv4(),
          type: 'container',
          direction: 'horizontal',
          children: [
            { id: uuidv4(), type: 'pane', contentType: 'terminal', config: { title: 'Term BL' } },
            { id: uuidv4(), type: 'pane', contentType: 'terminal', config: { title: 'Term BR' } }
          ]
        }
      ]
    })
  },
  {
      name: 'IDE Layout',
      description: 'Left sidebar with terminal on right.',
      create: () => ({
          id: uuidv4(),
          type: 'container',
          direction: 'horizontal',
          children: [
              {
                  id: uuidv4(),
                  type: 'pane',
                  contentType: 'file-explorer', // Placeholder
                  config: { title: 'Explorer' }
              },
              {
                  id: uuidv4(),
                  type: 'pane',
                  contentType: 'terminal',
                  config: { title: 'Terminal' }
              }
          ]
      })
  }
];
