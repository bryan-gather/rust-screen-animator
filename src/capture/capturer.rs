use std::ffi::OsString;

use image::{ImageBuffer, Rgba};

use super::{noop_capturer::NoopCapturer, windows_capturer::WindowsCapturer};

pub struct CaptureWindowResult {
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    position: (i32, i32),
}

// Platform-agnostic trait for something that can capture screenshots
pub trait Capturer {
    fn init(&self);

    fn list_windows(&self) -> Result<Vec<(u64, OsString)>, ()>;

    // TODO: Error type
    fn capture_window(&self, window_id: u64) -> Result<image::DynamicImage, ()>;
}

pub fn new() -> Box<dyn Capturer> {
    Box::new(WindowsCapturer {})
}
