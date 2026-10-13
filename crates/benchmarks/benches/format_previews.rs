use std::path::Path;
use std::sync::OnceLock;

use benchmarks::fixture_gen::{self, BenchFormat, fixture_path};
use benchmarks::variants;
use image::GenericImageView;

static FIXTURES_READY: OnceLock<()> = OnceLock::new();

fn init_fixtures() {
    FIXTURES_READY.get_or_init(|| {
        fixture_gen::ensure_all_fixtures();
    });
}

fn preview_baseline_full(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let img = image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    let (w, h) = img.dimensions();
    let rgba = img.to_rgba8().into_raw();
    Some((w, h, rgba))
}

fn preview_fir_downscale(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let rgba = img.to_rgba8();
    let (src_w, src_h) = rgba.dimensions();
    let (dst_w, dst_h) =
        variants::calculate_thumbnail_dimensions_for_test(src_w, src_h, 1920, 1440);
    let resized =
        variants::resize_rgba_for_test(&rgba.into_raw(), src_w, src_h, dst_w, dst_h).ok()?;
    Some((dst_w, dst_h, resized))
}

fn preview_webp_libwebp_full(path: &Path) -> (u32, u32, Vec<u8>) {
    let data = std::fs::read(path).unwrap();
    let img = webp::Decoder::new(&data).decode().unwrap();
    (img.width(), img.height(), img.to_vec())
}

fn preview_webp_libwebp_fir(path: &Path) -> (u32, u32, Vec<u8>) {
    let (src_w, src_h, rgba) = preview_webp_libwebp_full(path);
    let (dst_w, dst_h) =
        variants::calculate_thumbnail_dimensions_for_test(src_w, src_h, 1920, 1440);
    let resized = variants::resize_rgba_for_test(&rgba, src_w, src_h, dst_w, dst_h).unwrap();
    (dst_w, dst_h, resized)
}

fn preview_avif_ffmpeg(path: &Path) -> (u32, u32, Vec<u8>) {
    let ff = std::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args([
            "-frames:v",
            "1",
            "-vf",
            "scale='min(1920,iw)':'min(1440,ih)':force_original_aspect_ratio=decrease,setsar=1",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgba",
            "-",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .output()
        .unwrap();
    let rgba = ff.stdout;
    let probe = std::process::Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height",
            "-of",
            "csv=p=0",
        ])
        .arg(path)
        .output()
        .unwrap_or_else(|_| std::process::Output {
            stdout: b"5260,3511".to_vec(),
            stderr: vec![],
            status: Default::default(),
        });
    let ps = String::from_utf8_lossy(&probe.stdout);
    let mut parts = ps.trim().split(',');
    let sw: u32 = parts
        .next()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(5260);
    let sh: u32 = parts
        .next()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(3511);
    let (ow, oh) = if sw as f64 / sh as f64 > 1920.0 / 1440.0 {
        (1920u32, (1920 * sh / sw).max(1))
    } else {
        ((1440 * sw / sh).max(1), 1440u32)
    };
    let len = (ow * oh * 4) as usize;
    (ow, oh, rgba[..len.min(rgba.len())].to_vec())
}

// ── Divan benches ─────────────────────────────────────────────────────

const ALL_FORMATS: &[&str] = &[
    "Avif", "Bmp", "Exr", "Farbfeld", "Gif", "Hdr", "Ico", "Jpeg", "Png", "Pnm", "Qoi", "Tga",
    "Tiff", "WebP",
];

fn pick_fixture(name: &str) -> std::path::PathBuf {
    let fmt = match name {
        "Avif" => BenchFormat::Avif,
        "Bmp" => BenchFormat::Bmp,
        "Exr" => BenchFormat::Exr,
        "Farbfeld" => BenchFormat::Farbfeld,
        "Gif" => BenchFormat::Gif,
        "Hdr" => BenchFormat::Hdr,
        "Ico" => BenchFormat::Ico,
        "Jpeg" => BenchFormat::Jpeg,
        "Png" => BenchFormat::Png,
        "Pnm" => BenchFormat::Pnm,
        "Qoi" => BenchFormat::Qoi,
        "Tga" => BenchFormat::Tga,
        "Tiff" => BenchFormat::Tiff,
        "WebP" => BenchFormat::WebP,
        _ => unreachable!(),
    };
    fixture_path(fmt)
}

#[divan::bench(args = ALL_FORMATS, sample_count = 10)]
fn baseline_full(name: &str) -> Option<(u32, u32, Vec<u8>)> {
    init_fixtures();
    let p = pick_fixture(name);
    Some(divan::black_box(preview_baseline_full(&p)?))
}

#[divan::bench(args = ALL_FORMATS, sample_count = 10)]
fn fir_downscale(name: &str) -> Option<(u32, u32, Vec<u8>)> {
    init_fixtures();
    let p = pick_fixture(name);
    Some(divan::black_box(preview_fir_downscale(&p)?))
}

#[divan::bench(sample_count = 10)]
fn webp_libwebp_full() -> (u32, u32, Vec<u8>) {
    init_fixtures();
    let p = fixture_path(BenchFormat::WebP);
    divan::black_box(preview_webp_libwebp_full(&p))
}

#[divan::bench(sample_count = 10)]
fn webp_libwebp_fir() -> (u32, u32, Vec<u8>) {
    init_fixtures();
    let p = fixture_path(BenchFormat::WebP);
    divan::black_box(preview_webp_libwebp_fir(&p))
}

#[divan::bench(sample_count = 5)]
fn avif_ffmpeg_preview() -> (u32, u32, Vec<u8>) {
    init_fixtures();
    let p = fixture_path(BenchFormat::Avif);
    divan::black_box(preview_avif_ffmpeg(&p))
}

fn main() {
    divan::main();
}
