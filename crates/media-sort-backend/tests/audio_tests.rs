use media_sort_backend::media::audio_decoder::AudioPlayer;

#[test]
fn test_audio_player_new() {
    let player = AudioPlayer::new();
    assert!(player.is_ok());
}
