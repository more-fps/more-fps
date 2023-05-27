use clap::ValueEnum;
use std::num::NonZeroUsize;

/// How many frames do you want per second
#[derive(ValueEnum, Copy, Clone, Debug, Default)]
pub enum FPS {
    /// 60 fps
    #[default]
    Sixty = 60,
}

impl FPS {
    pub fn non_zero_usize(self) -> NonZeroUsize {
        NonZeroUsize::new(self as usize).unwrap()
    }
}
