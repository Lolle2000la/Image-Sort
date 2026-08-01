use std::path::{Path, PathBuf};

use media_sort_core::actions::delete_action::TrashRestoreHandle;
use media_sort_core::actions::reversible::ActionError;

#[cfg(target_os = "macos")]
fn macos_trash_item(path: &Path) -> Result<PathBuf, ActionError> {
    use objc2::rc::Retained;
    use objc2_foundation::{NSError, NSFileManager, NSString, NSURL};

    let path_str = path
        .to_str()
        .ok_or_else(|| ActionError::Io(std::io::Error::other("Invalid path encoding")))?;

    // SAFETY: We are calling Objective-C FFI methods through the `objc2` and
    // `objc2_foundation` runtimes. All pointers are checked for nullability
    // before dereferencing or retaining them, and the runtime handles the
    // reference counting correctly once retained.
    unsafe {
        let file_manager = NSFileManager::defaultManager();
        let ns_str = NSString::from_str(path_str);
        let url = NSURL::fileURLWithPath(&ns_str);

        let mut resulting_url: *mut NSURL = std::ptr::null_mut();
        let mut error: *mut NSError = std::ptr::null_mut();

        let success: bool = objc2::msg_send![
            &file_manager,
            trashItemAtURL: &*url,
            resultingItemURL: &mut resulting_url,
            error: &mut error
        ];

        if !success || resulting_url.is_null() {
            let err_msg = if !error.is_null() {
                let err = Retained::retain(error).expect("error pointer is checked to be non-null");
                err.localizedDescription().to_string()
            } else {
                "Cocoa trashItemAtURL failed".to_string()
            };
            return Err(ActionError::Io(std::io::Error::other(err_msg)));
        }

        let res_id =
            Retained::retain(resulting_url).expect("resulting_url is checked to be non-null");
        let path_nsstring = res_id
            .path()
            .expect("resulting trash URL must have a valid path");
        Ok(PathBuf::from(path_nsstring.to_string()))
    }
}

pub fn delete_to_trash(path: &Path) -> Result<Box<dyn TrashRestoreHandle>, ActionError> {
    let original_path = path.to_path_buf();

    #[cfg(not(target_os = "macos"))]
    {
        // Windows: callers may pass 8.3 short paths (e.g. TEMP resolves to
        // C:\Users\RUNNER~1\... on CI runners). Canonicalize while the file
        // still exists so the stored path matches what the trash metadata
        // records, stripping the \\?\ verbatim prefix canonicalize yields.
        #[cfg(target_os = "windows")]
        let original_path = {
            let canon = original_path.canonicalize().unwrap_or(original_path);
            match canon.to_string_lossy().strip_prefix(r"\\?\") {
                Some(stripped) => PathBuf::from(stripped.to_owned()),
                None => canon,
            }
        };

        trash::delete(&original_path)
            .map_err(|e| ActionError::Io(std::io::Error::other(e.to_string())))?;
        Ok(Box::new(NativeTrashRestore {
            original_path,
            flushed: false,
        }))
    }

    #[cfg(target_os = "macos")]
    {
        let trash_path = macos_trash_item(&original_path)?;
        Ok(Box::new(NativeTrashRestore {
            original_path,
            trash_path: Some(trash_path),
            flushed: false,
        }))
    }
}

/// Windows trash item matching: case-insensitive and tolerant of `\\?\`
/// verbatim prefixes, since the shell may report a different path form
/// than the caller passed (short 8.3 names, casing, verbatim paths).
#[cfg(target_os = "windows")]
fn windows_trash_paths_match(item_path: &Path, stored: &Path) -> bool {
    fn strip_verbatim(p: &Path) -> String {
        let s = p.to_string_lossy();
        s.strip_prefix(r"\\?\")
            .map(str::to_owned)
            .unwrap_or_else(|| s.into_owned())
    }
    strip_verbatim(item_path).eq_ignore_ascii_case(&strip_verbatim(stored))
}

struct NativeTrashRestore {
    original_path: PathBuf,
    #[cfg(target_os = "macos")]
    trash_path: Option<PathBuf>,
    flushed: bool,
}

impl TrashRestoreHandle for NativeTrashRestore {
    fn restore(&mut self) -> Result<(), ActionError> {
        if self.flushed {
            return Err(ActionError::RestorationFailed("already flushed".into()));
        }

        #[cfg(any(
            target_os = "windows",
            all(
                unix,
                not(target_os = "macos"),
                not(target_os = "ios"),
                not(target_os = "android")
            )
        ))]
        {
            let items = trash::os_limited::list()
                .map_err(|e| ActionError::Io(std::io::Error::other(e.to_string())))?;

            let item = items
                .into_iter()
                .find(|i| {
                    #[cfg(target_os = "windows")]
                    {
                        windows_trash_paths_match(&i.original_path(), &self.original_path)
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        i.original_path() == self.original_path
                    }
                })
                .ok_or_else(|| {
                    ActionError::RestorationFailed("item not found in system trash".into())
                })?;

            trash::os_limited::restore_all([item])
                .map_err(|e| ActionError::Io(std::io::Error::other(e.to_string())))?;

            self.flushed = true;
            Ok(())
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(ref trash_item_path) = self.trash_path {
                if !trash_item_path.exists() {
                    return Err(ActionError::RestorationFailed(
                        "Trash item no longer exists or trash was emptied".into(),
                    ));
                }
                if let Some(parent) = self.original_path.parent() {
                    std::fs::create_dir_all(parent).map_err(ActionError::Io)?;
                }

                std::fs::rename(trash_item_path, &self.original_path).map_err(ActionError::Io)?;

                self.flushed = true;
                Ok(())
            } else {
                Err(ActionError::RestorationFailed(
                    "Missing localized trash handle".into(),
                ))
            }
        }
    }

    fn flush_to_native_trash(&mut self) -> Result<(), ActionError> {
        self.flushed = true;
        Ok(())
    }
}

impl Drop for NativeTrashRestore {
    fn drop(&mut self) {
        if !self.flushed {
            let _ = self.flush_to_native_trash();
        }
    }
}
