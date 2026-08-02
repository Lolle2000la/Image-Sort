//! Performance-improvement candidate benchmarks. Each group compares the
//! current production approach against a proposed optimization, isolating the
//! change so we can measure whether it's worth adopting.
//!
//! Groups:
//!   A_fine_ffmpeg_path   — cache resolved ffmpeg path (no -version verify per call)
//!   B_resize_rgba        — eliminate src.to_vec() copy; reuse thread_local Resizer
//!   C_image_dimensions   — header-only read instead of full pixel decode
//!   D_detect_media_type  — HashMap + ASCII-lowercase stack buf vs 3 linear scans
//!   E_filtered_entries    — pre-lowercased names cache vs re-lowercase per call
//!   F_settings_save      — save() cost vs serialize-only vs no-op dirty check

use std::path::PathBuf;
use std::sync::OnceLock;

use benchmarks::perf_variants;

fn mock_state_root() -> &'static PathBuf {
    static ROOT: OnceLock<PathBuf> = OnceLock::new();
    ROOT.get_or_init(|| {
        PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../resources/MockState"
        ))
    })
}

fn jpeg_fixture() -> PathBuf {
    mock_state_root().join("mock 1.jpg")
}

fn mp4_fixture() -> PathBuf {
    mock_state_root().join("mock 3.mp4")
}

// ─── Group A: cached ffmpeg path ──────────────────────────────────────

#[divan::bench(sample_count = 10, sample_size = 1)]
fn a_find_ffmpeg_baseline(bencher: divan::Bencher) {
    if perf_variants::ffmpeg_path_baseline().is_none() {
        return;
    }
    bencher.bench(|| {
        let p = perf_variants::ffmpeg_path_baseline();
        divan::black_box(p);
    });
}

#[divan::bench(sample_count = 10, sample_size = 1)]
fn a_find_ffmpeg_cached(bencher: divan::Bencher) {
    if perf_variants::ffmpeg_path_baseline().is_none() {
        return;
    }
    bencher.bench(|| {
        let p = perf_variants::ffmpeg_path_cached();
        divan::black_box(p);
    });
}

#[divan::bench(sample_count = 5, sample_size = 1)]
fn a_extract_frame_baseline(bencher: divan::Bencher) {
    let mp4 = mp4_fixture();
    if !mp4.exists() || perf_variants::ffmpeg_path_baseline().is_none() {
        return;
    }
    bencher.bench(|| {
        let r = perf_variants::extract_frame_baseline(&mp4, 128, 128);
        if let Ok(img) = r {
            divan::black_box((img.width, img.height));
        }
    });
}

#[divan::bench(sample_count = 5, sample_size = 1)]
fn a_extract_frame_cached(bencher: divan::Bencher) {
    let mp4 = mp4_fixture();
    if !mp4.exists() || perf_variants::ffmpeg_path_baseline().is_none() {
        return;
    }
    bencher.bench(|| {
        let r = perf_variants::extract_frame_cached_ffmpeg(&mp4, 128, 128);
        if let Ok(img) = r {
            divan::black_box((img.width, img.height));
        }
    });
}

// ─── Group B: resize_rgba variants (no I/O) ──────────────────────────

fn synthetic_src() -> (Vec<u8>, u32, u32) {
    static SRC: OnceLock<(Vec<u8>, u32, u32)> = OnceLock::new();
    SRC.get_or_init(|| {
        let (w, h) = (4000u32, 3000u32);
        (perf_variants::synthetic_rgba(w, h), w, h)
    })
    .clone()
}

#[divan::bench]
fn b_resize_rgba_baseline(bencher: divan::Bencher) {
    let (src, w, h) = synthetic_src();
    bencher.bench(|| {
        let r = perf_variants::resize_rgba_baseline(divan::black_box(&src), w, h, 128, 96);
        divan::black_box(r.unwrap());
    });
}

#[divan::bench]
fn b_resize_rgba_owned_no_copy(bencher: divan::Bencher) {
    let (src, w, h) = synthetic_src();
    bencher.bench_local(|| {
        let src_owned = src.clone();
        let r =
            perf_variants::resize_rgba_owned_no_copy(divan::black_box(src_owned), w, h, 128, 96);
        divan::black_box(r.unwrap());
    });
}

#[divan::bench]
fn b_resize_rgba_baseline_alloc_owner(bencher: divan::Bencher) {
    let (src, w, h) = synthetic_src();
    bencher.bench_local(|| {
        let src_owned = src.clone();
        let r = perf_variants::resize_rgba_baseline(divan::black_box(&src_owned), w, h, 128, 96);
        divan::black_box(r.unwrap());
    });
}

#[divan::bench]
fn b_resize_rgba_thread_local_resizer(bencher: divan::Bencher) {
    let (src, w, h) = synthetic_src();
    bencher.bench(|| {
        let r =
            perf_variants::resize_rgba_thread_local_resizer(divan::black_box(&src), w, h, 128, 96);
        divan::black_box(r.unwrap());
    });
}

// ─── Group C: image dimensions ────────────────────────────────────────

#[divan::bench]
fn c_dims_baseline_full_decode(bencher: divan::Bencher) {
    let path = jpeg_fixture();
    if !path.exists() {
        return;
    }
    bencher.bench(|| {
        let d = perf_variants::dims_baseline_full_decode(divan::black_box(&path));
        divan::black_box(d.unwrap());
    });
}

