use crate::command;
use crate::Error;
use crate::NonZeroDecimal;
use crate::FPS;
use rust_decimal::Decimal;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct FrameGenerator<'a> {
    pub binary: &'a Path,
    pub model: &'a Path,
    pub fps: FPS,
    pub input_dir: &'a Path,
    pub output_dir: &'a PathBuf,
    pub extra_args: &'a str,
}

impl<'a> FrameGenerator<'a> {
    pub fn clear_output_dir(&self) -> Result<(), Error> {
        if self.output_dir.exists() {
            fs::remove_dir_all(self.output_dir)?;
            fs::create_dir_all(self.output_dir)?;
        }
        Ok(())
    }
    pub fn execute(&self, duration: NonZeroDecimal) -> Result<&Path, Error> {
        self.clear_output_dir()?;

        let frame_count = self.frame_count(duration)?;
        let args = format!(
            "-m {} -i {} -o {} -n {frame_count} {}",
            self.model.display(),
            self.input_dir.display(),
            self.output_dir.display(),
            self.extra_args
        );
        command::run(
            &self.binary.display().to_string(),
            args.as_str().try_into()?,
            Error::AICommand,
        )?;
        Ok(self.output_dir)
    }

    /// the frame count for ai binary to target
    /// denoted as the -n flag
    /// While this function does always return a `NonZeroDecimal`, we need a `Decimal` to `Display` in
    /// `execute`. So there's no point in returning a `NonZeroDecimal`
    fn frame_count(&self, duration: NonZeroDecimal) -> Result<Decimal, Error> {
        let fps = *NonZeroDecimal::try_new(self.fps.non_zero_usize().get())
            .ok_or(Error::BadFPS(self.fps.non_zero_usize()))?;

        let frame_count = fps
            .checked_mul(*duration)
            .ok_or(Error::MultiplicationOverflow(
                fps.to_string(),
                duration.to_string(),
            ))?
            .round_dp(0);

        Ok(frame_count)
    }
}
