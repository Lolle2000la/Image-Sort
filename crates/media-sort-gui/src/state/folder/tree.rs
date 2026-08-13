use std::path::{Path, PathBuf};

use media_sort_core::models::{FolderNode, PinnedFolder};

use crate::state::folder_inspection::inspect_folder;

pub(crate) fn collect_expanded_paths(tree: &[FolderNode]) -> std::collections::HashSet<PathBuf> {
    let mut set = std::collections::HashSet::new();
    fn collect(nodes: &[FolderNode], set: &mut std::collections::HashSet<PathBuf>) {
        for node in nodes {
            if node.is_expanded {
                set.insert(node.path.clone());
            }
            collect(&node.children, set);
        }
    }
    collect(tree, &mut set);
    set
}

pub(crate) fn build_tree_nodes_data(
    root: &Path,
    pinned_folders: &[PinnedFolder],
    expanded_paths: &std::collections::HashSet<PathBuf>,
) -> Vec<FolderNode> {
    fn restore_expansion(nodes: &mut [FolderNode], set: &std::collections::HashSet<PathBuf>) {
        for node in nodes {
            if set.contains(&node.path) {
                node.is_expanded = true;
            }
            restore_expansion(&mut node.children, set);
        }
    }

    let mut tree = Vec::new();

    let mut children: Vec<_> = build_parent_chain(root)
        .into_iter()
        .rev()
        .chain(build_children(root, Some(root)))
        .collect();
    restore_expansion(&mut children, expanded_paths);
    let root_inspection = inspect_folder(root);
    tree.push(FolderNode {
        path: root.to_path_buf(),
        name: root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| root.display().to_string()),
        children,
        is_current: true,
        is_expanded: expanded_paths.is_empty() || expanded_paths.contains(root),
        is_parent_nav: false,
        kind: root_inspection.kind,
        symlink_target: root_inspection.symlink_target,
    });

    for pinned in pinned_folders {
        if media_sort_core::path_utils::paths_equal(root, &pinned.path) {
            continue;
        }
        let mut pinned_children: Vec<_> = build_parent_chain(&pinned.path)
            .into_iter()
            .rev()
            .chain(build_children(&pinned.path, Some(root)))
            .collect();
        restore_expansion(&mut pinned_children, expanded_paths);
        let pinned_inspection = inspect_folder(&pinned.path);
        tree.push(FolderNode {
            path: pinned.path.clone(),
            name: pinned.name.clone(),
            children: pinned_children,
            is_current: false,
            is_expanded: expanded_paths.contains(&pinned.path),
            is_parent_nav: false,
            kind: pinned_inspection.kind,
            symlink_target: pinned_inspection.symlink_target,
        });
    }

    // Restored-expanded chain nodes were built with only their nested
    // breadcrumb chain; populate their real children so a restored tree
    // shows the same content as a manually expanded one.
    rebuild_expanded_children(&mut tree, Some(root));

    tree
}

pub(crate) fn first_visible_child(nodes: &[FolderNode], path: &Path) -> Option<PathBuf> {
    for node in nodes {
        if node.path.as_os_str().is_empty() {
            continue;
        }
        if node.path == path {
            return node
                .children
                .iter()
                .find(|c| !c.path.as_os_str().is_empty())
                .map(|c| c.path.clone());
        }
        if let Some(res) = first_visible_child(&node.children, path) {
            return Some(res);
        }
    }
    None
}

fn build_children(parent: &Path, current: Option<&Path>) -> Vec<FolderNode> {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return Vec::new();
    };

    let mut children: Vec<FolderNode> = entries
        .flatten()
        .filter(|entry| {
            entry.file_type().is_ok_and(|ft| {
                // Symbolic links to directories are followed transparently:
                // they appear as regular child nodes (with the symlink icon)
                // and their children are read from the link target.
                ft.is_dir() || (ft.is_symlink() && entry.path().is_dir())
            })
        })
        .map(|entry| {
            let path = entry.path();
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let is_current =
                current.is_some_and(|c| media_sort_core::path_utils::paths_equal(c, &path));

            let inspection = inspect_folder(&path);

            let node_children = if inspection.has_child_dir {
                vec![FolderNode {
                    is_expanded: true,
                    ..FolderNode::default()
                }]
            } else {
                Vec::new()
            };

            FolderNode {
                path,
                name,
                children: node_children,
                is_current,
                is_expanded: false,
                is_parent_nav: false,
                kind: inspection.kind,
                symlink_target: inspection.symlink_target,
            }
        })
        .collect();

    children.sort_by_cached_key(|a| a.name.to_lowercase());
    children
}

