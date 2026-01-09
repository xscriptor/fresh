# Plugin Usability Review

Review of plugin usability for: Find References, Live Grep, Git Grep, Git Find File, Search Replace in Project, and Path Complete.

## Summary

Testing revealed two critical infrastructure bugs affecting multiple plugins, plus several plugin-specific usability issues. The core problems are in the plugin mode keybinding system and i18n template substitution.

---

## Critical (P0) - Core Plugin Infrastructure Bugs

### 1. Custom mode keybindings don't work

**Affected Plugins:** Find References, Search Replace

**Details:** Keys defined via `editor.defineMode()` are completely non-functional:
- `q`, `Escape` (close panel)
- `Enter`/`Return` (activate/jump)
- `space`, `a`, `n`, `r` (Search Replace actions)

Navigation with arrow keys works (cursor moves), but all action keys fail. Users cannot close panels or activate items using the documented keybindings.

**nngroup Violation:** User control and freedom - users are trapped in panels with no way to exit via keyboard.

**Expected behavior:** All keybindings defined in `editor.defineMode()` should work when the virtual buffer has focus.

### 2. Template variable substitution broken in `editor.t()`

**Affected Plugins:** Find References, Search Replace

**Details:** The i18n system returns template strings without interpolating parameters:
- Shows `{symbol}`, `{count}`, `{limit}` instead of actual values
- Example: "References to {symbol} ({count}{limit})" instead of "References to 'Args' (4)"

**nngroup Violation:** Visibility of system status - users cannot see actual counts or context.

**Expected behavior:** `editor.t("key", { param: value })` should substitute `{param}` with `value`.

---

## High (P1) - Plugin-Specific Bugs

### 3. Live Grep: Preview doesn't update on navigation

**Details:** When pressing Up/Down to navigate results, the preview pane stays on the first result instead of updating to show the currently selected item.

**nngroup Violation:** Visibility of system status

### 4. Live Grep: Selection mismatch on confirm

**Details:** The file that opens on Enter is different from what's shown in the preview. For example, preview shows `build.rs:714` but `types/fresh.d.ts.template` opens.

**nngroup Violation:** Consistency and standards - unpredictable behavior

### 5. Git Grep: Opens file at wrong position

**Details:** Files open at line 1 instead of the matched line location, defeating the purpose of the search.

**nngroup Violation:** Match between system and real world

### 6. All search plugins: No visual selection indicator

**Affected Plugins:** Live Grep, Git Grep, Git Find File, Find References, Search Replace

**Details:** When navigating results with arrow keys, there's no visual indication of which item is currently selected. Users navigate blind.

**nngroup Violation:** Visibility of system status

---

## Medium (P2) - UX Improvements

### 7. No keyboard shortcuts for plugin commands

**Affected:** Live Grep, Git Grep, Git Find File, Search Replace

**Details:** These frequently-used commands have no keyboard shortcuts assigned. Users must open command palette every time.

**nngroup Violation:** Flexibility and efficiency of use

### 8. Inconsistent features across search plugins

**Details:** Live Grep has a preview pane; Git Grep doesn't. This inconsistency can confuse users switching between similar tools.

**nngroup Violation:** Consistency and standards

### 9. Status messages truncated

**Details:** Important error messages are cut off in the status bar (e.g., "Failed to op..."). Users cannot see full error details.

**nngroup Violation:** Help users recognize, diagnose, and recover from errors

---

## Low (P3) - Minor Issues

### 10. Git Find File: "New file" suggestion not shown

**Details:** When no files match the search, the "Create new file" option mentioned in code isn't visible to users.

### 11. Tab key behavior in Live Grep

**Details:** Tab replaces search text instead of being ignored or completing, causing accidental input loss.

---

## Positive Observations

