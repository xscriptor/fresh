# Git Blame View Transform Bug Analysis

## Current Status
The git blame view transform is partially working. Block headers appear for the **second and subsequent blocks**, but **not for the first block** (starting at byte 0).

## Test Output Evidence

### Working Test (2 blocks) - `test_git_blame_line_numbers_correct`
```
      │                                           <- Row 1: blank gutter, EMPTY content (BUG!)
      │ Line 1 from first commit                  <- Row 2: blank gutter (wrong! should show line 1)
    2 │ Line 2 from first commit                  <- Row 3: line 2
      │ ── c859585 ... "Second commit" ──         <- Row 4: block 2 header WORKS
      │ Line 3 from second commit                 <- Row 5: blank gutter (correct after header)
    3 │ Line 4 from second commit                 <- Row 6: line 3
```

### Failing Test (1 block) - `test_git_blame_scroll_to_bottom`
```
      │                                           <- Row 1: blank gutter, EMPTY content
      │ Line 1 content                            <- Row 2: blank gutter (wrong!)
    2 │ Line 2 content                            <- Row 3: line 2
```

## Key Observations

1. **Blank gutter appears** - This proves tokens with `source_offset: null` ARE being injected
2. **Header text is empty for block 1** - The row exists but has no visible text
3. **Block 2 header works perfectly** - Shows full header `── hash (author, date) "message" ──`
4. **Following line also has blank gutter** - The Newline token after header seems to affect the next line too

## Architecture Summary

### View Transform Flow
1. `render.rs` fires `view_transform_request` hook with base tokens
2. Plugin processes tokens, injects headers at block boundaries
3. Plugin calls `submitViewTransform()` with modified tokens
4. `process_commands()` receives the transform (async - may be delayed)
5. `build_view_data()` uses the transform tokens
6. `flatten_tokens()` converts tokens to text + source_offset mapping
7. Rendering uses mapping to determine line numbers (None = blank gutter)

### Token Injection Logic (git_blame.ts)
```typescript
// For each source token, check if we need to inject header before it
if (byteOffset === block.startByte && !processedBlocks.has(blockKey)) {
  // Inject header token
  transformed.push({
    source_offset: null,
    kind: { Text: headerText },  // <- THE BUG: headerText empty for block 1?
    style: { fg, bg, bold: true, italic: false }
  });
  // Inject newline
  transformed.push({
    source_offset: null,
    kind: "Newline",
    style: { ... }
  });
}
// Then push original token
transformed.push(token);
```

## Hypotheses

### H1: Header text is empty when startByte === 0
The `formatBlockHeader()` function or block data might be malformed for the first block.

### H2: Token ordering issue at byte 0
The first token in the stream might be processed differently, causing the header to be lost or overwritten.

### H3: Timing/async issue specific to first render
The transform might not be fully applied on the first render cycle, and subsequent cycles overwrite it.

### H4: View anchor calculation issue
The `calculate_view_anchor()` function uses `source_offset` to find the viewport position. Injected tokens at byte 0 might confuse this calculation.

## Next Steps

1. **Add isolated unit test** - Test `formatBlockHeader()` directly with block at startByte=0
2. **Log exact tokens** - Print the `transformed` array before submission to verify content
3. **Test timing** - Add multiple render cycles before checking output
4. **Compare token streams** - Log tokens for both block 1 and block 2 side by side

## Related Files
- `plugins/git_blame.ts` - View transform implementation
- `src/ui/split_rendering.rs` - Token rendering and line numbers
- `src/view.rs` - `flatten_tokens()` function
- `tests/e2e/git.rs` - E2E tests for git blame