fn is_dummy_or_empty(children: &[FolderNode]) -> bool {
    children.is_empty() || (children.len() == 1 && children[0].path.as_os_str().is_empty())
}

fn build_parent_chain(current: &Path) -> Vec<FolderNode> {
    let ancestors: Vec<std::path::PathBuf> =
        std::iter::successors(current.parent(), |p| p.parent())
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.to_path_buf())
            .collect();

    if ancestors.is_empty() {
        return Vec::new();
    }

    ancestors
        .into_iter()
        .rev()
        .fold(None, |prev: Option<FolderNode>, ancestor| {
            let name = ancestor
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| ancestor.display().to_string());

            let inspection = inspect_folder(&ancestor);

            Some(FolderNode {
                path: ancestor,
                name,
                children: prev.map(|p| vec![p]).unwrap_or_default(),
                is_current: false,
                is_expanded: false,
                is_parent_nav: true,
                kind: inspection.kind,
                symlink_target: inspection.symlink_target,
            })
        })
        .map(|rootmost| vec![rootmost])
        .unwrap_or_default()
}

/// Rebuilds a node's children from the filesystem: the real subfolder
/// listing is prepended with the preserved parent-navigation (chain) child
/// nodes, so a chain node keeps its ancestor breadcrumbs after expanding.
fn rebuild_node_children(node: &mut FolderNode, current_folder: Option<&Path>) {
    let current = if node.is_current {
        Some(node.path.as_path())
    } else {
        current_folder
    };

    let parent_nav_nodes: Vec<FolderNode> = node
        .children
        .drain(..)
        .filter(|c| c.is_parent_nav)
        .collect();

    let mut new_children = build_children(&node.path, current);

    new_children.splice(0..0, parent_nav_nodes);

    node.children = new_children;
}

/// Whether an expanded node still needs its children rebuilt. Chain nodes
/// built from `build_parent_chain` start with only nested chain children
/// (or none at all) — no real subfolder listing — and `restore_expansion`
/// can mark them expanded again, so they must be populated.
fn needs_children_rebuild(node: &FolderNode) -> bool {
    node.is_expanded
        && node.path.is_dir()
        && (is_dummy_or_empty(&node.children)
            || (node.is_parent_nav
                && node
                    .children
                    .iter()
                    .all(|c| c.is_parent_nav || c.path.as_os_str().is_empty())))
}

/// Populates children for every expanded node that lacks them (see
/// [`needs_children_rebuild`]). Called after `restore_expansion` in
/// `build_tree_nodes_data` so restored-expanded chain nodes show their real
/// children instead of only the nested breadcrumb chain.
fn rebuild_expanded_children(nodes: &mut [FolderNode], current_folder: Option<&Path>) {
    for node in nodes.iter_mut() {
        if needs_children_rebuild(node) {
            rebuild_node_children(node, current_folder);
        }
        rebuild_expanded_children(&mut node.children, current_folder);
    }
}

/// Counts the flat index of `path` in the tree, mirroring the view's
/// ordering: empty-path (dummy) nodes are skipped, depth-first.
pub(crate) fn flat_index_of(nodes: &[FolderNode], path: &Path) -> Option<usize> {
    let mut running = 0;
    fn walk(nodes: &[FolderNode], path: &Path, running: &mut usize) -> Option<usize> {
        for node in nodes {
            if node.path.as_os_str().is_empty() {
                continue;
            }
            if node.path == path {
                return Some(*running);
            }
            *running += 1;
            if let Some(found) = walk(&node.children, path, running) {
                return Some(found);
            }
        }
        None
    }
    walk(nodes, path, &mut running)
}

