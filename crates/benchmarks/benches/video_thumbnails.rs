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
    static PLAYER: std::cell::RefCell<Option<mpv_utils::MpvContext>> =
        const { std::cell::RefCell::new(None) };
}

fn with_player<R>(f: impl FnOnce(&mut mpv_utils::MpvContext) -> R) -> Option<R> {
    PLAYER.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            match mpv_utils::MpvContext::new_thumbnail_player() {
                Ok(p) => *slot = Some(p),
                Err(e) => {
                    eprintln!("SKIP: MpvContext::new_thumbnail_player() failed: {e}");
                    return None;
                }
            }
        }
        Some(f(slot.as_mut().unwrap()))
    })
}

#[divan::bench(sample_count = 10)]
fn baseline_poll_10ms() -> (u32, u32, Vec<u8>) {
    with_player(|player| {
        divan::black_box(variants::baseline_poll_10ms(player, &fixture()).unwrap())
    })
    .unwrap_or((0, 0, Vec::new()))
}

#[divan::bench(sample_count = 10)]
fn poll_1ms() -> (u32, u32, Vec<u8>) {
    with_player(|player| divan::black_box(variants::poll_1ms(player, &fixture()).unwrap()))
        .unwrap_or((0, 0, Vec::new()))
}

#[divan::bench(sample_count = 10)]
fn poll_0ms_spin() -> (u32, u32, Vec<u8>) {
    with_player(|player| divan::black_box(variants::poll_0ms_spin(player, &fixture()).unwrap()))
        .unwrap_or((0, 0, Vec::new()))
}

#[divan::bench(sample_count = 10)]
fn seek_10pct() -> (u32, u32, Vec<u8>) {
    with_player(|player| divan::black_box(variants::seek_10pct(player, &fixture()).unwrap()))
        .unwrap_or((0, 0, Vec::new()))
}

#[divan::bench(sample_count = 10)]
fn ffmpeg_extract() -> (u32, u32, Vec<u8>) {
    with_player(|player| divan::black_box(variants::ffmpeg_extract(player, &fixture()).unwrap()))
        .unwrap_or((0, 0, Vec::new()))
}

fn main() {
    divan::main();
}
