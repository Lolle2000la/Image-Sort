fn main() {
    #[cfg(target_os = "windows")]
    setup_mpv_windows();
}

#[cfg(target_os = "windows")]
fn setup_mpv_windows() {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_dir = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest_dir);
    let mpv_vendor_dir = workspace_dir.join("target").join("mpv-win64");

    let lib_a = mpv_vendor_dir.join("libmpv.dll.a");
    let dll = mpv_vendor_dir.join("libmpv-2.dll");

    if !lib_a.exists() || !dll.exists() {
        let _ = fs::create_dir_all(&mpv_vendor_dir);
        let archive_path = mpv_vendor_dir.join("mpv-dev.7z");

        let download_url = "https://github.com/shinchiro/mpv-winbuild-cmake/releases/download/20260810/mpv-dev-x86_64-20260810-git-513d3407d4.7z";

        let status = Command::new("curl.exe")
            .args(&["-L", "-o", archive_path.to_str().unwrap(), download_url])
            .status();

        if let Ok(s) = status {
            if s.success() {
                let _ = Command::new("tar.exe")
                    .args(&[
                        "-xf",
                        archive_path.to_str().unwrap(),
                        "-C",
                        mpv_vendor_dir.to_str().unwrap(),
                    ])
                    .status();
            }
        }
    }

    if mpv_vendor_dir.exists() {
        if lib_a.exists() {
            let _ = fs::copy(&lib_a, mpv_vendor_dir.join("mpv.lib"));
            let _ = fs::copy(&lib_a, mpv_vendor_dir.join("libmpv.lib"));
            let _ = fs::copy(&lib_a, mpv_vendor_dir.join("libmpv-2.lib"));
        }

        let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
        if let Some(profile_dir) = out_dir.ancestors().nth(3) {
            if dll.exists() {
                let _ = fs::copy(&dll, profile_dir.join("libmpv-2.dll"));
                let _ = fs::copy(&dll, profile_dir.join("mpv-1.dll"));
                let _ = fs::copy(&dll, profile_dir.join("mpv-2.dll"));
                let _ = fs::copy(&dll, profile_dir.join("mpv.dll"));
            }
        }

        println!(
            "cargo:rustc-link-search=native={}",
            mpv_vendor_dir.display()
        );
    }

    println!("cargo:rerun-if-changed=build.rs");
}
