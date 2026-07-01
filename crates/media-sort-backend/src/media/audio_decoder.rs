use std::fs::File;
use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use rodio::{Decoder, OutputStream, Sink, Source};
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

impl From<rodio::StreamError> for AudioError {
    fn from(e: rodio::StreamError) -> Self {
        AudioError::Playback(e.to_string())
    }
}

impl From<rodio::source::SeekError> for AudioError {
    fn from(e: rodio::source::SeekError) -> Self {
        AudioError::Seek(e.to_string())
    }
}

pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
    duration: Mutex<f64>,
}

fn probe_duration(path: &Path) -> Option<f64> {
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;

    let file = File::open(path).ok()?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .ok()?;
    let track = probed.format.default_track()?;
    let params = &track.codec_params;
    let tb = params.time_base?;
    let frames = params.n_frames?;
    Some(tb.calc_time(frames).seconds as f64 + tb.calc_time(frames).frac as f64)
}

impl AudioPlayer {
    pub fn new() -> Result<Self, AudioError> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        Ok(Self {
            _stream: stream,
            sink,
            duration: Mutex::new(0.0),
        })
    }

    pub fn play(&self, path: &Path) -> Result<(), AudioError> {
        let file = File::open(path)?;
        let decoder = Decoder::new(file)?;
        let dur = probe_duration(path)
            .or_else(|| decoder.total_duration().map(|d| d.as_secs_f64()))
            .unwrap_or(0.0);
        self.sink.append(decoder);
        if let Ok(mut d) = self.duration.lock() {
            *d = dur;
        }
        self.sink.play();
        Ok(())
    }

    pub fn pause(&self) {
        self.sink.pause();
    }

    pub fn resume(&self) {
        self.sink.play();
    }

    pub fn stop(&self) {
        self.sink.stop();
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn empty(&self) -> bool {
        self.sink.empty()
    }

    pub fn position(&self) -> f64 {
        self.sink.get_pos().as_secs_f64()
    }

    pub fn duration(&self) -> f64 {
        *self.duration.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn volume(&self) -> f32 {
        self.sink.volume()
    }

    pub fn set_volume(&self, vol: f32) {
        self.sink.set_volume(vol);
    }

    pub fn seek(&self, pos_secs: f64) -> Result<(), AudioError> {
        let dur = Duration::from_secs_f64(pos_secs.max(0.0));
        self.sink.try_seek(dur)?;
        Ok(())
    }
}
