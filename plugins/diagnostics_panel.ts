/// <reference path="./lib/fresh.d.ts" />

import {
  ResultsPanel,
  ResultItem,
  ResultsProvider,
  EventEmitter,
  getRelativePath,
} from "./lib/results-panel.ts";

const editor = getEditor();

/**
 * Diagnostics Panel Plugin
 *
 * Uses VS Code-inspired Provider pattern:
 * - DiagnosticsProvider emits events when diagnostics change
 * - ResultsPanel handles UI with syncWithEditor for bidirectional cursor sync
 * - Toggle between current file and all files
 */

// ============================================================================
// Diagnostics Provider
// ============================================================================

interface DiagnosticResultItem extends ResultItem {
  diagnosticSeverity: number; // 1=error, 2=warning, 3=info, 4=hint
}

class DiagnosticsProvider implements ResultsProvider<DiagnosticResultItem> {
  private _onDidChangeResults = new EventEmitter<void>();
  readonly onDidChangeResults = this._onDidChangeResults.event;

  private showAllFiles = false;
  private sourceBufferId: number | null = null;

  setShowAllFiles(value: boolean): void {
    this.showAllFiles = value;
    this._onDidChangeResults.fireVoid();
  }

  getShowAllFiles(): boolean {
    return this.showAllFiles;
  }

  setSourceBuffer(bufferId: number | null): void {
    this.sourceBufferId = bufferId;
    // Don't fire change event here - only when filter changes or diagnostics update
  }

  notifyDiagnosticsChanged(): void {
    this._onDidChangeResults.fireVoid();
  }

  provideResults(): DiagnosticResultItem[] {
    const diagnostics = editor.getAllDiagnostics();

    // Get active file URI for filtering
    let activeUri: string | null = null;
    if (this.sourceBufferId !== null) {
      const path = editor.getBufferPath(this.sourceBufferId);
      if (path) {
        activeUri = "file://" + path;
      }
    }

    // Filter diagnostics
    const filterUri = this.showAllFiles ? null : activeUri;
    const filtered = filterUri
      ? diagnostics.filter((d) => d.uri === filterUri)
      : diagnostics;

    // Sort by file, then line, then severity
    filtered.sort((a, b) => {
      // File comparison
      if (a.uri !== b.uri) {
        // Active file first
        if (activeUri) {
          if (a.uri === activeUri) return -1;
          if (b.uri === activeUri) return 1;
        }
        return a.uri < b.uri ? -1 : 1;
      }
      // Line comparison
      const lineDiff = a.range.start.line - b.range.start.line;
      if (lineDiff !== 0) return lineDiff;
      // Severity comparison
      return a.severity - b.severity;
    });

    // Convert to ResultItems
    return filtered.map((diag, index) => {
      const filePath = this.uriToPath(diag.uri);
      const line = diag.range.start.line + 1;
      const col = diag.range.start.character + 1;
      const message = diag.message.split("\n")[0]; // First line only

      return {
        id: `diag-${index}-${diag.uri}-${line}-${col}`,
        label: `${line}:${col} ${message}`,
        location: {
          file: filePath,
          line: line,
          column: col,
        },
        severity: this.severityToString(diag.severity),
        diagnosticSeverity: diag.severity,
        metadata: { uri: diag.uri, message: diag.message },
      };
    });
  }

  private uriToPath(uri: string): string {
    if (uri.startsWith("file://")) {
      return uri.slice(7);
    }
    return uri;
  }

  private severityToString(
    severity: number
  ): "error" | "warning" | "info" | "hint" {
    switch (severity) {
      case 1:
        return "error";
      case 2:
        return "warning";
      case 3:
        return "info";
      case 4:
        return "hint";
      default:
        return "info";
    }
  }
}

// ============================================================================
// Panel State
// ============================================================================

const diagnosticsProvider = new DiagnosticsProvider();
let panel: ResultsPanel<DiagnosticResultItem> | null = null;
let isOpen = false;
let sourceSplitId: number | null = null;

function getTitle(): string {
  const showAll = diagnosticsProvider.getShowAllFiles();
  const filterLabel = showAll
    ? editor.t("panel.all_files")
    : editor.t("panel.current_file");
  return editor.t("panel.header", { filter: filterLabel });
}

// ============================================================================
// Commands
// ============================================================================