#[divan::bench]
fn c_dims_header_only(bencher: divan::Bencher) {
    let path = jpeg_fixture();
    if !path.exists() {
        return;
    }
    bencher.bench(|| {
        let d = perf_variants::dims_header_only(divan::black_box(&path));
        divan::black_box(d.unwrap());
    });
}

#[divan::bench]
fn c_dims_turbojpeg_header(bencher: divan::Bencher) {
    let path = jpeg_fixture();
    if !path.exists() {
        return;
    }
    bencher.bench(|| {
        let d = perf_variants::dims_turbojpeg_header(divan::black_box(&path));
        divan::black_box(d.unwrap());
    });
}

// ─── Group D: detect_media_type lookup ────────────────────────────────

fn detect_paths() -> Vec<PathBuf> {
    let exts = [
        "jpg",
        "JPG",
        "png",
        "PNG",
        "jpeg",
        "gif",
        "GIF",
        "bmp",
        "tiff",
        "webp",
        "qoi",
        "tga",
        "avif",
        "mp4",
        "MP4",
        "mkv",
        "webm",
        "avi",
        "mov",
        "wmv",
        "flv",
        "m4v",
        "mp3",
        "Mp3",
        "flac",
        "wav",
        "aac",
        "m4a",
        "ogg",
        "opus",
        "unknown_ext",
        "no_extension_file",
        "very_long_extension_for_testing",
    ];
    exts.iter()
        .map(|e| PathBuf::from(format!("/synthetic/file.{}", e)))
        .collect()
}

#[divan::bench]
fn d_detect_media_type_baseline(bencher: divan::Bencher) {
    let paths = detect_paths();
    bencher.bench(|| {
        let mut acc: u32 = 0;
        for p in &paths {
            let t = perf_variants::detect_media_type_baseline(divan::black_box(p), true);
            acc += t as u32;
        }
        divan::black_box(acc);
    });
}

#[divan::bench]
fn d_detect_media_type_hashset(bencher: divan::Bencher) {
    let paths = detect_paths();
    bencher.bench(|| {
        let mut acc: u32 = 0;
        for p in &paths {
            let t = perf_variants::detect_media_type_hashset(divan::black_box(p), true);
            acc += t as u32;
        }
        divan::black_box(acc);
    });
}

// ─── Group E: filtered_entries memoization ────────────────────────────

fn grid_entries() -> Vec<media_sort_core::models::MediaEntry> {
    static GRID: OnceLock<Vec<media_sort_core::models::MediaEntry>> = OnceLock::new();
    GRID.get_or_init(|| perf_variants::synthetic_media_entries(5000))
        .clone()
}

fn lower_cache() -> Vec<String> {
    static LOWER: OnceLock<Vec<String>> = OnceLock::new();
    LOWER
        .get_or_init(|| {
            let entries = grid_entries();
            perf_variants::precompute_lower_names(&entries)
        })
        .clone()
}

#[divan::bench]
fn e_filtered_entries_baseline_no_query(bencher: divan::Bencher) {
    let entries = grid_entries();
    bencher.bench(|| {
        let v = perf_variants::filtered_entries_baseline(divan::black_box(&entries), "");
        divan::black_box(v.len());
    });
}

#[divan::bench]
fn e_filtered_entries_baseline_with_query(bencher: divan::Bencher) {
    let entries = grid_entries();
    bencher.bench(|| {
        let v = perf_variants::filtered_entries_baseline(divan::black_box(&entries), "dsc");
        divan::black_box(v.len());
    });
}

#[divan::bench]
fn e_filtered_entries_precomputed_lower(bencher: divan::Bencher) {
    let entries = grid_entries();
    let lower = lower_cache();
    bencher.bench(|| {
        let v = perf_variants::filtered_entries_precomputed_lower(
            divan::black_box(&entries),
            divan::black_box(&lower),
            "dsc",
        );
        divan::black_box(v.len());
    });
}

// ─── Group F: settings.save ────────────────────────────────────────────

fn settings_store_temp() -> media_sort_core::settings::store::SettingsStore {
    use media_sort_core::settings::store::SettingsStore;
    let dir = std::env::temp_dir().join(format!("media-sort-bench-perf-{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok();
    SettingsStore {
        custom_path: Some(dir.join("config.toml")),
        ..Default::default()
    }
}

#[divan::bench(sample_count = 50, sample_size = 1)]
fn f_settings_save_baseline(bencher: divan::Bencher) {
    let mut store = settings_store_temp();
    bencher.bench_local(|| {
        let _ = perf_variants::settings_save_baseline(divan::black_box(&mut store));
    });
}

#[divan::bench(sample_count = 50, sample_size = 1)]
fn f_settings_save_serialize_only(bencher: divan::Bencher) {
    let store = settings_store_temp();
    bencher.bench(|| {
        let s = perf_variants::settings_save_serialize_only(divan::black_box(&store)).unwrap();
        divan::black_box(s.len());
    });
}

#[divan::bench(sample_count = 50, sample_size = 1)]
fn f_settings_save_noop_if_clean(bencher: divan::Bencher) {
    bencher.bench(|| {
        let r = perf_variants::settings_save_noop_if_clean(divan::black_box(false));
        let _ = divan::black_box(r);
    });
}

fn main() {
    divan::main();
}
