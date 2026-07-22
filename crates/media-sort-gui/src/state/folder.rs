use std::fmt;
use std::path::PathBuf;
use std::sync::mpsc;

use media_sort_core::models::{FolderNode, PinnedFolder};

#[derive(Default)]
pub struct FolderState {
    pub current_folder: Option<PathBuf>,
    pub folder_tree: Vec<FolderNode>,
    pub pinned_folders: Vec<PinnedFolder>,
    pub selected_folder: Option<PathBuf>,
    pub(crate) selected_folder_idx: Option<usize>,
    pub dragging_folder_divider: bool,
    pub dragging_pinned_folder: Option<PathBuf>,
    pub hovered_pinned_folder: Option<PathBuf>,
    pub folder_tree_receiver: Option<mpsc::Receiver<Vec<FolderNode>>>,
}

impl fmt::Debug for FolderState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FolderState")
            .field("current_folder", &self.current_folder)
            .field("folder_tree_len", &self.folder_tree.len())
            .field("pinned_folders_len", &self.pinned_folders.len())
            .field("selected_folder", &self.selected_folder)
            .field("selected_folder_idx", &self.selected_folder_idx)
            .field("dragging_folder_divider", &self.dragging_folder_divider)
            .field("dragging_pinned_folder", &self.dragging_pinned_folder)
            .field("hovered_pinned_folder", &self.hovered_pinned_folder)
            .field("folder_tree_receiver", &self.folder_tree_receiver.is_some())
            .finish()
    }
}

impl FolderState {
    pub fn collect_visible_folders(&self) -> Vec<PathBuf> {
        let mut list = Vec::new();
        super::collect_visible_folders_recursive(&self.folder_tree, &mut list);
        list
    }

    pub fn set_selected(&mut self, path: PathBuf, idx: usize) {
        self.selected_folder = Some(path.clone());
        let visible = self.collect_visible_folders();
        if let Some(pos) = visible.iter().position(|p| p == &path) {
            self.selected_folder_idx = Some(pos);
        } else {
            self.selected_folder_idx = Some(idx);
        }
    }

    pub(crate) fn sync_selected_idx(&mut self) {
        if let Some(ref path) = self.selected_folder.clone() {
            let visible = self.collect_visible_folders();
            if let Some(old_idx) = self.selected_folder_idx {
                self.selected_folder_idx = visible
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| *p == path)
                    .min_by_key(|(i, _)| i.abs_diff(old_idx))
                    .map(|(i, _)| i);
            } else {
                self.selected_folder_idx = visible.iter().position(|p| p == path);
            }
        }
        if self.selected_folder.is_none() || self.selected_folder_idx.is_none() {
            let visible = self.collect_visible_folders();
            if let Some(first) = visible.into_iter().next() {
                self.selected_folder = Some(first);
                self.selected_folder_idx = Some(0);
            }
        }
    }

    pub fn select_below(&mut self) {
        let visible = self.collect_visible_folders();
        if visible.is_empty() {
            return;
        }
        let current_idx = self.selected_folder_idx.or_else(|| {
            self.selected_folder
                .as_ref()
                .and_then(|f| visible.iter().position(|p| p == f))
        });
        let next = current_idx.map(|i| i + 1).unwrap_or(0);
        if next < visible.len() {
            self.selected_folder = Some(visible[next].clone());
            self.selected_folder_idx = Some(next);
        }
    }

    pub fn select_above(&mut self) {
        let visible = self.collect_visible_folders();
        if visible.is_empty() {
            return;
        }
        let current_idx = self.selected_folder_idx.or_else(|| {
            self.selected_folder
                .as_ref()
                .and_then(|f| visible.iter().position(|p| p == f))
        });
        if let Some(idx) = current_idx
            && idx > 0
        {
            self.selected_folder = Some(visible[idx - 1].clone());
            self.selected_folder_idx = Some(idx - 1);
        }
    }

    pub fn expand_selected(&mut self) {
        let Some(selected) = self.selected_folder.clone() else {
            return;
        };
        if let Some(expanded) = super::find_node_expanded(&self.folder_tree, &selected) {
            if expanded {
                if let Some(first_child_path) =
                    super::first_visible_child(&self.folder_tree, &selected)
                {
                    let visible = self.collect_visible_folders();
                    let idx = self.selected_folder_idx.unwrap_or(0);
                    if let Some(child_idx) = visible
                        .iter()
                        .enumerate()
                        .skip(idx + 1)
                        .find(|(_, p)| *p == &first_child_path)
                        .map(|(i, _)| i)
                    {
                        self.selected_folder = Some(first_child_path);
                        self.selected_folder_idx = Some(child_idx);
                        return;
                    }
                }
            } else {
                super::set_expand_recursive(
                    &mut self.folder_tree,
                    &selected,
                    true,
                    self.current_folder.as_deref(),
                );
            }
        }
        self.sync_selected_idx();
    }

    pub fn collapse_selected(&mut self) {
        let Some(selected) = self.selected_folder.clone() else {
            return;
        };
        if let Some(expanded) = super::find_node_expanded(&self.folder_tree, &selected) {
            if expanded {
                super::set_expand_recursive(
                    &mut self.folder_tree,
                    &selected,
                    false,
                    self.current_folder.as_deref(),
                );
            } else if let Some(parent) = selected.parent()
                && super::find_node_expanded(&self.folder_tree, parent).is_some()
            {
                let visible = self.collect_visible_folders();
                if let Some(old_idx) = self.selected_folder_idx
                    && let Some(i) = visible[..old_idx.min(visible.len())]
                        .iter()
                        .rposition(|p| *p == parent)
                {
                    self.selected_folder = Some(parent.to_path_buf());
                    self.selected_folder_idx = Some(i);
                    return;
                }
                if let Some(pos) = visible.iter().position(|p| *p == parent) {
                    self.selected_folder = Some(parent.to_path_buf());
                    self.selected_folder_idx = Some(pos);
                }
            }
        }
        self.sync_selected_idx();
    }
}
