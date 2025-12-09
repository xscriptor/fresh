/// <reference path="./lib/fresh.d.ts" />

/**
 * C# Language Server Support Plugin
 *
 * Handles LSP server requests from C# language servers like:
 * - csharp-ls
 * - csharp-language-server (Roslyn-based)
 * - OmniSharp
 *
 * Features:
 * - Auto-restore NuGet packages when opening C# files
 * - Auto-restore NuGet packages when the server requests it
 */

interface LspServerRequestData {
  language: string;
  method: string;
  server_command: string;
  params: string | null;
}

interface ProjectNeedsRestoreParams {
  projectFilePath: string;
}

interface AfterFileOpenData {
  path: string;
  buffer_id: number;
}

// Track which directories we've already restored to avoid repeated restores
const restoredDirectories = new Set<string>();

/**
 * Run dotnet restore for a project
 */
async function restoreProject(projectPath: string): Promise<void> {
  editor.setStatus(`Restoring NuGet packages for ${projectPath}...`);
  editor.debug(`csharp_support: Running dotnet restore for ${projectPath}`);

  try {
    const result = await editor.spawnProcess("dotnet", ["restore", projectPath]);

    if (result.exit_code === 0) {
      editor.setStatus(`NuGet restore completed for ${projectPath}`);
      editor.debug(`csharp_support: dotnet restore succeeded`);
    } else {
      editor.setStatus(`NuGet restore failed: ${result.stderr}`);
      editor.debug(`csharp_support: dotnet restore failed: ${result.stderr}`);
    }
  } catch (e) {
    editor.setStatus(`NuGet restore error: ${e}`);
    editor.debug(`csharp_support: dotnet restore error: ${e}`);
  }
}

/**
 * Handle LSP server requests from C# language servers (Roslyn-based)
 */
globalThis.on_csharp_lsp_server_request = function (
  data: LspServerRequestData
): void {
  // Only handle requests from C# language servers
  if (data.server_command !== "csharp-ls" && data.server_command !== "csharp-language-server") {
    return;
  }

  editor.debug(
    `csharp_support: Received LSP request ${data.method} from ${data.server_command}`
  );

  switch (data.method) {
    case "workspace/_roslyn_projectNeedsRestore": {
      // Roslyn LSP server is asking us to restore a project
      if (data.params) {
        try {
          const params: ProjectNeedsRestoreParams = JSON.parse(data.params);
          if (params.projectFilePath) {
            restoreProject(params.projectFilePath);
          }
        } catch (e) {
          editor.debug(`csharp_support: Failed to parse params: ${e}`);
        }
      }
      break;
    }

    default:
      // Log unhandled requests for debugging
      editor.debug(
        `csharp_support: Unhandled LSP request: ${data.method}`
      );
  }
};

// Register hook for LSP server requests
editor.on("lsp_server_request", "on_csharp_lsp_server_request");

/**
 * Get the directory containing the file
 */
function getDirectory(filePath: string): string {
  const lastSlash = filePath.lastIndexOf("/");
  if (lastSlash === -1) {
    return ".";
  }
  return filePath.substring(0, lastSlash);
}

/**
 * Proactively run dotnet restore when opening a C# file
 * This ensures the LSP server has access to restored packages from the start
 */
globalThis.on_csharp_file_open = async function (
  data: AfterFileOpenData
): Promise<void> {
  // Only handle .cs files
  if (!data.path.endsWith(".cs")) {
    return;
  }

  const dir = getDirectory(data.path);

  // Skip if we've already restored this directory
  if (restoredDirectories.has(dir)) {
    return;
  }

  // Mark as restored (even before we try, to avoid repeated attempts)
  restoredDirectories.add(dir);

  editor.debug(`csharp_support: C# file opened, running dotnet restore in ${dir}`);
  await restoreProject(dir);
};

// Register hook for file open
editor.on("after_file_open", "on_csharp_file_open");