pub(crate) fn toggle_expand_recursive(
    nodes: &mut [FolderNode],
    path: &Path,
    idx: usize,
    running_idx: &mut usize,
    current_folder: Option<&Path>,
) -> bool {
    for node in nodes.iter_mut() {
        // Mirror the view's flat-index ordering: dummy nodes (empty path)
        // are skipped and do not consume an index.
        if node.path.as_os_str().is_empty() {
            continue;
        }
        let node_idx = *running_idx;
        *running_idx += 1;
        // The index disambiguates duplicate paths (the current root and a
        // chain node can share the same path, e.g. pinned breadcrumbs).
        if node_idx == idx && node.path == path {
            if node.path.exists() && node.children.is_empty() && !node.is_parent_nav {
                return true;
            }
            node.is_expanded = !node.is_expanded;
            if node.is_expanded
                && (is_dummy_or_empty(&node.children) || node.is_parent_nav)
                && node.path.is_dir()
            {
                rebuild_node_children(node, current_folder);
            }
            return true;
        }
        if toggle_expand_recursive(&mut node.children, path, idx, running_idx, current_folder) {
            return true;
        }
    }
    false
}

pub(crate) fn collect_visible_folders_recursive(nodes: &[FolderNode], list: &mut Vec<PathBuf>) {
    for node in nodes {
        if node.path.as_os_str().is_empty() {
            continue;
        }
        list.push(node.path.clone());
        if node.is_expanded {
            collect_visible_folders_recursive(&node.children, list);
        }
    }
}

pub(crate) fn set_expand_recursive(
    nodes: &mut [FolderNode],
    path: &Path,
    expand: bool,
    current_folder: Option<&Path>,
) -> bool {
    for node in nodes.iter_mut() {
        if node.path == path {
            if expand && node.path.exists() && node.children.is_empty() && !node.is_parent_nav {
                return true;
            }
            if node.is_expanded != expand {
                node.is_expanded = expand;
                if node.is_expanded
                    && (is_dummy_or_empty(&node.children) || node.is_parent_nav)
                    && node.path.is_dir()
                {
                    rebuild_node_children(node, current_folder);
                }
            }
            return true;
        }
        if set_expand_recursive(&mut node.children, path, expand, current_folder) {
            return true;
        }
    }
    false
}