- **Path Complete / File Browser:** Works well with live filtering and path navigation
- **Help text in panels:** Find References and Search Replace show keybinding help (though keybindings don't work)
- **Git Find File fuzzy search:** Good algorithm with intelligent scoring
- **Live Grep split preview concept:** Good design when working correctly
- **Arrow key navigation:** Works correctly across all plugins for moving cursor

---

## Design Principles

### 1. Use standard, discoverable keybindings

**Problem:** Current plugins use obscure single-letter keybindings that users must memorize:
- `a` - select all
- `n` - select none
- `r` - replace
- `q` - close

These are not discoverable and conflict with potential text input.

**Recommendation:** Follow VSCode-style patterns using standard keys:

| Action | Recommended Key | Rationale |
|--------|----------------|-----------|
| Navigate items | `Up` / `Down` | Universal, already works |
| Activate/confirm | `Enter` | Universal standard |
| Close/cancel | `Escape` | Universal standard |
| Toggle selection | `Space` | Common in checkbox UIs |
| Select all | `Ctrl+A` | Universal shortcut |
| Execute action | `Ctrl+Enter` | VSCode pattern for "do it" |

### 2. Minimize required keybindings

**Current Search Replace keybindings (too many):**
```
[SPC] toggle  [a] all  [n] none  [r] REPLACE  [RET] preview  [q] close
```

**Recommended minimal set:**
```
[Up/Down] navigate  [Space] toggle  [Enter] replace selected  [Esc] close
```

Changes:
- Remove `a` (select all) - use `Ctrl+A` or remove entirely (start with all selected)
- Remove `n` (select none) - rarely needed, can use toggle repeatedly
- Remove `r` - use `Enter` to execute (it's the primary action)
- Remove `q` - `Escape` is sufficient and standard

### 3. Consistent keybindings across all plugins

All result-list plugins should use the same keys:

| Key | Action |
|-----|--------|
| `Up` / `Down` | Navigate between items |
| `Enter` | Activate selected item (open file, execute replace, jump to reference) |
| `Escape` | Close panel |
| `Space` | Toggle selection (only for multi-select plugins like Search Replace) |

### 4. Arrow keys and Enter must always work

These are the primary interaction keys. Users expect:
- Arrow keys to move selection
- Enter to activate/confirm

This is partially implemented - arrow keys work but Enter doesn't due to the mode keybinding bug.

---

## Recommended Fix Priority

1. **Fix `editor.defineMode()` keybinding activation** - Unblocks all panel interactions
2. **Fix `editor.t()` parameter substitution** - Restores status visibility
3. **Add visual selection indicator** - Fundamental UX requirement
4. **Simplify plugin keybindings** - Remove obscure keys, standardize on arrows/Enter/Escape
5. **Fix Live Grep preview update** - Core feature broken
6. **Fix Git Grep cursor positioning** - Core feature broken

---

## Keybinding Refactoring Plan

### Find References
Current:
```typescript
["Return", "references_goto"],
["q", "references_close"],
["Escape", "references_close"],
```
Recommended: Keep as-is (already minimal), just remove `q`.

### Search Replace
Current:
```typescript
["Return", "search_replace_preview"],
["space", "search_replace_toggle_item"],
["a", "search_replace_select_all"],
["n", "search_replace_select_none"],
["r", "search_replace_execute"],
["q", "search_replace_close"],
["Escape", "search_replace_close"],
```
Recommended:
```typescript
["Return", "search_replace_execute"],  // Primary action
["space", "search_replace_toggle_item"],
["Escape", "search_replace_close"],
["Ctrl+a", "search_replace_select_all"],  // Optional, use standard shortcut
```

### Live Grep / Git Grep / Git Find File
These use the prompt system with suggestions - keybindings are handled by the prompt, not custom modes. Ensure:
- `Up` / `Down` navigate suggestions
- `Enter` confirms selection
- `Escape` cancels

---

## Files to Investigate

- `src/services/plugins/` - Plugin runtime, mode registration
- `src/app/plugin_commands.rs` - Command handlers for plugin API
- `src/i18n.rs` or plugin i18n handling - Template substitution logic
- `plugins/live_grep.ts:331-345` - `onLiveGrepSelectionChanged` handler
- `plugins/git_grep.ts` - File opening logic
- `plugins/find_references.ts:35-44` - Mode keybinding definition
- `plugins/search_replace.ts:35-48` - Mode keybinding definition
