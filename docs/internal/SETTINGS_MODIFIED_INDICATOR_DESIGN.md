# Settings Modified Indicator Design

## Problem Statement

The current settings UI has several UX issues related to the "modified" indicator:

1. **Incorrect baseline for comparison**: The current implementation compares settings values against the schema default. This causes:
   - Auto-discovered content (like plugins) to show as "modified" on initial load because they differ from the empty schema default `{}`
   - The "Reset" button clears plugins entirely because it resets to schema default

2. **Section-level indicators are misleading**: The dot indicators on sections (General, Editor, Plugins) show "modified" based on comparison to schema defaults, not based on what the user has actually configured in the current layer.

3. **No visibility into individual item modifications**: Users cannot see which specific items have been modified at the current layer vs inherited from a lower layer.

4. **Hidden inheritance sources**: Users have no transparency into where a setting's effective value originates, making it difficult to understand configuration state.

## Goals

Design a UX similar to IntelliJ IDEA's and VS Code's settings:
- Show which items are explicitly defined in the current target layer (User/Project)
- "Reset" should remove the value from the current layer, falling back to inherited
- Auto-managed content (plugins) should not show as "modified" and should not be resettable
- Provide transparency into inheritance hierarchy ("Effective Value" visualization)

## Layered Configuration Architecture

Fresh uses a 4-layer configuration system (highest precedence first):
1. **Session** - Temporary runtime overrides (not persisted)
2. **Project** - Project-specific settings (`.fresh/config.json`)
3. **User** - User-global settings (`~/.config/fresh/config.json`)
4. **System** - Built-in defaults (schema defaults)

Values cascade: higher layers override lower layers. The final config is the merge of all layers.

### Inheritance Evaluation Model

The effective value `V_eff` of a setting `s` is determined by evaluating layers in precedence order:

```
V_eff(s) = V(s, layer_k)  where k = max{i | V(s, layer_i) is defined}
```

In other words: the effective value comes from the highest-precedence layer that defines it.

## Design

### Transparency of Inheritance (Critical)

Research indicates that systems hiding the source of a setting's value cause the highest degree of user frustration. The UI must clearly show:

1. **Where each value originates** (layer source badge)
2. **What would happen if the value were reset** (inheritance fallback)
3. **Which items are user-configured vs inherited**

This follows the "Recognizability over Recall" principle - users should see inheritance state at a glance rather than having to remember or investigate.

### Definition of "Modified"

**Current behavior**: `modified = (current_value != schema_default)`

**Proposed behavior**: `modified = (value is defined in target_layer)`

For example, when editing User layer settings:
- An item is "modified" if it has a value defined in the User layer
- An item is NOT modified if it comes from System defaults or is undefined

This aligns with the UX concept: "modified" means "the user explicitly configured this in the current layer."

### Section-Level Indicators

The dot indicator next to category names (e.g., "General", "Editor") should show:
- **Filled dot (●)**: At least one item in this section is defined in the target layer
- **Empty (space)**: No items in this section are defined in the target layer

This is computed by aggregating: `category_modified = any(item.modified for item in category.items)`

### Individual Item Indicators

Each setting item should display:

1. **Layer source badge**: A small label showing which layer the current value comes from
   - Positioned inline after the setting name or value
   - Color-coded by layer:
     - `[default]` - dimmed/gray (System layer)
     - `[user]` - subtle highlight (User layer)
     - `[project]` - distinct color (Project layer)
     - `[session]` - ephemeral indicator (Session layer)

2. **Modified indicator (●)**: Shows if the item is defined in the current target layer
   - Appears as a small dot next to the setting name
   - Only visible when the setting is defined in the target layer being edited

#### Visual Hierarchy

Following the research on visual hierarchy, the modified indicator should have higher visual weight than the layer source badge, as it represents actionable state (something the user can reset).

```
┌─────────────────────────────────────────────────────────────┐
│ ● Tab Size               : [  4  ] [-] [+]     [project]    │
│   Number of spaces per tab                                  │
├─────────────────────────────────────────────────────────────┤
│   Line Numbers           : [x]                 [default]    │
│   Show line numbers in the gutter                           │
└─────────────────────────────────────────────────────────────┘
```

In this example:
- "Tab Size" has ● because it's defined in the current target layer (Project)
- "Line Numbers" has no ● because its value comes from defaults

### Reset Behavior

**Current behavior**: Reset sets the value to schema default.

**Proposed behavior**: Reset removes the value from the current layer's delta.

This means:
- If User layer defines `tab_size: 2`, clicking Reset removes it from User layer
- The value then falls back to System default (or Project layer if editing Session)
- Items not defined in the current layer have nothing to reset
- Reset button should be disabled/hidden for items not defined in current layer

#### Reset Confirmation

For destructive operations (reset all in section, reset to defaults), show a confirmation indicating:
- What values will be removed
- What the new effective values will be after reset

