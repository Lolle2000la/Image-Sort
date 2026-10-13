use iced::Element;
use iced::widget::text;

use media_sort_core::models::{FolderKind, FolderNode};

use crate::message::Message;

fn icon_element(icon: lucide_icons::Icon) -> Element<'static, Message> {
    text(char::from(icon))
        .font(iced::Font::with_name("lucide"))
        .size(16)
        .into()
}

/// The icon for a tree node based on its path and [`FolderKind`]. The view
/// overrides this for parent-navigation (chain) nodes and pinned roots.
pub fn icon_for(node: &FolderNode) -> lucide_icons::Icon {
    // Root folders (`/` on unix, drive roots on Windows) always get the root
    // icon, regardless of their kind.
    if node.path.parent().is_none() {
        return lucide_icons::Icon::FolderRoot;
    }
    match node.kind {
        // Priority order mirrors `inspect_folder`: Locked > Symlink > Git.
        FolderKind::Locked => lucide_icons::Icon::FolderLock,
        FolderKind::Symlink => lucide_icons::Icon::FolderSymlink,
        FolderKind::Git => lucide_icons::Icon::FolderGit2,
        FolderKind::Default => {
            if node.is_expanded && !node.children.is_empty() {
                lucide_icons::Icon::FolderOpen
            } else {
                lucide_icons::Icon::Folder
            }
        }
    }
}

pub fn node_icon(node: &FolderNode) -> Element<'static, Message> {
    icon_element(icon_for(node))
}

pub fn bookmark_icon() -> Element<'static, Message> {
    icon_element(lucide_icons::Icon::FolderBookmark)
}

pub fn root_icon() -> Element<'static, Message> {
    icon_element(lucide_icons::Icon::FolderRoot)
}

pub fn arrow_up_icon() -> Element<'static, Message> {
    icon_element(lucide_icons::Icon::ArrowUp)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn node(kind: FolderKind) -> FolderNode {
        FolderNode {
            path: PathBuf::from("/some/folder"),
            name: "folder".into(),
            kind,
            ..FolderNode::default()
        }
    }

    // `lucide_icons::Icon` has no PartialEq; compare the rendered glyphs.
    fn assert_icon(node: &FolderNode, expected: lucide_icons::Icon) {
        assert_eq!(char::from(icon_for(node)), char::from(expected));
    }

    #[test]
    fn test_icon_for_root_path() {
        let mut root = node(FolderKind::Default);
        root.path = PathBuf::from("/");
        assert_icon(&root, lucide_icons::Icon::FolderRoot);
    }

    #[test]
    fn test_icon_for_locked() {
        assert_icon(&node(FolderKind::Locked), lucide_icons::Icon::FolderLock);
    }

    #[test]
    fn test_icon_for_symlink() {
        assert_icon(
            &node(FolderKind::Symlink),
            lucide_icons::Icon::FolderSymlink,
        );
    }

    #[test]
    fn test_icon_for_git() {
        assert_icon(&node(FolderKind::Git), lucide_icons::Icon::FolderGit2);
    }

    #[test]
    fn test_icon_for_expanded_with_children() {
        let mut n = node(FolderKind::Default);
        n.is_expanded = true;
        n.children = vec![FolderNode::default()];
        assert_icon(&n, lucide_icons::Icon::FolderOpen);
    }

    #[test]
    fn test_icon_for_plain_folder() {
        assert_icon(&node(FolderKind::Default), lucide_icons::Icon::Folder);
    }
}