globalThis.show_diagnostics_panel = async function (): Promise<void> {
  if (isOpen && panel) {
    // Already open - just focus the panel
    panel.focusPanel();
    return;
  }

  // Capture source context
  sourceSplitId = editor.getActiveSplitId();
  const sourceBufferId = editor.getActiveBufferId();
  diagnosticsProvider.setSourceBuffer(sourceBufferId);

  // Create the panel
  panel = new ResultsPanel(editor, "diagnostics", diagnosticsProvider, {
    title: getTitle(),
    syncWithEditor: true, // Bidirectional cursor sync
    groupBy: "file", // Group diagnostics by file
    ratio: 0.7,
    onSelect: (item) => {
      if (item.location) {
        panel!.openInSource(
          item.location.file,
          item.location.line,
          item.location.column
        );
        const displayPath = getRelativePath(editor, item.location.file);
        editor.setStatus(
          editor.t("status.jumped_to", {
            file: displayPath,
            line: String(item.location.line),
          })
        );
      }
    },
    onClose: () => {
      isOpen = false;
      panel = null;
      sourceSplitId = null;
      editor.setStatus(editor.t("status.closed"));
    },
  });

  await panel.show();
  isOpen = true;

  // Show count
  const diagnostics = editor.getAllDiagnostics();
  editor.setStatus(
    editor.t("status.diagnostics_count", { count: String(diagnostics.length) })
  );
};

globalThis.diagnostics_close = function (): void {
  if (panel) {
    panel.close();
  }
};

globalThis.diagnostics_goto = function (): void {
  if (!panel || !isOpen) return;

  const item = panel.getSelectedItem();
  if (item && item.location) {
    panel.openInSource(
      item.location.file,
      item.location.line,
      item.location.column
    );
    const displayPath = getRelativePath(editor, item.location.file);
    editor.setStatus(
      editor.t("status.jumped_to", {
        file: displayPath,
        line: String(item.location.line),
      })
    );
  } else {
    editor.setStatus(editor.t("status.move_to_diagnostic"));
  }
};

globalThis.diagnostics_toggle_all = function (): void {
  if (!isOpen) return;

  const newValue = !diagnosticsProvider.getShowAllFiles();
  diagnosticsProvider.setShowAllFiles(newValue);

  // Update panel title
  if (panel) {
    (panel as unknown as { options: { title: string } }).options.title = getTitle();
  }

  const label = newValue
    ? editor.t("panel.all_files")
    : editor.t("panel.current_file");
  editor.setStatus(editor.t("status.showing", { label }));
};

globalThis.diagnostics_refresh = function (): void {
  if (!isOpen) return;

  diagnosticsProvider.notifyDiagnosticsChanged();
  editor.setStatus(editor.t("status.refreshed"));
};

globalThis.toggle_diagnostics_panel = function (): void {
  if (isOpen) {
    globalThis.diagnostics_close();
  } else {
    globalThis.show_diagnostics_panel();
  }
};

// ============================================================================
// Event Handlers
// ============================================================================

// When diagnostics update, notify the provider
globalThis.on_diagnostics_updated = function (_data: {
  uri: string;
  count: number;
}): void {
  if (isOpen) {
    diagnosticsProvider.notifyDiagnosticsChanged();
  }
};

// When a different buffer becomes active, update filter context
globalThis.on_diagnostics_buffer_activated = function (data: {
  buffer_id: number;
}): void {
  if (!isOpen || !panel) return;

  // Ignore if the diagnostics panel itself became active
  if (panel.bufferId === data.buffer_id) {
    return;
  }

  // Update source buffer and refresh if not showing all files
  diagnosticsProvider.setSourceBuffer(data.buffer_id);
  if (!diagnosticsProvider.getShowAllFiles()) {
    diagnosticsProvider.notifyDiagnosticsChanged();
    // Update title
    (panel as unknown as { options: { title: string } }).options.title = getTitle();
  }
};

// Register event handlers
editor.on("diagnostics_updated", "on_diagnostics_updated");
editor.on("buffer_activated", "on_diagnostics_buffer_activated");

// ============================================================================
// Mode Definition (for custom keybindings beyond Enter/Escape)
// ============================================================================

// Note: The ResultsPanel already defines a mode with Enter/Escape.
// We define additional keybindings here.
editor.defineMode(
  "diagnostics-extra",
  "diagnostics-results", // Parent mode from ResultsPanel
  [
    ["a", "diagnostics_toggle_all"],
    ["r", "diagnostics_refresh"],
    ["Tab", "diagnostics_goto"],
  ],
  true
);

// ============================================================================
// Command Registration
// ============================================================================

editor.registerCommand(
  "%cmd.show_diagnostics_panel",
  "%cmd.show_diagnostics_panel_desc",
  "show_diagnostics_panel",
  "normal"
);

editor.registerCommand(
  "%cmd.toggle_diagnostics_panel",
  "%cmd.toggle_diagnostics_panel_desc",
  "toggle_diagnostics_panel",
  "normal"
);

// ============================================================================
// Initialization
// ============================================================================

editor.setStatus(editor.t("status.loaded"));
editor.debug("Diagnostics Panel plugin initialized (Provider pattern v2)");
