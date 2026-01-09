# Plugin Architecture Plan: Provider vs Controller Patterns

## Problem Analysis

The current plugin system has inconsistent UX because plugins mix two fundamentally different patterns:

### Current State

| Plugin | Pattern Used | UI Owner | Result |
|--------|-------------|----------|--------|
| Live Grep | **Provider** (via prompt) | Editor | Arrow keys work, consistent UX |
| Git Grep | **Provider** (via prompt) | Editor | Arrow keys work, consistent UX |
| Git Find File | **Provider** (via prompt) | Editor | Arrow keys work, consistent UX |
| Find References | **Controller** (virtual buffer) | Plugin | Keybindings broken, custom UI |
| Search Replace | **Controller** (virtual buffer) | Plugin | Keybindings broken, custom UI |
| Diagnostics Panel | **Controller** (virtual buffer) | Plugin | Works but reimplements everything |

### Root Cause

When plugins try to **own the UI** (Controller pattern) via virtual buffers:
- They must reimplement navigation (arrows, Enter, Escape)
- They must reimplement selection highlighting
- They must define custom modes with keybindings
- The `editor.defineMode()` keybindings are fragile/broken
- Each plugin looks and behaves differently

When plugins **provide data** and let the editor render (Provider pattern):
- Navigation is automatic and consistent
- Selection highlighting is automatic
- Keybindings work reliably
- All plugins look and behave the same

---

## Proposed Architecture

### 1. Standardize on Provider Pattern Where Possible

**Principle:** If a feature just shows a list of results that users navigate and select, use the Provider pattern.

#### A. Results Panel Provider (for persistent panels)

For features like Find References, Diagnostics that show results in a split panel:

```typescript
// Plugin provides data, editor handles UI
editor.showResultsPanel({
  title: "References to 'Args'",
  items: [
    { label: "src/main.rs:33:8", description: "struct Args {", location: {...} },
    { label: "src/main.rs:235:12", description: "args: &Args,", location: {...} },
  ],
  onSelect: (item) => {
    // Called when user presses Enter
    editor.openFile(item.location.file, item.location.line, item.location.column);
  },
  onClose: () => {
    // Called when user presses Escape
  }
});
```

**Editor responsibilities:**
- Render the panel with consistent styling
- Handle Up/Down navigation
- Handle Enter (call `onSelect`)
- Handle Escape (call `onClose`)
- Highlight current selection
- Show item count in status

**Plugin responsibilities:**
- Provide the items list
- Handle the `onSelect` callback
- Optionally update items dynamically

#### B. QuickPick Controller (for transient pickers)

For features like Live Grep, Git Find File that need real-time filtering:

```typescript
// This is what startPrompt + setPromptSuggestions already does
// But formalize it as the "QuickPick" pattern
const picker = editor.createQuickPick({
  placeholder: "Search in files...",
  onDidChangeValue: (query) => {
    // User typed something - update results
    const results = performSearch(query);
    picker.items = results;
  },
  onDidAccept: (item) => {
    // User pressed Enter
    editor.openFile(item.file, item.line, item.column);
    picker.dispose();
  },
  onDidHide: () => {
    // User pressed Escape
    picker.dispose();
  }
});
picker.show();
```

**This is essentially what the prompt system already does** - just needs better documentation and consistency.

---

### 2. Simplify Current Plugins

#### Find References → Use Results Panel Provider

Instead of creating a virtual buffer with custom mode, use a standardized panel:

```typescript
// OLD (Controller - broken)
editor.defineMode("references-list", null, [...]);
editor.createVirtualBufferInSplit({...});
// Plugin manually handles everything

// NEW (Provider - works)
editor.showResultsPanel({
  id: "references",
  title: `References to '${symbol}'`,
  items: references.map(ref => ({
    label: `${ref.file}:${ref.line}:${ref.column}`,
    description: ref.lineText,
    data: ref
  })),
  onSelect: (item) => editor.openFile(item.data.file, item.data.line, item.data.column)
});
```

#### Search Replace → Hybrid Approach

Search Replace needs checkboxes (multi-select), which is more complex:

```typescript
editor.showResultsPanel({
  id: "search-replace",
  title: "Search & Replace",
  multiSelect: true,  // Enable checkboxes
  items: results.map(r => ({ ...r, selected: true })),
  actions: [
    { key: "Enter", label: "Replace Selected", action: executeReplace },
    { key: "Space", label: "Toggle", action: toggleCurrent },
  ],
  onSelect: (item) => previewItem(item)
});
```

#### Live Grep / Git Grep / Git Find File → Already Provider Pattern

