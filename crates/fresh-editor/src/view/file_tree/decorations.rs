use std::collections::HashMap;
use std::path::{Path, PathBuf};

// Re-export from fresh-core for shared type usage
pub use fresh_core::file_explorer::FileExplorerDecoration;

/// Cached decoration lookups for file explorer rendering.
#[derive(Debug, Default, Clone)]
pub struct FileExplorerDecorationCache {
    direct: HashMap<PathBuf, FileExplorerDecoration>,
    bubbled: HashMap<PathBuf, FileExplorerDecoration>,
}

impl FileExplorerDecorationCache {
    /// Rebuild the cache from a list of decorations.
    pub fn rebuild<I>(decorations: I, root: &Path) -> Self
    where
        I: IntoIterator<Item = FileExplorerDecoration>,
    {
        let mut direct = HashMap::new();
        for decoration in decorations {
            if !decoration.path.starts_with(root) {
                continue;
            }
            insert_best(&mut direct, decoration);
        }

        let mut bubbled = HashMap::new();
        for (path, decoration) in &direct {
            for ancestor in path.ancestors() {
                if !ancestor.starts_with(root) {
                    break;
                }
                insert_best(
                    &mut bubbled,
                    FileExplorerDecoration {
                        path: ancestor.to_path_buf(),
                        symbol: decoration.symbol.clone(),
                        color: decoration.color,
                        priority: decoration.priority,
                    },
                );
            }
        }

        Self { direct, bubbled }
    }

    /// Lookup a decoration for an exact path.
    /// Also tries the canonicalized path to handle symlinks.
    pub fn direct_for_path(&self, path: &Path) -> Option<&FileExplorerDecoration> {
        self.direct.get(path).or_else(|| {
            // Try canonicalized path for symlink support
            path.canonicalize().ok().and_then(|p| self.direct.get(&p))
        })
    }

    /// Lookup a bubbled decoration for a path (direct or descendant).
    /// Also tries the canonicalized path to handle symlinks.
    pub fn bubbled_for_path(&self, path: &Path) -> Option<&FileExplorerDecoration> {
        self.bubbled.get(path).or_else(|| {
            // Try canonicalized path for symlink support
            path.canonicalize().ok().and_then(|p| self.bubbled.get(&p))
        })
    }
}

fn insert_best(
    map: &mut HashMap<PathBuf, FileExplorerDecoration>,
    decoration: FileExplorerDecoration,
) {
    let replace = match map.get(&decoration.path) {
        Some(existing) => decoration.priority >= existing.priority,
        None => true,
    };

    if replace {
        map.insert(decoration.path.clone(), decoration);
    }
}
