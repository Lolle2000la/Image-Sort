fn main() {
    #[cfg(all(target_os = "windows", feature = "winbuild"))]
    setup_mpv_windows();
}

#[allow(dead_code)]
const DEFAULT_MPV_WINBUILD_TAG: &str = "2026-08-08-dd5d17d328";

#[cfg(all(target_os = "windows", feature = "winbuild"))]
fn setup_mpv_windows() {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;

    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "x86_64".to_string());
    let mpv_arch = match target_arch.as_str() {
        "x86_64" => "x86_64",
        "x86" => "i686",
        "aarch64" => "aarch64",
        other => other,
    };

    let tag = env::var("MPV_WINBUILD_TAG").unwrap_or_else(|_| DEFAULT_MPV_WINBUILD_TAG.to_string());

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_dir = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .unwrap_or(&manifest_dir);
    let mpv_vendor_dir = workspace_dir
        .join("target")
        .join(format!("mpv-winbuild-{}", mpv_arch));

    let lib_a = mpv_vendor_dir.join("libmpv.dll.a");
    let dll = mpv_vendor_dir.join("libmpv-2.dll");
    let ffmpeg_exe = mpv_vendor_dir.join("ffmpeg.exe");

    if !lib_a.exists() || !dll.exists() || !ffmpeg_exe.exists() {
        let _ = fs::create_dir_all(&mpv_vendor_dir);

        let release_api_url = format!(
            "https://api.github.com/repos/Lolle2000la/mpv-winbuild/releases/tags/{}",
            tag
        );

        let json_output = Command::new("curl.exe")
            .args(&[
                "-sL",
                "-H",
                "User-Agent: Rust-Build-Script",
                &release_api_url,
            ])
            .output();

        let mut mpv_url = None;
        let mut ffmpeg_url = None;

        if let Ok(out) = json_output {
            if out.status.success() {
                let body = String::from_utf8_lossy(&out.stdout);
                for line in body.lines() {
                    if line.contains("browser_download_url") {
                        if let Some(url_start) = line.find("https://") {
                            let url = line[url_start..]
                                .trim_matches(|c| c == '"' || c == ',' || c == ' ');
                            if url.contains(&format!("mpv-dev-{}", mpv_arch))
                                && url.ends_with(".7z")
                                && !url.contains("-lgpl")
                                && !url.contains("-v3")
                            {
                                mpv_url = Some(url.to_string());
                            } else if url.contains(&format!("ffmpeg-{}", mpv_arch))
                                && url.ends_with(".7z")
                                && !url.contains("-lgpl")
                                && !url.contains("-v3")
                            {
                                ffmpeg_url = Some(url.to_string());
                            }
                        }
                    }
                }
            }
        }

        if let (Some(mpv_dl), Some(ffmpeg_dl)) = (mpv_url, ffmpeg_url) {
            let mpv_archive = mpv_vendor_dir.join("mpv-dev.7z");
            let ffmpeg_archive = mpv_vendor_dir.join("ffmpeg.7z");

            let _ = Command::new("curl.exe")
                .args(&["-fL", "-o", mpv_archive.to_str().unwrap(), &mpv_dl])
                .status();

            let _ = Command::new("curl.exe")
                .args(&["-fL", "-o", ffmpeg_archive.to_str().unwrap(), &ffmpeg_dl])
                .status();

            let has_7z = Command::new("7z.exe")
                .arg("--help")
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if has_7z {
                let _ = Command::new("7z.exe")
                    .args(&[
                        "e",
                        mpv_archive.to_str().unwrap(),
                        &format!("-o{}", mpv_vendor_dir.display()),
                        "-y",
                    ])
                    .status();
                let _ = Command::new("7z.exe")
                    .args(&[
                        "e",
                        ffmpeg_archive.to_str().unwrap(),
                        &format!("-o{}", mpv_vendor_dir.display()),
                        "-y",
                    ])
                    .status();
            } else {
                let _ = Command::new("tar.exe")
                    .args(&[
                        "-xf",
                        mpv_archive.to_str().unwrap(),
                        "-C",
                        mpv_vendor_dir.to_str().unwrap(),
                    ])
                    .status();
                let _ = Command::new("tar.exe")
                    .args(&[
                        "-xf",
                        ffmpeg_archive.to_str().unwrap(),
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
            if ffmpeg_exe.exists() {
                let _ = fs::copy(&ffmpeg_exe, profile_dir.join("ffmpeg.exe"));
            }
        }

        println!(
            "cargo:rustc-link-search=native={}",
            mpv_vendor_dir.display()
        );
    }

    println!("cargo:rerun-if-changed=build.rs");
}
