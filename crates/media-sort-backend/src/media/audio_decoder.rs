use std::fs::File;
use std::path::Path;

use rodio::{Decoder, OutputStream, Sink};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    #[error("decoder error: {0}")]
    Decoder(String),
    #[error("playback error: {0}")]
    Playback(String),
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

pub struct AudioPlayer {
    _stream: OutputStream,
    sink: Sink,
}

impl AudioPlayer {
    pub fn new() -> Result<Self, AudioError> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;
        Ok(Self {
            _stream: stream,
            sink,
        })
    }

    pub fn play(&self, path: &Path) -> Result<(), AudioError> {
        let file = File::open(path)?;
        let source = Decoder::new(file)?;
        self.sink.append(source);
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

    pub fn set_volume(&self, vol: f32) {
        self.sink.set_volume(vol);
    }
}
