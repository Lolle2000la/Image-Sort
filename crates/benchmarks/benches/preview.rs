use std::path::PathBuf;

use benchmarks::variants;

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
fn baseline_full(name: &str) -> (u32, u32, Vec<u8>) {
    let path = fixture(name);
    divan::black_box(variants::baseline_full(&path).unwrap())
}

#[divan::bench(args = FIXTURE_PATHS)]
fn fir_downscale(name: &str) -> (u32, u32, Vec<u8>) {
    let path = fixture(name);
    divan::black_box(variants::fir_downscale(&path).unwrap())
}

#[divan::bench(args = FIXTURE_PATHS)]
fn zune_preview(name: &str) -> (u32, u32, Vec<u8>) {
    let path = fixture(name);
    divan::black_box(variants::zune_preview(&path).unwrap())
}

#[divan::bench(args = FIXTURE_PATHS)]
fn turbojpeg_preview(name: &str) -> (u32, u32, Vec<u8>) {
    let path = fixture(name);
    divan::black_box(variants::turbojpeg_preview(&path).unwrap())
}

fn main() {
    divan::main();
}