### Auto-Managed Content (Maps with `x-no-add`)

Plugins and other auto-discovered content use `x-no-add` schema extension:
- These Maps are populated automatically, not by user configuration
- They should **never** show as "modified" (even though they differ from empty default)
- They should **never** be resettable (Reset has no meaning for auto-discovered content)
- They should skip the modified calculation entirely
- Individual entries within these maps CAN show modified (e.g., disabling a plugin)

### Progressive Disclosure

Following the 80/20 principle from UX research:
- Show commonly-used settings by default
- Hide advanced settings behind expandable sections
- Consider adding "Show Advanced" toggle per category

Currently Fresh shows all settings flat, which may be acceptable given the search functionality.

## Implementation Changes

### 1. `build_item` / `build_page` functions

Add parameters:
- `layer_sources: &HashMap<String, ConfigLayer>` - Maps paths to their source layer
- `target_layer: ConfigLayer` - The layer being edited

Calculate modified as:
```rust
// For regular items
let modified = layer_sources.get(&schema.path) == Some(&target_layer);

// For Maps with no_add (auto-managed)
let modified = false; // Container is never "modified"

// For entries WITHIN Maps (even no_add maps)
// Check if the specific entry path exists in target layer
let entry_path = format!("{}/{}", schema.path, entry_key);
let entry_modified = layer_sources.get(&entry_path) == Some(&target_layer);
```

### 2. `reset_current_to_default` function

Change from:
```rust
// Set value to schema default
self.set_pending_change(&path, default.clone());
```

To:
```rust
// Remove value from delta (fall back to inherited)
// Only if the item is defined in the current layer
if item.modified {
    self.remove_from_delta(&path);
    // Recalculate effective value from remaining layers
    let new_value = self.compute_effective_value(&path);
    self.update_control(&path, new_value);
}
```

### 3. Add `remove_from_delta` method

New method to remove a path from the pending changes that will result in deletion from the layer:

```rust
/// Mark a path for removal from the current layer's delta.
/// On save, this path will be deleted from the layer file.
pub fn remove_from_delta(&mut self, path: &str) {
    // Use a special marker value or separate tracking for deletions
    self.pending_deletions.insert(path.to_string());
    self.pending_changes.remove(path);
}
```

### 4. Section indicator calculation

Already correct: `page.items.iter().any(|i| i.modified)`

Once `modified` is calculated correctly per-item, section indicators will automatically work.

### 5. Render layer source badges

In `render.rs`, add rendering for layer source badges:

```rust
fn render_layer_badge(layer: ConfigLayer, theme: &Theme) -> Span {
    let (text, style) = match layer {
        ConfigLayer::System => ("default", Style::default().fg(theme.text_muted)),
        ConfigLayer::User => ("user", Style::default().fg(theme.text_secondary)),
        ConfigLayer::Project => ("project", Style::default().fg(theme.accent)),
        ConfigLayer::Session => ("session", Style::default().fg(theme.warning)),
    };
    Span::styled(format!("[{}]", text), style)
}
```

## Migration Path

1. Update `build_item` signature to accept layer info
2. Update all callers (`build_page`, `build_pages`)
3. Pass `layer_sources` and `target_layer` from `SettingsState`
4. Add `pending_deletions` tracking to `SettingsState`
5. Update `reset_current_to_default` to remove from delta
6. Add layer source badge rendering
7. Update tests that rely on old "modified" semantics

## Testing Considerations

1. **Modified indicator accuracy**: Test that items defined in target layer show ●
2. **Reset behavior**: Test that reset removes from delta, not sets to default
3. **Auto-managed content**: Test that `no_add` maps don't show modified
4. **Layer switching**: Test that modified indicators update when switching target layer
5. **Save/Load cycle**: Test that saved changes persist correctly and show as modified on reload

## Future Considerations

- **Effective Value Visualization**: On hover/focus, show a "Policy Stack" popup displaying all layers that attempted to define the value (similar to AWS IAM Policy Simulator)
- **Diff view**: Side-by-side comparison showing what's defined at each layer
- **Search by layer**: Filter settings to show only those defined in a specific layer
- **Conflict indicators**: When a higher layer overrides a lower layer, consider visual indication
- **Export/Import**: Allow exporting layer-specific settings for sharing
- **Validation warnings**: Show warnings when settings might conflict or cause issues

## Alignment with Industry Best Practices

This design follows key principles identified in configuration UX research:

1. **Transparency of Inheritance**: Clear layer source badges and modified indicators
2. **Safety Through Fallback**: Reset removes from layer rather than clearing entirely
3. **Good Defaults**: Auto-managed content doesn't pollute modified state
4. **Explicit Save Model**: Changes are staged, not auto-applied (matches VS Code/IntelliJ)
5. **Search First**: Existing search functionality supports navigation (keep this)
6. **Visual Hierarchy**: Modified indicator has higher weight than source badge
