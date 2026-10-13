use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DenyReason {
    NoOpenFolder,
    MixedItems,
    MultipleFolders,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum DragDropMode {
    #[default]
    None,
    Files {
        has_destination: bool,
    },
    SingleFolder,
    Denied(DenyReason),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DragZone {
    #[default]
    None,
    Copy,
    Move,
    Open,
    Pin,
    Denied,
}

#[derive(Debug, Clone, Default)]
pub struct DragDropState {
    pub hovering: bool,
    pub dragged_paths: Vec<PathBuf>,
    pub mode: DragDropMode,
    pub target_zone: DragZone,
    pub last_cursor_position: Option<iced::Point>,
}

impl DragDropState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_path(&mut self, path: PathBuf, has_open_folder: bool) {
        if !self.dragged_paths.contains(&path) {
            self.dragged_paths.push(path);
        }
        self.hovering = true;
        self.reevaluate_mode(has_open_folder);
    }

    pub fn reevaluate_mode(&mut self, has_open_folder: bool) {
        if self.dragged_paths.is_empty() {
            self.mode = DragDropMode::None;
            self.target_zone = DragZone::None;
            return;
        }

        let num_dirs = self.dragged_paths.iter().filter(|p| p.is_dir()).count();
        let num_files = self.dragged_paths.len() - num_dirs;

        if num_dirs > 0 && num_files > 0 {
            self.mode = DragDropMode::Denied(DenyReason::MixedItems);
            self.target_zone = DragZone::Denied;
        } else if num_dirs > 1 {
            self.mode = DragDropMode::Denied(DenyReason::MultipleFolders);
            self.target_zone = DragZone::Denied;
        } else if num_dirs == 1 {
            self.mode = DragDropMode::SingleFolder;
            if self.target_zone == DragZone::Denied || self.target_zone == DragZone::None {
                self.target_zone = DragZone::Open;
            }
        } else {
            // All items are files
            if !has_open_folder {
                self.mode = DragDropMode::Denied(DenyReason::NoOpenFolder);
                self.target_zone = DragZone::Denied;
            } else {
                self.mode = DragDropMode::Files {
                    has_destination: true,
                };
                if self.target_zone == DragZone::Denied || self.target_zone == DragZone::None {
                    self.target_zone = DragZone::Copy;
                }
            }
        }
    }

    pub fn update_cursor(&mut self, cursor_pos: iced::Point, window_size: (f32, f32)) {
        self.last_cursor_position = Some(cursor_pos);

        if !self.hovering {
            return;
        }

        match &self.mode {
            DragDropMode::Denied(_) => {
                self.target_zone = DragZone::Denied;
            }
            DragDropMode::Files { .. } => {
                let mid_x = window_size.0 / 2.0;
                if cursor_pos.x < mid_x {
                    self.target_zone = DragZone::Copy;
                } else {
                    self.target_zone = DragZone::Move;
                }
            }
            DragDropMode::SingleFolder => {
                let mid_x = window_size.0 / 2.0;
                if cursor_pos.x < mid_x {
                    self.target_zone = DragZone::Open;
                } else {
                    self.target_zone = DragZone::Pin;
                }
            }
            DragDropMode::None => {
                self.target_zone = DragZone::None;
            }
        }
    }

    pub fn reset(&mut self) {
        self.hovering = false;
        self.dragged_paths.clear();
        self.mode = DragDropMode::None;
        self.target_zone = DragZone::None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    fn temp_test_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("media_sort_drag_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        dir
    }

    #[test]
    fn test_single_file_with_destination() {
        let tmp = temp_test_dir();
        let file_path = tmp.join("file1.txt");
        File::create(&file_path).unwrap();

        let mut state = DragDropState::new();
        state.add_path(file_path.clone(), true);

        assert!(state.hovering);
        assert_eq!(
            state.mode,
            DragDropMode::Files {
                has_destination: true
            }
        );
        assert_eq!(state.target_zone, DragZone::Copy);

        let window_size = (1000.0, 800.0);
        state.update_cursor(iced::Point::new(250.0, 400.0), window_size);
        assert_eq!(state.target_zone, DragZone::Copy);

        state.update_cursor(iced::Point::new(750.0, 400.0), window_size);
        assert_eq!(state.target_zone, DragZone::Move);
    }

    #[test]
    fn test_file_without_destination_denied() {
        let tmp = temp_test_dir();
        let file_path = tmp.join("file1.txt");
        File::create(&file_path).unwrap();

        let mut state = DragDropState::new();
        state.add_path(file_path, false);

        assert_eq!(state.mode, DragDropMode::Denied(DenyReason::NoOpenFolder));
        assert_eq!(state.target_zone, DragZone::Denied);
    }

    #[test]
    fn test_single_folder_drag() {
        let tmp = temp_test_dir();
        let folder_path = tmp.join("subfolder");
        std::fs::create_dir_all(&folder_path).unwrap();

        let mut state = DragDropState::new();
        state.add_path(folder_path, true);

        assert_eq!(state.mode, DragDropMode::SingleFolder);
        assert_eq!(state.target_zone, DragZone::Open);

        let window_size = (1000.0, 800.0);
        state.update_cursor(iced::Point::new(750.0, 400.0), window_size);
        assert_eq!(state.target_zone, DragZone::Pin);
    }

    #[test]
    fn test_multiple_folders_denied() {
        let tmp = temp_test_dir();
        let f1 = tmp.join("sub1");
        let f2 = tmp.join("sub2");
        std::fs::create_dir_all(&f1).unwrap();
        std::fs::create_dir_all(&f2).unwrap();

        let mut state = DragDropState::new();
        state.add_path(f1, true);
        state.add_path(f2, true);

        assert_eq!(
            state.mode,
            DragDropMode::Denied(DenyReason::MultipleFolders)
        );
        assert_eq!(state.target_zone, DragZone::Denied);
    }

    #[test]
    fn test_mixed_items_denied() {
        let tmp = temp_test_dir();
        let f1 = tmp.join("sub1");
        let file1 = tmp.join("file1.txt");
        std::fs::create_dir_all(&f1).unwrap();
        File::create(&file1).unwrap();

        let mut state = DragDropState::new();
        state.add_path(f1, true);
        state.add_path(file1, true);

        assert_eq!(state.mode, DragDropMode::Denied(DenyReason::MixedItems));
        assert_eq!(state.target_zone, DragZone::Denied);
    }
}
