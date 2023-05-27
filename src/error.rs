use rust_decimal::Decimal;
use std::ffi::OsString;
use std::io;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("IO operation failed: {0:?}")]
    Io(#[from] io::Error),
    #[error("Failed to execute Ffprobe")]
    FfprobeCommand,
    #[error("Failed to execute ffmpeg")]
    FfmpegCommand,
    #[error("Failed to execute interpolation command")]
    InterpolationCommand,
    #[error("Command parse failure: {0}")]
    ParseError(#[from] shell_words::ParseError),
    #[error("Decimal error: {0}")]
    Decimal(#[from] rust_decimal::Error),
    #[error("Unable to create scene time ranges:\nstart {0}\nmax_step_size {1}\nend {2}")]
    UnableToCreateTimeRanges(Decimal, NonZeroUsize, Decimal),
    #[error("Unable to calculate expected interpolation frame count from FPS: {0}")]
    BadFPS(NonZeroUsize),
    #[error("Multiplcation overflow: {0} * {1}")]
    MultiplicationOverflow(String, String),
    #[error("Missing extension for file: {0}")]
    MissingExtension(PathBuf),
    #[error("Invalid Unicode: {0:?}")]
    InvalidUnicode(OsString),
    #[error("Unable to read dir: {0:?}")]
    ReadDir(PathBuf),
}
