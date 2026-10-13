use std::path::Path;
use std::sync::OnceLock;

use benchmarks::fixture_gen::{self, BenchFormat, fixture_path};
use benchmarks::variants;

static FIXTURES_READY: OnceLock<()> = OnceLock::new();

fn init_fixtures() {
    FIXTURES_READY.get_or_init(|| {
        fixture_gen::ensure_all_fixtures();
    });
}

fn thumb_baseline(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let img = image::ImageReader::open(path)
        .ok()?
        .with_guessed_format()
        .ok()?
        .decode()
        .ok()?;
    let thumbnail = img.thumbnail(128, 128).to_rgba8();
    let (w, h) = thumbnail.dimensions();
    Some((w, h, thumbnail.into_raw()))
}

fn thumb_fir(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    let bytes = std::fs::read(path).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let rgba = img.to_rgba8();
    let (src_w, src_h) = rgba.dimensions();
    let (dst_w, dst_h) = variants::calculate_thumbnail_dimensions_for_test(src_w, src_h, 128, 128);
    let resized =
        variants::resize_rgba_for_test(&rgba.into_raw(), src_w, src_h, dst_w, dst_h).ok()?;
    Some((dst_w, dst_h, resized))
}

fn thumb_ico_entry_decode(path: &Path) -> (u32, u32, Vec<u8>) {
    let file = std::fs::File::open(path).unwrap();
    let icon_dir = ico::IconDir::read(file).unwrap();
    let entry = icon_dir
        .entries()
        .iter()
        .filter(|e| e.width() <= 128 && e.height() <= 128)
        .max_by_key(|e| e.width())
        .or_else(|| icon_dir.entries().iter().max_by_key(|e| e.width()))
        .unwrap();
    let decoded = entry.decode().unwrap();
    (
        decoded.width(),
        decoded.height(),
        decoded.rgba_data().to_vec(),
    )
}

fn thumb_webp_libwebp_plain(path: &Path) -> (u32, u32, Vec<u8>) {
    let data = std::fs::read(path).unwrap();
    let img = webp::Decoder::new(&data).decode().unwrap();
    (img.width(), img.height(), img.to_vec())
}

fn thumb_webp_libwebp_thumbnail(path: &Path) -> (u32, u32, Vec<u8>) {
    let (src_w, src_h, rgba) = thumb_webp_libwebp_plain(path);
    let (dst_w, dst_h) = variants::calculate_thumbnail_dimensions_for_test(src_w, src_h, 128, 128);
    let resized = variants::resize_rgba_for_test(&rgba, src_w, src_h, dst_w, dst_h).unwrap();
    (dst_w, dst_h, resized)
}

fn thumb_avif_ffmpeg(path: &Path) -> (u32, u32, Vec<u8>) {
    let ff = std::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args([
            "-frames:v",
            "1",
            "-vf",
            "scale='min(128,iw)':'min(128,ih)':force_original_aspect_ratio=decrease,setsar=1",
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
    let (ow, oh) = if sw > sh {
        (128u32, (128 * sh / sw).max(1))
    } else {
        ((128 * sw / sh).max(1), 128u32)
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

#[divan::bench(args = ALL_FORMATS, sample_count = 15)]
fn baseline(name: &str) -> Option<(u32, u32, Vec<u8>)> {
    init_fixtures();
    let p = pick_fixture(name);
    Some(divan::black_box(thumb_baseline(&p)?))
}

#[divan::bench(args = ALL_FORMATS, sample_count = 15)]
fn fast_image_resize(name: &str) -> Option<(u32, u32, Vec<u8>)> {
    init_fixtures();
    let p = pick_fixture(name);
    Some(divan::black_box(thumb_fir(&p)?))
}

#[divan::bench(sample_count = 15)]
fn ico_entry_decode() -> (u32, u32, Vec<u8>) {
    init_fixtures();
    let p = fixture_path(BenchFormat::Ico);
    divan::black_box(thumb_ico_entry_decode(&p))
}

#[divan::bench(sample_count = 15)]
fn webp_libwebp_plain() -> (u32, u32, Vec<u8>) {
    init_fixtures();
    let p = fixture_path(BenchFormat::WebP);
    divan::black_box(thumb_webp_libwebp_plain(&p))
}

#[divan::bench(sample_count = 15)]
fn webp_libwebp_thumbnail() -> (u32, u32, Vec<u8>) {
    init_fixtures();
    let p = fixture_path(BenchFormat::WebP);
    divan::black_box(thumb_webp_libwebp_thumbnail(&p))
}

#[divan::bench(sample_count = 10)]
fn avif_ffmpeg_pipe() -> (u32, u32, Vec<u8>) {
    init_fixtures();
    let p = fixture_path(BenchFormat::Avif);
    divan::black_box(thumb_avif_ffmpeg(&p))
}

fn main() {
    divan::main();
}
