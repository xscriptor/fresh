/// <reference path="./lib/fresh.d.ts" />

/**
 * Live Grep Plugin
 *
 * Project-wide search with ripgrep and live preview.
 * - Type to search across all files
 * - Navigate results with Up/Down to see preview
 * - Press Enter to open file at location
 */

interface GrepMatch {
  file: string;
  line: number;
  column: number;
  content: string;
}

// State management
let grepResults: GrepMatch[] = [];
let previewBufferId: number | null = null;
let previewSplitId: number | null = null;
let originalSplitId: number | null = null;
let lastQuery: string = "";
let searchDebounceTimer: number | null = null;
let previewCreated: boolean = false;

// Parse ripgrep output line
// Format: file:line:column:content
function parseRipgrepLine(line: string): GrepMatch | null {
  const match = line.match(/^([^:]+):(\d+):(\d+):(.*)$/);
  if (match) {
    return {
      file: match[1],
      line: parseInt(match[2], 10),
      column: parseInt(match[3], 10),
      content: match[4],
    };
  }
  return null;
}

// Parse ripgrep output into suggestions
function parseRipgrepOutput(stdout: string): {
  results: GrepMatch[];
  suggestions: PromptSuggestion[];
} {
  const results: GrepMatch[] = [];
  const suggestions: PromptSuggestion[] = [];

  for (const line of stdout.split("\n")) {
    if (!line.trim()) continue;
    const match = parseRipgrepLine(line);
    if (match) {
      results.push(match);

      // Truncate long content for display
      const displayContent =
        match.content.length > 60
          ? match.content.substring(0, 57) + "..."
          : match.content;

      suggestions.push({
        text: `${match.file}:${match.line}`,
        description: displayContent.trim(),
        value: `${results.length - 1}`, // Store index as value
        disabled: false,
      });

      // Limit to 100 results for performance
      if (results.length >= 100) {
        break;
      }
    }
  }

  return { results, suggestions };
}

// Create or update preview with real file buffer
async function updatePreview(match: GrepMatch): Promise<void> {
  try {
    if (!previewCreated) {
      // Create a split first with a placeholder virtual buffer
      const result = await editor.createVirtualBufferInSplit({
        name: "*Loading...*",
        mode: "normal",
        read_only: true,
        entries: [{ text: "Loading preview...", properties: {} }],
        ratio: 0.5,
        direction: "vertical",
        show_line_numbers: false,
      });

      previewSplitId = result.split_id ?? null;
      previewCreated = true;

      // Now open the real file in that split
      if (previewSplitId !== null) {
        editor.openFileInSplit(previewSplitId, match.file, match.line, match.column);
        previewBufferId = editor.getActiveBufferId();

        // Close the placeholder virtual buffer
        if (result.buffer_id) {
          editor.closeBuffer(result.buffer_id);
        }
      }

      // Return focus to original split so prompt stays active
      if (originalSplitId !== null) {
        editor.focusSplit(originalSplitId);
      }
    } else if (previewSplitId !== null) {
      // Update preview: open file in existing preview split
      editor.openFileInSplit(previewSplitId, match.file, match.line, match.column);
      previewBufferId = editor.getActiveBufferId();

      // Return focus to original split
      if (originalSplitId !== null) {
        editor.focusSplit(originalSplitId);
      }
    }
  } catch (e) {
    editor.debug(`Failed to update preview: ${e}`);
  }
}

// Close preview split (buffer will be closed with it)
function closePreview(): void {
  if (previewSplitId !== null) {
    editor.closeSplit(previewSplitId);
    previewSplitId = null;
    previewBufferId = null;
  }
  previewCreated = false;
}

