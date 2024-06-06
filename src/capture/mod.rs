mod noop_capturer;

#[cfg(target_os = "windows")]
mod windows_capturer;

#[cfg(target_os = "macos")]
mod macos_capturer;

pub mod capturer;
