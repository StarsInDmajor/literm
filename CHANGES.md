# Change Log

## Pane Type Selector

### Overview
Added a dropdown button to each pane allowing users to change the pane type dynamically.

### Files Modified

#### 1. `/client/src/stores/layoutStore.ts`
- **Added**: `changePaneType(targetId: string, newContentType: ContentType)` function
- **Functionality**: Updates the content type of a pane and automatically updates the title

#### 2. `/client/src/components/layout/Pane.svelte`
- **Added**: Dropdown component in the title bar for selecting pane types
- **Features**:
  - Visual dropdown with icons and labels for each pane type
  - Click outside to close functionality
  - Smooth transitions and hover effects
  - Checkmark indicator for current selection
  - Accessible button elements with proper ARIA labels

- **Pane Types Available**:
  - `terminal` ⏻ - Terminal pane with xterm.js
  - `file-explorer` 📁 - File browser (placeholder)
  - `preview` 👁️ - Preview component for PDF/images (placeholder)
  - `empty` ⭕ - Empty pane

- **UI Improvements**:
  - Separated pane type selector from title with a separator
  - Better visual hierarchy in title bar
  - Improved placeholder content for each pane type

#### 3. `/client/src/components/layout/LayoutsModal.svelte`
- **Fixed**: Added missing import for `LayoutNode` type

### Features

#### Pane Type Selector
- Located in the left side of the pane title bar
- Displays current pane type with icon and label
- Click to open dropdown menu
- Shows all available pane types with icons
- Checkmark indicates current selection
- Click outside or select an option to close

#### Pane Type Switching
- Instant switching between pane types
- Maintains pane ID and other configurations
- Updates title automatically based on pane type
- Clean placeholder content for unimplemented types

### Usage

1. Each pane has a dropdown button in its title bar showing the current type
2. Click the dropdown to see available pane types
3. Click on any type to switch the pane
4. The pane content updates immediately

---

## Smart Pane Closing

### Overview
Close button now removes panes and refills layout automatically instead of showing empty panes.

### Features
- **Removes Pane**: Clicking close (✕) removes the pane from layout
- **Layout Refill**: Other panes expand to fill the space
- **No Empty Panes**: Eliminates wasted screen space
- **Tree Optimization**: Simplifies layout tree recursively

### How It Works
1. **Close Button Clicked**: Pane removed from layout tree
2. **Parent Container**: System checks parent container state
3. **Refill Logic**:
   - If no siblings remain → parent becomes a Terminal pane
   - If one sibling → sibling takes parent's place
   - If multiple siblings → container stays with remaining children

### Files Modified
- `client/src/stores/layoutStore.ts`: Added `closePaneAndRefill()` function and helper `findParentAndIndex()`

---

## Terminal Improvements

### Log Preservation Fix
**Problem**: Splitting panes caused terminal log loss due to new WebSocket connections being created.
**Solution**: Disabled ResizeObserver in `TerminalPane.svelte` to prevent terminal refresh during split operations.

**Key Changes**:
- **TerminalPane.svelte**: Disabled handle resize during split
- **layoutStore.ts**: New panes default to terminal type

**Result**:
- Original terminal maintains its WebSocket connection during split
- New pane creates a fresh terminal session
- No resize events trigger terminal refresh

### Resize Optimization
**Problem**: Pane operations (split, close, maximize) were triggering unnecessary terminal resize events, causing flickering and backend load.
**Solution**: Disabled ResizeObserver during pane operations.

**Features**:
- **Manual Resize Mode**: Flag to control resize events
- **Operation Detection**: Split/close/maximize trigger manual mode
- **Selective Resizing**: Only manual refresh button sends resize
- **Auto-restore**: ResizeObserver re-enabled after operations

**Files Modified**:
- `client/src/components/panes/TerminalPane.svelte`: Added manual resize mode
- `client/src/components/layout/Pane.svelte`: Operation coordination

---

## Build Status
- ✅ TypeScript compilation: Passed
- ✅ Svelte check: Passed
- ✅ Production build: Successful