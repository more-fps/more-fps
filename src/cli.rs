use crate::ResetData;
use std::num::NonZeroUsize;
use std::path::PathBuf;

use crate::NonZeroDecimal;
use clap::Parser;

use crate::FPS;

#[derive(Debug, Parser)]
pub struct Cli {
    /// Path to the file for which we'll increase the frame rate
    #[arg(value_parser=is_file)]
    pub input: PathBuf,

    /// final output path
    /// if it exists, we'll try to build on-top of it
    #[arg(value_parser=output_dne)]
    pub output: PathBuf,
    ///
    /// AIModel used to generate interpolated frames
    #[arg(value_parser=is_file, env)]
    pub interpolation_binary: PathBuf,

    #[arg(value_parser=is_dir, env)]
    pub interpolation_model: PathBuf,

    /// The number target frame count for the interpolation binary
    /// The default will have the interpolation binary change your (most likely 24fps) video to
    /// 60fps
    #[arg(long, value_enum, default_value_t=FPS::default())]
    pub fps: FPS,

    /// Path to put temporary/intermediate data
    /// like ffmpeg generated frames and interpolated frames
    /// If the path doesn't exist, it will be created
    /// Perferably a fast m.2 ssd or ramdisk because they are fast
    #[arg(short, value_parser=dne_or_is_dir)]
    pub temp_dir: PathBuf,

    /// Maximum number of seconds to extract (assuming the scene splits are too big)
    #[arg(short='m', default_value_t = NonZeroUsize::new(50).unwrap())]
    pub max_step_size: NonZeroUsize,

    /// Extra args you may want to pass to the interpolation binary
    #[arg(long, default_value_t = default_interpolation_args())]
    pub interpolation_args: String,

    /// Clears cached data
    #[arg(short='r', default_value_t = ResetData::default())]
    pub reset: ResetData,

    /// How should we split the video up before interpolating
    /// If there is a big difference between frames, the interpolation
    /// will generate bad frames.
    #[arg(short='s', default_value_t = String::from(".2"), value_parser=can_be_decimal)]
    pub scene_gt: String,

    /// https://trac.ffmpeg.org/wiki/Encode/H.264#a1.ChooseaCRFvalue
    #[arg(long, default_value_t = NonZeroUsize::new(18).unwrap())]
    pub crf: NonZeroUsize,
}

fn can_be_decimal(scene_gt: &str) -> Result<String, String> {
    NonZeroDecimal::try_from(scene_gt)
        .map_err(|e| format!("scene_gt should be a non-zero decimal: {e}"))?;
    Ok(scene_gt.to_owned())
}

/// Confirm the path exists + is a file
fn is_file(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.is_file() {
        return Err(format!("path doesn't exist or isn't a file: {path:?}"));
    }
    Ok(path)
}

fn is_dir(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.is_dir() {
        return Err(format!("path doesn't exist is isn't a directory: {path:?}"));
    }
    Ok(path)
}

fn dne_or_is_dir(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.exists() && !path.is_dir() {
        return Err(format!(
            "Path should not exist or should be a folder: {path:?}"
        ));
    }
    Ok(path)
}

fn output_dne(path: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if path.exists() {
        return Err(format!(
            "Output path already exist. Please delete the file to continue: {path:?}"
        ));
    }
    Ok(path)
}

fn default_interpolation_args() -> String {
    let cpu_count = num_cpus::get();
    format!("-g 0,-1 -j {cpu_count}:{cpu_count},16:32:16")
}
