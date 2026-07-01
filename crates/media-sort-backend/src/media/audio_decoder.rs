use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use rodio::{Decoder, DeviceSinkBuilder, Player, Source};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("decoder error: {0}")]
    Decoder(String),
    #[error("playback error: {0}")]
    Playback(String),
    #[error("seek error: {0}")]
    Seek(String),
}

impl From<rodio::decoder::DecoderError> for AudioError {
    fn from(e: rodio::decoder::DecoderError) -> Self {
        AudioError::Decoder(e.to_string())
    }
}

impl From<rodio::PlayError> for AudioError {
    fn from(e: rodio::PlayError) -> Self {
        AudioError::Playback(e.to_string())
    }
}

impl From<rodio::DeviceSinkError> for AudioError {
    fn from(e: rodio::DeviceSinkError) -> Self {
        AudioError::Playback(e.to_string())
    }
}

impl From<rodio::source::SeekError> for AudioError {
    fn from(e: rodio::source::SeekError) -> Self {
        AudioError::Seek(e.to_string())
    }
}

pub struct AudioPlayer {
    _sink: rodio::MixerDeviceSink,
    player: Player,
    duration: Mutex<f64>,
}

fn probe_duration(path: &Path) -> Option<f64> {
    use symphonia::core::formats::probe::Hint;
    use symphonia::core::formats::{FormatOptions, TrackType};
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::units::Timestamp;

    let file = File::open(path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let format = symphonia::default::get_probe()
        .probe(
            &hint,
            mss,
            FormatOptions::default(),
            MetadataOptions::default(),
        )
        .ok()?;
    let track = format.default_track(TrackType::Audio)?;
    let tb = track.time_base?;
    let frames = track.num_frames?;
    let ts = Timestamp::try_from(frames).ok()?;
    let time = tb.calc_time(ts)?;
    Some(time.as_secs_f64())
}

impl AudioPlayer {
    pub fn new() -> Result<Self, AudioError> {
        let sink = DeviceSinkBuilder::open_default_sink()?;
        let player = Player::connect_new(sink.mixer());
        Ok(Self {
            _sink: sink,
            player,
            duration: Mutex::new(0.0),
        })
    }

    pub fn play(&self, path: &Path) -> Result<(), AudioError> {
        let file = File::open(path)?;
        let decoder = Decoder::try_from(file)?;
        let dur = probe_duration(path)
            .or_else(|| decoder.total_duration().map(|d| d.as_secs_f64()))
            .unwrap_or(0.0);
        self.player.append(decoder);
        if let Ok(mut d) = self.duration.lock() {
            *d = dur;
        }
        self.player.play();
        Ok(())
    }

    pub fn pause(&self) {
        self.player.pause();
    }

    pub fn resume(&self) {
        self.player.play();
    }

    pub fn stop(&self) {
        self.player.stop();
    }

    pub fn is_paused(&self) -> bool {
        self.player.is_paused()
    }

    pub fn empty(&self) -> bool {
        self.player.empty()
    }

    pub fn position(&self) -> f64 {
        self.player.get_pos().as_secs_f64()
    }

    pub fn duration(&self) -> f64 {
        *self.duration.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn volume(&self) -> f32 {
        self.player.volume()
    }

    pub fn set_volume(&self, vol: f32) {
        self.player.set_volume(vol);
    }

    pub fn seek(&self, pos_secs: f64) -> Result<(), AudioError> {
        let dur = Duration::from_secs_f64(pos_secs.max(0.0));
        self.player.try_seek(dur)?;
        Ok(())
    }
}
