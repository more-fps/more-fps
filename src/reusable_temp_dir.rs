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
    interpolation_dir: PathBuf,
}

impl ReusableTempDir {
    pub fn try_new(base_dir: PathBuf, reset_data: ResetData) -> Result<Self, io::Error> {
        let ffmpeg_dir = base_dir.join("ffmpeg");
        let interpolation_dir = base_dir.join("interpolation");

        match reset_data {
            ResetData::Everything => fs::remove_dir_all(&base_dir).unwrap_or_default(),
            ResetData::Nothing => (),
        };

        // recreate the folders
        dir_exists_or_create(&ffmpeg_dir)?;
        dir_exists_or_create(&interpolation_dir)?;

        Ok(Self {
            base_dir,
            ffmpeg_dir,
            interpolation_dir,
        })
    }

    pub fn ffmpeg_dir(&self) -> &PathBuf {
        &self.ffmpeg_dir
    }

    pub fn interpolation_dir(&self) -> &PathBuf {
        &self.interpolation_dir
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
    ///   "interpolation" - used for storing interpolated frames
    ///   "scene_data.txt" - holds scene timestamps
    #[default]
    Everything,
    /// Nothing will be deleted... meaning we try to continue from where we left off
    Nothing,
}
