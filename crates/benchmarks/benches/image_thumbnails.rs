use std::path::PathBuf;

use benchmarks::variants;

fn fixtures() -> Vec<PathBuf> {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../resources/MockState");
    vec![
        PathBuf::from(root).join("mock 1.jpg"),
        PathBuf::from(root).join("mock 2.jpg"),
        PathBuf::from(root).join("mock 4.jpg"),
        PathBuf::from(root).join("mock 5.png"),
        PathBuf::from(root).join("mock 6.jpg"),
        PathBuf::from(root).join("mock 7.jpg"),
        PathBuf::from(root).join("mock 8.jpg"),
    ]
}

const FIXTURE_PATHS: &[&str] = &[
    "mock 1.jpg",
    "mock 2.jpg",
    "mock 4.jpg",
    "mock 5.png",
    "mock 6.jpg",
    "mock 7.jpg",
    "mock 8.jpg",
];

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/MockState"
    ))
    .join(name)
}

#[divan::bench(args = FIXTURE_PATHS)]
fn baseline_gui(name: &str) -> (u32, u32, Vec<u8>) {
    let path = fixture(name);
    divan::black_box(variants::baseline_gui(&path).unwrap())
}

#[divan::bench(args = FIXTURE_PATHS)]
fn backend_fir(name: &str) -> (u32, u32, Vec<u8>) {
    let path = fixture(name);
    divan::black_box(variants::backend_fir(&path).unwrap())
}

#[divan::bench(args = FIXTURE_PATHS)]
fn fir_no_probe(name: &str) -> (u32, u32, Vec<u8>) {
    let path = fixture(name);
    divan::black_box(variants::fir_no_probe(&path).unwrap())
}

#[divan::bench(args = FIXTURE_PATHS)]
fn no_exif_reread(name: &str) -> (u32, u32, Vec<u8>) {
    let path = fixture(name);
    divan::black_box(variants::no_exif_reread(&path).unwrap())
}

#[divan::bench(args = FIXTURE_PATHS)]
fn zune_decode(name: &str) -> (u32, u32, Vec<u8>) {
    let path = fixture(name);
    divan::black_box(variants::zune_decode(&path).unwrap())
}

#[divan::bench(args = FIXTURE_PATHS)]
fn zune_decode_no_orient(name: &str) -> (u32, u32, Vec<u8>) {
    let path = fixture(name);
    divan::black_box(variants::zune_decode_no_orient(&path).unwrap())
}

#[divan::bench(args = FIXTURE_PATHS)]
fn turbojpeg_scaled(name: &str) -> (u32, u32, Vec<u8>) {
    let path = fixture(name);
    divan::black_box(variants::turbojpeg_scaled(&path).unwrap())
}

/// Throughput-style: process ALL image fixtures sequentially.
#[divan::bench]
fn folder_sweep_baseline_gui() -> usize {
    let paths = fixtures();
    let mut total_bytes = 0usize;
    for path in &paths {
        let (_w, _h, rgba) = divan::black_box(variants::baseline_gui(path).unwrap());
        total_bytes += rgba.len();
    }
    total_bytes
}

#[divan::bench]
fn folder_sweep_zune_decode() -> usize {
    let paths = fixtures();
    let mut total_bytes = 0usize;
    for path in &paths {
        let (_w, _h, rgba) = divan::black_box(variants::zune_decode(path).unwrap());
        total_bytes += rgba.len();
    }
    total_bytes
}

#[divan::bench]
fn folder_sweep_turbojpeg() -> usize {
    let paths = fixtures();
    let mut total_bytes = 0usize;
    for path in &paths {
        let (_w, _h, rgba) = divan::black_box(variants::turbojpeg_scaled(path).unwrap());
        total_bytes += rgba.len();
    }
    total_bytes
}

fn main() {
    divan::main();
}