pub(crate) fn find_node_expanded(nodes: &[FolderNode], path: &Path) -> Option<bool> {
    for node in nodes {
        if node.path.as_os_str().is_empty() {
            continue;
        }
        if node.path == path {
            return Some(node.is_expanded);
        }
        if let Some(res) = find_node_expanded(&node.children, path) {
            return Some(res);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    use media_sort_core::models::FolderKind;
    #[cfg(unix)]
    use media_sort_core::settings::store::SettingsStore;

    #[cfg(unix)]
    use crate::state::AppState;

    #[test]
    fn test_toggle_expand_collapsed_node() {
        let mut root = FolderNode {
            path: PathBuf::from("/root"),
            name: "root".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
            ..FolderNode::default()
        };
        let child_path = PathBuf::from("/root/sub");
        let found = {
            let idx = flat_index_of(&root.children, &child_path).unwrap_or(0);
            toggle_expand_recursive(&mut root.children, &child_path, idx, &mut 0, None)
        };
        assert!(!found);
        let child = FolderNode {
            path: child_path.clone(),
            name: "sub".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
            ..FolderNode::default()
        };
        root.children = vec![child];
        let found = {
            let idx = flat_index_of(&root.children, &child_path).unwrap_or(0);
            toggle_expand_recursive(&mut root.children, &child_path, idx, &mut 0, None)
        };
        assert!(found);
        assert!(root.children[0].is_expanded);
    }

    #[test]
    fn test_toggle_expand_toggle_back() {
        let child = FolderNode {
            path: PathBuf::from("/root/sub"),
            name: "sub".into(),
            children: vec![],
            is_current: false,
            is_expanded: true,
            is_parent_nav: false,
            ..FolderNode::default()
        };
        let mut children = vec![child];
        let found = {
            let idx = flat_index_of(&children, &PathBuf::from("/root/sub")).unwrap_or(0);
            toggle_expand_recursive(
                &mut children,
                &PathBuf::from("/root/sub"),
                idx,
                &mut 0,
                None,
            )
        };
        assert!(found);
        assert!(!children[0].is_expanded);
    }

    #[test]
    fn test_toggle_expand_nested_path() {
        let grandchild = FolderNode {
            path: PathBuf::from("/root/sub/deep"),
            name: "deep".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
            ..FolderNode::default()
        };
        let child = FolderNode {
            path: PathBuf::from("/root/sub"),
            name: "sub".into(),
            children: vec![grandchild],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
            ..FolderNode::default()
        };
        let mut children = vec![child];
        let found = {
            let idx = flat_index_of(&children, &PathBuf::from("/root/sub/deep")).unwrap_or(0);
            toggle_expand_recursive(
                &mut children,
                &PathBuf::from("/root/sub/deep"),
                idx,
                &mut 0,
                None,
            )
        };
        assert!(found);
        assert!(!children[0].is_expanded);
        assert!(children[0].children[0].is_expanded);
    }

    #[test]
    fn test_toggle_expand_parent_nav_node() {
        let dir = std::env::temp_dir().join(format!("mediasort_test_nav_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sub1 = dir.join("sub1");
        let sub2 = dir.join("sub2");
        std::fs::create_dir(&sub1).unwrap();
        std::fs::create_dir(&sub2).unwrap();

        let child_node = FolderNode {
            path: sub1.clone(),
            name: "sub1".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: false,
            ..FolderNode::default()
        };

        let nav_node = FolderNode {
            path: dir.clone(),
            name: "dir".into(),
            children: vec![child_node],
            is_current: false,
            is_expanded: false,
            is_parent_nav: true,
            ..FolderNode::default()
        };

        let mut tree = vec![nav_node];
        let found = {
            let idx = flat_index_of(&tree, &dir).unwrap_or(0);
            toggle_expand_recursive(&mut tree, &dir, idx, &mut 0, Some(&sub1))
        };

        assert!(found);
        assert!(tree[0].is_expanded);
        assert_eq!(tree[0].children.len(), 2);
        assert!(tree[0].is_parent_nav);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_toggle_expand_parent_nav_preserves_chain() {
        let dir = std::env::temp_dir().join(format!("mediasort_test_chain_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sub1 = dir.join("sub1");
        std::fs::create_dir(&sub1).unwrap();

        let grandparent_node = FolderNode {
            path: PathBuf::from("/grandparent"),
            name: "grandparent".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: true,
            ..FolderNode::default()
        };

        let nav_node = FolderNode {
            path: dir.clone(),
            name: "dir".into(),
            children: vec![grandparent_node],
            is_current: false,
            is_expanded: false,
            is_parent_nav: true,
            ..FolderNode::default()
        };

        let mut tree = vec![nav_node];
        let found = {
            let idx = flat_index_of(&tree, &dir).unwrap_or(0);
            toggle_expand_recursive(&mut tree, &dir, idx, &mut 0, Some(&sub1))
        };

        assert!(found);
        assert!(tree[0].is_expanded);
        assert_eq!(tree[0].children.len(), 2);
        assert!(
            tree[0]
                .children
                .iter()
                .any(|c| c.path == std::path::Path::new("/grandparent") && c.is_parent_nav)
        );
        assert!(tree[0].children.iter().any(|c| c.path == sub1));
        assert!(tree[0].is_parent_nav);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_toggle_expand_parent_nav_retains_special_handling() {
        let dir =
            std::env::temp_dir().join(format!("mediasort_test_handling_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sub1 = dir.join("sub1");
        std::fs::create_dir(&sub1).unwrap();

        let nav_node = FolderNode {
            path: dir.clone(),
            name: "dir".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: true,
            ..FolderNode::default()
        };

        let mut tree = vec![nav_node];
        let found = {
            let idx = flat_index_of(&tree, &dir).unwrap_or(0);
            toggle_expand_recursive(&mut tree, &dir, idx, &mut 0, Some(&sub1))
        };

        assert!(found);
        assert!(tree[0].is_expanded);
        assert!(
            tree[0].is_parent_nav,
            "Folder lost its special parent navigation status upon expansion!"
        );
        assert!(
            !tree[0].children.is_empty(),
            "children should be populated after expand"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_build_children_filters_files() {
        let dir = std::env::temp_dir().join(format!("mediasort_bc_filter_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::create_dir(dir.join("subdir")).unwrap();
        std::fs::write(dir.join("file.txt"), b"data").unwrap();
        std::fs::write(dir.join("another.jpg"), b"image").unwrap();

        let children = build_children(&dir, None);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "subdir");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_build_children_nonexistent_dir() {
        let nonexistent = std::path::PathBuf::from("/nonexistent/dir_12345_xyz");
        let children = build_children(&nonexistent, None);
        assert!(children.is_empty());
    }

    #[test]
    fn test_build_children_is_current() {
        let dir = std::env::temp_dir().join(format!("mediasort_bc_current_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sub = dir.join("sub");
        std::fs::create_dir(&sub).unwrap();

        let canonical_sub = sub.canonicalize().unwrap();
        let children = build_children(&dir, Some(&canonical_sub));
        assert_eq!(children.len(), 1);
        assert!(children[0].is_current);

        let children2 = build_children(&dir, None);
        assert!(!children2[0].is_current);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_build_children_no_subdirectories_no_dummy() {
        let dir =
            std::env::temp_dir().join(format!("mediasort_test_nodummy_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let sub = dir.join("sub_with_only_files");
        std::fs::create_dir(&sub).unwrap();

        for i in 0..5 {
            std::fs::write(sub.join(format!("file_{}.jpg", i)), b"data").unwrap();
        }

        let children = build_children(&dir, None);

        assert_eq!(children.len(), 1);
        assert!(
            children[0].children.is_empty(),
            "Dummy node injected into a directory containing zero subfolders!"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn test_build_children_includes_symlinked_dirs() {
        let dir = std::env::temp_dir().join(format!("mediasort_bc_symlink_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let real = dir.join("real");
        let real_sub = real.join("nested");
        std::fs::create_dir_all(&real_sub).unwrap();
        let link = dir.join("link");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        let children = build_children(&dir, None);
        assert_eq!(children.len(), 2);

        let link_node = children.iter().find(|c| c.path == link).unwrap();
        assert_eq!(link_node.kind, FolderKind::Symlink);
        assert_eq!(
            link_node.symlink_target,
            Some(real.canonicalize().unwrap()),
            "the resolved final target must be shown"
        );
        assert!(
            !link_node.children.is_empty(),
            "symlinked dirs with subfolders need the dummy child for the chevron"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn test_toggle_expand_parent_nav_idempotency() {
        let dir =
            std::env::temp_dir().join(format!("mediasort_test_idempotency_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sub = dir.join("sub1");
        std::fs::create_dir(&sub).unwrap();

        let grandparent_node = FolderNode {
            path: PathBuf::from("/grandparent"),
            name: "grandparent".into(),
            children: vec![],
            is_current: false,
            is_expanded: false,
            is_parent_nav: true,
            ..FolderNode::default()
        };

        let mut tree = vec![FolderNode {
            path: dir.clone(),
            name: "dir".into(),
            children: vec![grandparent_node],
            is_current: false,
            is_expanded: false,
            is_parent_nav: true,
            ..FolderNode::default()
        }];

        {
            let idx = flat_index_of(&tree, &dir).unwrap_or(0);
            toggle_expand_recursive(&mut tree, &dir, idx, &mut 0, Some(&sub))
        };
        assert_eq!(tree[0].children.len(), 2);

        {
            let idx = flat_index_of(&tree, &dir).unwrap_or(0);
            toggle_expand_recursive(&mut tree, &dir, idx, &mut 0, Some(&sub))
        };
        assert!(!tree[0].is_expanded);

        {
            let idx = flat_index_of(&tree, &dir).unwrap_or(0);
            toggle_expand_recursive(&mut tree, &dir, idx, &mut 0, Some(&sub))
        };
        assert!(tree[0].is_expanded);
        assert_eq!(
            tree[0].children.len(),
            2,
            "Re-expanding a parent navigation node duplicated or corrupted the child array!"
        );

        std::fs::remove_dir_all(&dir).ok();
    }

    /// Flat index of the `nth` node (0-based) whose path equals `path`,
    /// mirroring the view's ordering (dummy nodes skipped).
    fn flat_index_of_nth(nodes: &[FolderNode], path: &Path, nth: usize) -> Option<usize> {
        let mut running = 0;
        let mut seen = 0;
        fn walk(
            nodes: &[FolderNode],
            path: &Path,
            nth: usize,
            running: &mut usize,
            seen: &mut usize,
        ) -> Option<usize> {
            for node in nodes {
                if node.path.as_os_str().is_empty() {
                    continue;
                }
                if node.path == path {
                    if *seen == nth {
                        return Some(*running);
                    }
                    *seen += 1;
                }
                *running += 1;
                if let Some(found) = walk(&node.children, path, nth, running, seen) {
                    return Some(found);
                }
            }
            None
        }
        walk(nodes, path, nth, &mut running, &mut seen)
    }

    /// Collects `(path, flat_index)` for every parent-nav (chain) node,
    /// mirroring the view's ordering (dummy nodes skipped).
    #[cfg(unix)]
    fn collect_parent_nav_indices(
        nodes: &[FolderNode],
        running: &mut usize,
        out: &mut Vec<(PathBuf, usize)>,
    ) {
        for node in nodes {
            if node.path.as_os_str().is_empty() {
                continue;
            }
            let idx = *running;
            *running += 1;
            if node.is_parent_nav {
                out.push((node.path.clone(), idx));
            }
            collect_parent_nav_indices(&node.children, running, out);
        }
    }

    /// Finds the first parent-nav node with `path`, ignoring duplicate
    /// non-chain nodes (roots) with the same path.
    fn find_parent_nav_node<'a>(nodes: &'a [FolderNode], path: &Path) -> Option<&'a FolderNode> {
        for node in nodes {
            if node.path == path && node.is_parent_nav {
                return Some(node);
            }
            if let Some(found) = find_parent_nav_node(&node.children, path) {
                return Some(found);
            }
        }
        None
    }

    #[test]
    fn test_toggle_expand_disambiguates_duplicate_paths_by_index() {
        // Regression: the current root and a chain node (pinned breadcrumbs)
        // can share the same path. Path-only matching toggled the FIRST
        // match (the root) even when the user clicked the chain node's
        // chevron. The flat index from the view must select the clicked node.
        let path = PathBuf::from("/same/path");

        let mut tree = vec![
            FolderNode {
                path: path.clone(),
                name: "root".into(),
                children: vec![],
                is_current: true,
                is_expanded: true,
                is_parent_nav: false,
                ..FolderNode::default()
            },
            FolderNode {
                path: PathBuf::from("/pinned"),
                name: "pinned".into(),
                children: vec![FolderNode {
                    path: path.clone(),
                    name: "chain".into(),
                    children: vec![],
                    is_current: false,
                    is_expanded: false,
                    is_parent_nav: true,
                    ..FolderNode::default()
                }],
                is_current: false,
                is_expanded: true,
                is_parent_nav: false,
                ..FolderNode::default()
            },
        ];

        // Chain node is the second occurrence -> flat index 2.
        let chain_idx = flat_index_of_nth(&tree, &path, 1).unwrap();
        assert_eq!(chain_idx, 2);
        {
            let mut running = 0;
            toggle_expand_recursive(&mut tree, &path, chain_idx, &mut running, None)
        };

        // The chain node toggled (false -> true), the root did NOT. With
        // path-only first-match matching the root would have collapsed.
        assert!(find_parent_nav_node(&tree, &path).unwrap().is_expanded);
        assert!(
            tree[0].is_expanded,
            "the root must keep its own expansion state"
        );
    }

    #[test]
    fn test_restored_expanded_chain_node_gets_real_children() {
        // Regression: after a rebuild, previously-expanded chain nodes were
        // restored with only their nested breadcrumb chain (or nothing) —
        // the real children were only built on a manual collapse+expand,
        // which is impossible when the node has no chevron.
        let base =
            std::env::temp_dir().join(format!("mediasort_chain_restore_{}", std::process::id()));
        let root = base.join("current");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(root.join("alpha")).unwrap();
        std::fs::create_dir_all(root.join("beta")).unwrap();

        // Mark the root's parent (an ancestor chain node) as expanded.
        let parent = base.clone();
        let mut expanded_paths = std::collections::HashSet::new();
        expanded_paths.insert(parent.clone());

        let tree = build_tree_nodes_data(&root, &[], &expanded_paths);

        let parent_nav = find_parent_nav_node(&tree, &parent).expect("chain node exists");
        assert!(
            parent_nav.is_expanded,
            "restored chain node must stay expanded"
        );
        let names: Vec<_> = parent_nav
            .children
            .iter()
            .filter(|c| !c.is_parent_nav)
            .map(|c| c.name.as_str())
            .collect();
        assert!(
            names.contains(&"current"),
            "restored-expanded chain node must show its real children, got {names:?}"
        );

        std::fs::remove_dir_all(&base).ok();
    }

    #[cfg(unix)]
    #[test]
    fn test_symlinked_pinned_chain_full_listing_after_reopen() {
        use std::os::unix::fs::symlink;

        let base =
            std::env::temp_dir().join(format!("mediasort_chain_symlink_{}", std::process::id()));
        let home = base.join("home").join("luca");
        let real_nextcloud = base.join("data").join("luca").join("Nextcloud");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&real_nextcloud).unwrap();
        std::fs::create_dir_all(home.join("development")).unwrap();
        std::fs::create_dir_all(home.join("other_stuff")).unwrap();
        symlink(&real_nextcloud, home.join("Nextcloud")).unwrap();

        let mut state = AppState::new(SettingsStore::default());
        state.pin_folder(&home.join("Nextcloud"));
        state.open_folder(&home);
        state.build_folder_tree();

        // Expand the pinned root, then every parent-nav (chain) node in the
        // tree at its actual flat index. On Linux the root and the pinned
        // chain share ancestor paths (e.g. /tmp/...); on macOS they do not
        // (open_folder canonicalizes /var to /private/var while the pinned
        // chain keeps the lexical paths), so path-based expansion must not
        // assume which occurrence belongs to the pinned chain.
        let pinned_idx =
            flat_index_of_nth(&state.folder.folder_tree, &home.join("Nextcloud"), 0).unwrap();
        state.toggle_folder_expand(&home.join("Nextcloud"), pinned_idx);

        let mut chain_nodes: Vec<(PathBuf, usize)> = Vec::new();
        collect_parent_nav_indices(&state.folder.folder_tree, &mut 0, &mut chain_nodes);
        // Descending index order: rebuilding a node only shifts the flat
        // indices of nodes AFTER it, so earlier entries stay valid.
        chain_nodes.sort_by_key(|b| std::cmp::Reverse(b.1));
        for (path, idx) in chain_nodes {
            state.toggle_folder_expand(&path, idx);
        }

        let chain_node = find_parent_nav_node(&state.folder.folder_tree, &home)
            .expect("pinned chain /home/luca node exists");
        let names: Vec<_> = chain_node
            .children
            .iter()
            .filter(|c| !c.is_parent_nav)
            .map(|c| c.name.as_str())
            .collect();
        for expected in ["Nextcloud", "development", "other_stuff"] {
            assert!(
                names.contains(&expected),
                "chain node of {home:?} missing {expected}, got {names:?}"
            );
        }

        // Re-open /home/luca as the current folder: the pinned chain node is
        // restored expanded and must keep its real children (regression:
        // it used to be restored with only the nested chain / no children).
        state.open_folder(&home);
        state.build_folder_tree();
        let chain_node = find_parent_nav_node(&state.folder.folder_tree, &home)
            .expect("pinned chain /home/luca node exists after reopen");
        assert!(chain_node.is_expanded);
        let names: Vec<_> = chain_node
            .children
            .iter()
            .filter(|c| !c.is_parent_nav)
            .map(|c| c.name.as_str())
            .collect();
        for expected in ["Nextcloud", "development", "other_stuff"] {
            assert!(
                names.contains(&expected),
                "restored chain node of {home:?} missing {expected}, got {names:?}"
            );
        }

        std::fs::remove_dir_all(&base).ok();
    }

    #[test]
    fn test_build_parent_chain_linear_structure() {
        let deep_path = PathBuf::from("/a/b/c/d");
        let chain = build_parent_chain(&deep_path);

        assert_eq!(chain.len(), 1);
        assert!(chain[0].is_parent_nav);

        let mut current = &chain[0];
        let expected = ["/a/b/c", "/a/b", "/a", "/"];
        for exp in &expected {
            assert_eq!(current.path, PathBuf::from(exp), "at path {exp}");
            if current.children.len() == 1 {
                current = &current.children[0];
            }
        }
        assert!(current.children.is_empty());
    }
}
