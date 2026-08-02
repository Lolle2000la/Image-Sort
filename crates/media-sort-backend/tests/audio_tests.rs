use media_sort_backend::media::audio_decoder::AudioPlayer;

#[test]
fn test_audio_player_new() {
    // Audio output is non-fatal by design (AppState keeps a None player):
    // headless CI runners have no sound card, so skip instead of failing.
    match AudioPlayer::new() {
        Ok(_) => {}
        Err(e) => eprintln!("SKIP: no audio device available: {e}"),
    }
}
