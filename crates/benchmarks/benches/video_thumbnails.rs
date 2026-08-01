use std::path::PathBuf;

use benchmarks::variants;

fn fixture() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../resources/MockState"
    ))
    .join("mock 3.mp4")
}

thread_local! {
    static PLAYER: std::cell::RefCell<media_sort_backend::media::mpv_context::MpvContext> =
        std::cell::RefCell::new(
            media_sort_backend::media::mpv_context::MpvContext::new_thumbnail_player()
                .expect("failed to create mpv context")
        );
}

#[divan::bench(sample_count = 10)]
fn baseline_poll_10ms() -> (u32, u32, Vec<u8>) {
    PLAYER.with(|cell| {
        let mut player = cell.borrow_mut();
        divan::black_box(variants::baseline_poll_10ms(&mut player, &fixture()).unwrap())
    })
}

#[divan::bench(sample_count = 10)]
fn poll_1ms() -> (u32, u32, Vec<u8>) {
    PLAYER.with(|cell| {
        let mut player = cell.borrow_mut();
        divan::black_box(variants::poll_1ms(&mut player, &fixture()).unwrap())
    })
}

#[divan::bench(sample_count = 10)]
fn poll_0ms_spin() -> (u32, u32, Vec<u8>) {
    PLAYER.with(|cell| {
        let mut player = cell.borrow_mut();
        divan::black_box(variants::poll_0ms_spin(&mut player, &fixture()).unwrap())
    })
}

#[divan::bench(sample_count = 10)]
fn seek_10pct() -> (u32, u32, Vec<u8>) {
    PLAYER.with(|cell| {
        let mut player = cell.borrow_mut();
        divan::black_box(variants::seek_10pct(&mut player, &fixture()).unwrap())
    })
}

#[divan::bench(sample_count = 10)]
fn ffmpeg_extract() -> (u32, u32, Vec<u8>) {
    PLAYER.with(|cell| {
        let mut player = cell.borrow_mut();
        divan::black_box(variants::ffmpeg_extract(&mut player, &fixture()).unwrap())
    })
}

fn main() {
    divan::main();
}
