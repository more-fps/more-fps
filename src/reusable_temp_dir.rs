use clap::ValueEnum;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use strum_macros::Display;

#[derive(Debug)]
pub struct ReusableTempDir {
    base_dir: PathBuf,
    ffmpeg_dir: PathBuf,
    generated_frames_dir: PathBuf,
}

impl ReusableTempDir {
    pub fn try_new(base_dir: PathBuf, reset_data: ResetData) -> Result<Self, io::Error> {
        let ffmpeg_dir = base_dir.join("ffmpeg");
        let generated_frames_dir = base_dir.join("generated_frames");

        match reset_data {
            ResetData::Everything => fs::remove_dir_all(&base_dir).unwrap_or_default(),
            ResetData::Nothing => (),
        };

        // recreate the folders
        dir_exists_or_create(&ffmpeg_dir)?;
        dir_exists_or_create(&generated_frames_dir)?;

        Ok(Self {
            base_dir,
            ffmpeg_dir,
            generated_frames_dir,
        })
    }

    pub fn ffmpeg_dir(&self) -> &PathBuf {
        &self.ffmpeg_dir
    }

    pub fn generated_frames_dir(&self) -> &PathBuf {
        &self.generated_frames_dir
    }

    pub fn delete(self) -> Result<(), io::Error> {
        fs::remove_dir_all(self.base_dir)?;
        Ok(())
    }
}

fn dir_exists_or_create(path: &Path) -> Result<(), io::Error> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

#[derive(ValueEnum, Copy, Clone, Debug, Default, Display)]
#[strum(serialize_all = "kebab-case")]
pub enum ResetData {
    /// Delete the entire temp_directory which contains a few building blocks:
    ///   "ffmpeg" - used for storing extracted frames
    ///   "generated_frames" - used for storing generated frames
    ///   "scene_data.txt" - holds scene timestamps
    #[default]
    Everything,
    /// Nothing will be deleted... meaning we try to continue from where we left off
    Nothing,
}