// Run ripgrep search
async function runSearch(query: string): Promise<void> {
  if (!query || query.trim().length < 2) {
    editor.setPromptSuggestions([]);
    grepResults = [];
    return;
  }

  // Avoid duplicate searches
  if (query === lastQuery) {
    return;
  }
  lastQuery = query;

  try {
    const result = await editor.spawnProcess("rg", [
      "--line-number",
      "--column",
      "--no-heading",
      "--color=never",
      "--smart-case",
      "--max-count=100",
      "-g", "!.git",
      "-g", "!node_modules",
      "-g", "!target",
      "-g", "!*.lock",
      "--",
      query,
    ]);

    if (result.exit_code === 0) {
      const { results, suggestions } = parseRipgrepOutput(result.stdout);
      grepResults = results;
      editor.setPromptSuggestions(suggestions);

      if (results.length > 0) {
        editor.setStatus(`Found ${results.length} matches`);
        // Show preview of first result
        await updatePreview(results[0]);
      } else {
        editor.setStatus("No matches found");
      }
    } else if (result.exit_code === 1) {
      // No matches
      grepResults = [];
      editor.setPromptSuggestions([]);
      editor.setStatus("No matches found");
    } else {
      editor.setStatus(`Search error: ${result.stderr}`);
    }
  } catch (e) {
    editor.setStatus(`Search error: ${e}`);
  }
}

// Start live grep
globalThis.start_live_grep = function (): void {
  // Clear previous state
  grepResults = [];
  lastQuery = "";
  previewBufferId = null;
  previewSplitId = null;
  previewCreated = false;

  // Remember original split to keep focus
  originalSplitId = editor.getActiveSplitId();

  // Start the prompt
  editor.startPrompt("Live grep: ", "live-grep");
  editor.setStatus("Type to search (min 2 chars)...");
};

// Handle prompt input changes
globalThis.onLiveGrepPromptChanged = function (args: {
  prompt_type: string;
  input: string;
}): boolean {
  if (args.prompt_type !== "live-grep") {
    return true;
  }

  // Debounce search to avoid too many requests while typing
  if (searchDebounceTimer !== null) {
    // Can't actually cancel in this runtime, but we track it
  }

  // Run search (with small delay effect via async)
  runSearch(args.input);

  return true;
};

// Handle selection changes - update preview
globalThis.onLiveGrepSelectionChanged = function (args: {
  prompt_type: string;
  selected_index: number;
}): boolean {
  if (args.prompt_type !== "live-grep") {
    return true;
  }

  const match = grepResults[args.selected_index];
  if (match) {
    updatePreview(match);
  }

  return true;
};

// Handle prompt confirmation - keep preview open and focus it
globalThis.onLiveGrepPromptConfirmed = function (args: {
  prompt_type: string;
  selected_index: number | null;
  input: string;
}): boolean {
  if (args.prompt_type !== "live-grep") {
    return true;
  }

  // If we have a preview, focus it (file is already open there)
  if (previewSplitId !== null) {
    editor.focusSplit(previewSplitId);
    if (args.selected_index !== null && grepResults[args.selected_index]) {
      const selected = grepResults[args.selected_index];
      editor.setStatus(`Opened ${selected.file}:${selected.line}`);
    }
  } else if (args.selected_index !== null && grepResults[args.selected_index]) {
    // No preview split, open file in original split
    const selected = grepResults[args.selected_index];
    editor.openFile(selected.file, selected.line, selected.column);
    editor.setStatus(`Opened ${selected.file}:${selected.line}`);
  } else {
    editor.setStatus("No file selected");
  }

  // Clear state but don't close preview
  grepResults = [];
  originalSplitId = null;
  previewSplitId = null;
  previewBufferId = null;
  previewCreated = false;

  return true;
};

// Handle prompt cancellation - close preview
globalThis.onLiveGrepPromptCancelled = function (args: {
  prompt_type: string;
}): boolean {
  if (args.prompt_type !== "live-grep") {
    return true;
  }

  // Close preview and cleanup
  closePreview();
  grepResults = [];
  originalSplitId = null;
  editor.setStatus("Live grep cancelled");

  return true;
};

// Register event handlers
editor.on("prompt_changed", "onLiveGrepPromptChanged");
editor.on("prompt_selection_changed", "onLiveGrepSelectionChanged");
editor.on("prompt_confirmed", "onLiveGrepPromptConfirmed");
editor.on("prompt_cancelled", "onLiveGrepPromptCancelled");

// Register command
editor.registerCommand(
  "Live Grep (Find in Files)",
  "Search for text across project with live preview",
  "start_live_grep",
  "normal"
);

editor.debug("Live Grep plugin loaded");
editor.setStatus("Live Grep ready - use command palette or bind 'start_live_grep'");
