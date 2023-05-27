use clap::Parser;
use log::{debug, info};
use more_fps::Cli;
use more_fps::Error;
use more_fps::FfmpegStepper;
use more_fps::Interpolation;
use more_fps::ReusableTempDir;

fn main() -> Result<(), Error> {
    env_logger::init();
    let args = Cli::parse();
    debug!("{args:?}");

    let temp_dir = ReusableTempDir::try_new(args.temp_dir, args.reset)?;
    let ffmpeg_stepper = FfmpegStepper::try_new(
        temp_dir.ffmpeg_dir(),
        args.input,
        args.crf,
        args.fps.non_zero_usize(),
    )?;

    let interpolation = Interpolation {
        binary: &args.interpolation_binary,
        model: &args.interpolation_model,
        fps: args.fps,
        input_dir: ffmpeg_stepper.frames_dir(),
        extra_args: &args.interpolation_args,
        output_dir: temp_dir.interpolation_dir(),
    };

    let time_ranges = ffmpeg_stepper
        .flattened_time_ranges(args.max_step_size, &args.scene_gt)?
        .into_iter()
        .enumerate();
    let existing_video_count = ffmpeg_stepper.existing_video_count()?;

    for (index, time_range) in time_ranges.skip(existing_video_count) {
        let duration = time_range.duration();

        ffmpeg_stepper.extract_frames(&time_range)?;
        let interpolated_frames_dir = interpolation.execute(duration)?.to_owned();
        ffmpeg_stepper.frames_to_video(index, interpolated_frames_dir)?;
        info!(
            "Extracting a total of {} seconds",
            time_range.start + *duration
        );
        //pause();
    }

    info!("Finished extracting ALL frames, now creating the final video");
    ffmpeg_stepper.aggregate(&args.output)?;

    temp_dir.delete()?;
    Ok(())
}

// Useful for debugging
//use std::io;
//use std::io::prelude::*;
//
//fn pause() {
//    let mut stdin = io::stdin();
//    let mut stdout = io::stdout();
//
//    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
//    write!(stdout, "Press any key to continue...").unwrap();
//    stdout.flush().unwrap();
//
//    // Read a single byte and discard
//    let _ = stdin.read(&mut [0u8]).unwrap();
//
//}