These already use `startPrompt()` + `setPromptSuggestions()` which IS the Provider pattern. Just need to fix:
1. Preview not updating → Fix `prompt_selection_changed` handler
2. Cursor position on open → Fix the `openFile` call parameters

---

### 3. Implementation Phases

#### Phase 1: Fix Existing Plugins (Short-term)

Without changing the API, fix the immediate issues:

1. **Live Grep**: Fix `prompt_selection_changed` to update preview
2. **Git Grep**: Fix cursor positioning when opening file
3. **Find References**: Change parent mode from `null` to `"normal"` (already done)
4. **Search Replace**: Simplify keybindings, fix mode inheritance

#### Phase 2: Introduce Results Panel API (Medium-term)

Add a new editor API that plugins can use instead of raw virtual buffers:

```typescript
interface ResultsPanelOptions {
  id: string;
  title: string;
  items: ResultItem[];
  multiSelect?: boolean;
  onSelect?: (item: ResultItem) => void;
  onClose?: () => void;
}

editor.showResultsPanel(options: ResultsPanelOptions): ResultsPanel;
editor.hideResultsPanel(id: string): void;
```

This API would:
- Create the virtual buffer internally
- Handle all navigation and selection
- Apply consistent styling
- Call plugin callbacks for actions

#### Phase 3: Migrate Plugins (Long-term)

Migrate existing plugins to use the new API:
- Find References → `showResultsPanel` ✓ (Done - with syncWithEditor, groupBy)
- Diagnostics Panel → `showResultsPanel` ✓ (Done - with Provider pattern for live updates)
- Search Replace → Consider `showResultsPanel` with `multiSelect` (already simplified keybindings)

---

### 4. Keybinding Strategy

All result panels should use the same keybindings, handled by the EDITOR:

| Key | Action | Handler |
|-----|--------|---------|
| `Up` / `Down` | Navigate items | Editor (automatic) |
| `Enter` | Activate item | Editor calls `onSelect` |
| `Escape` | Close panel | Editor calls `onClose` |
| `Space` | Toggle selection (if multiSelect) | Editor (automatic) |

**No custom modes needed.** The editor's results panel implementation handles everything.

---

### 5. Visual Consistency

The editor-owned results panel ensures:
- Same selection highlighting style
- Same layout (title, items, help footer)
- Same scrolling behavior
- Same status bar integration

---

## Summary

| Pattern | When to Use | API |
|---------|-------------|-----|
| **Provider (QuickPick)** | Transient search with filtering | `startPrompt` + `setPromptSuggestions` |
| **Provider (Results Panel)** | Persistent panel with results list | `showResultsPanel` (new API) |
| **Controller** | Only when plugin needs full UI control | `createVirtualBufferInSplit` (escape hatch) |

The key insight is: **Let the editor own the UI.** Plugins should provide data and handle callbacks, not try to reimplement navigation and selection.

---

## Known Issues (Discovered During Implementation)

### `openFileInSplit` doesn't position cursor correctly when file is already open

**Severity:** Low (workaround exists)

**Description:** When calling `editor.openFileInSplit(split, path, line, column)` and the file is already open in that split, the cursor may not move to the specified position correctly.

**Workaround:** Use `editor.focusSplit(splitId)` followed by `editor.openFile(path, line, column)` instead. This pattern works correctly and is what `ResultsPanel.openInSource()` uses.

---

## Immediate Actions (Phase 1) - ALL COMPLETE ✓

1. ~~Create ResultsPanel abstraction~~ ✓ (Done - `plugins/lib/results-panel.ts` with VS Code-style Provider pattern)
2. ~~Migrate Find References to use ResultsPanel~~ ✓ (Done - with syncWithEditor, groupBy: "file")
3. ~~Fix Live Grep preview update issue~~ ✓ (Done - added `prompt_selection_changed` hook)
4. ~~Git Grep cursor positioning~~ ✓ (Verified - code is correct, uses same pattern as Find References)
5. ~~Simplify Search Replace keybindings~~ ✓ (Done - uses Enter/Space/Escape, inherits from normal)
6. ~~Diagnostics Panel migration~~ ✓ (Done - uses Provider pattern with live updates via onDidChangeResults)
7. ~~Document the Provider pattern~~ ✓ (This document + inline code documentation)

### Key Architecture Features Implemented:
- **Provider Pattern**: ResultsProvider interface with provideResults() and onDidChangeResults event
- **Bidirectional Cursor Sync**: syncWithEditor option auto-syncs panel selection with source cursor
- **Event System**: EventEmitter<T> for typed events, Disposable for cleanup
- **Static Provider Helper**: createStaticProvider() for one-shot data like Find References
