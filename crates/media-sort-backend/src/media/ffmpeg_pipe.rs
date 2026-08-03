use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[cfg(target_os = "windows")]
const FFMPEG_EXE: &str = "ffmpeg.exe";
#[cfg(not(target_os = "windows"))]
const FFMPEG_EXE: &str = "ffmpeg";

static FFMPEG_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

fn verify_ffmpeg(ffmpeg_path: &Path) -> bool {
    std::process::Command::new(ffmpeg_path)
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

/// Locate a usable ffmpeg binary: next to the current executable first
/// (bundled releases), then PATH. The result is cached for the lifetime of
/// the process in `FFMPEG_PATH` — the cached lookup skips both the PATH scan
/// and the `ffmpeg -version` verify spawn on every subsequent call.
pub fn find_ffmpeg() -> Option<PathBuf> {
    FFMPEG_PATH.get_or_init(find_ffmpeg_uncached).clone()
}

fn find_ffmpeg_uncached() -> Option<PathBuf> {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let candidate = dir.join(FFMPEG_EXE);
        if candidate.exists() && verify_ffmpeg(&candidate) {
            return Some(candidate);
        }
    }

    let path_candidate = std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path).find_map(|dir| {
            let p = dir.join(FFMPEG_EXE);
            p.exists().then_some(p)
        })
    });

    if let Some(ref p) = path_candidate
        && verify_ffmpeg(p)
    {
        return path_candidate;
    }

    None
}

/// Extract the first video frame, scaled to fit max_w×max_h (aspect preserved),
/// as RGBA. Uses `-vf scale=W:H:force_original_aspect_ratio=decrease,setsar=1`
/// piped as PNG (`-f image2pipe -vcodec png -`), decoded via the image crate.
pub fn extract_frame(path: &Path, max_w: u32, max_h: u32) -> Result<super::DecodedImage, String> {
    let ffmpeg = find_ffmpeg().ok_or_else(|| "ffmpeg not found".to_string())?;

    let vf = format!(
        "scale={}:{}:force_original_aspect_ratio=decrease,setsar=1",
        max_w, max_h
    );

    let output = std::process::Command::new(&ffmpeg)
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args([
            "-frames:v",
            "1",
            "-vf",
            &vf,
            "-f",
            "image2pipe",
            "-vcodec",
            "png",
            "-",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .map_err(|e| format!("ffmpeg spawn: {e}"))?;

    let stderr_tail = || {
        let s = String::from_utf8_lossy(&output.stderr);
        s.trim().chars().take(500).collect::<String>()
    };
    if !output.status.success() {
        return Err(format!(
            "ffmpeg exited with {}: {}",
            output.status,
            stderr_tail()
        ));
    }

    let bytes = output.stdout;
    if bytes.is_empty() {
        return Err(format!("ffmpeg produced empty output: {}", stderr_tail()));
    }

    let img = decode_png_with_limits(&bytes)?;

    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    Ok(super::DecodedImage::new(w, h, rgba.into_raw()))
}

/// Decode PNG bytes from the ffmpeg pipe under the shared decode budget so a
/// hostile stream (e.g. a PNG bomb served by ffmpeg) cannot force oversized
/// allocations.
fn decode_png_with_limits(bytes: &[u8]) -> Result<image::DynamicImage, String> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes));
    reader.limits(super::image_decoder::image_decode_limits());
    reader.set_format(image::ImageFormat::Png);
    reader.decode().map_err(|e| format!("png decode: {e}"))
}
