use crate::command;
use crate::Error;
use crate::TimeRange;
use crate::TimeRanges;
use log::debug;
use regex::Regex;
use rust_decimal::Decimal;
use std::fs;
use std::io;
use std::io::Write;
use std::num::NonZeroUsize;
use std::path::Path;
use std::path::PathBuf;

pub fn ffmpeg<T: AsRef<str>>(requirements: T) -> Result<String, Error> {
    command::run(
        "ffmpeg",
        requirements.as_ref().try_into()?,
        Error::FfmpegCommand,
    )
}

#[derive(Debug)]
pub struct FfmpegStepper {
    input_file: PathBuf,
    frames_dir: PathBuf,
    videos_dir: PathBuf,
    scene_file: PathBuf,
    /// text file
    concat_file: PathBuf,
    /// the video file generated from concatting the videos_dir
    video_file: PathBuf,
    crf: NonZeroUsize,
    fps: NonZeroUsize,
    input_extension: String,
}

impl FfmpegStepper {
    pub fn try_new(
        temp_dir: &Path,
        input_file: PathBuf,
        crf: NonZeroUsize,
        fps: NonZeroUsize,
    ) -> Result<Self, Error> {
        let frames_dir = temp_dir.join("frames");
        dir_exists_or_create(&frames_dir)?;

        let videos_dir = temp_dir.join("videos");
        dir_exists_or_create(&videos_dir)?;

        let input_extension = get_extension(&input_file)?.to_owned();

        let scene_file = temp_dir.join("scene_timestamps.txt");
        let concat_file = temp_dir.join("concat.txt");
        let video_file = temp_dir.join(format!("video.{input_extension}"));

        Ok(Self {
            input_file,
            frames_dir,
            scene_file,
            concat_file,
            video_file,
            videos_dir,
            crf,
            fps,
            input_extension,
        })
    }

    pub fn frames_dir(&self) -> &Path {
        &self.frames_dir
    }

    /// Returns the number of video pieces we've already extracted
    pub fn existing_video_count(&self) -> Result<usize, Error> {
        let count = fs::read_dir(&self.videos_dir)
            .map_err(|_| Error::ReadDir(self.videos_dir.to_path_buf()))?
            .count();
        Ok(count)
    }

    /// Returns a frame extractor which can be used to extract frames
    // todo if scene_file doesn't exist yet, we should clear the video_dir
    pub fn flattened_time_ranges(
        &self,
        max_step_size: NonZeroUsize,
        scene_gt: &str,
    ) -> Result<Vec<TimeRange>, Error> {
        let mut timestamps: Vec<Decimal> = vec![Decimal::ZERO];
        timestamps.append(&mut find_scene_timestamps(
            &self.input_file,
            scene_gt,
            &self.scene_file,
        )?);
        let time_ranges = timestamps
            .as_slice()
            .windows(2)
            .map(|pair| (pair[0], pair[1]))
            .map(|(start, end)| {
                TimeRanges::try_new(start, max_step_size, end)
                    .ok_or(Error::UnableToCreateTimeRanges(start, max_step_size, end))
            })
            .collect::<Result<Vec<TimeRanges>, _>>()?;

        let flattened_time_ranges = time_ranges
            .into_iter()
            .flatten()
            // skipping because we already have existing videos
            .collect();

        Ok(flattened_time_ranges)
    }

    pub fn extract_frames(&self, time_range: &TimeRange) -> Result<&Path, Error> {
        if self.frames_dir.exists() {
            fs::remove_dir_all(&self.frames_dir)?;
            fs::create_dir_all(&self.frames_dir)?;
        }

        extract_frames(
            &time_range.start,
            &self.input_file,
            &time_range.duration(),
            &self.frames_dir,
        )?;
        Ok(&self.frames_dir)
    }

    /// Takes the extracted frames when calling `extract_frames` and creates a video in the
    /// `video_dir`
    pub fn frames_to_video(&self, video_number: usize, input_dir: PathBuf) -> Result<(), Error> {
        let video_path = self
            .videos_dir
            .join(format!("{video_number}.{}", self.input_extension));

        // maybe this will work for windows?
        #[cfg(target_os="windows")]
        let args = format!("-y framerate {} -{} -pattern_type sequence -i %08d.png -crf {} -c:v libx264 -pix_fmt yuv420p {}",
            self.fps,
            self.crf,
            video_path.display()
        );
        #[cfg(not(target_os="windows"))]
        let args = format!("-y -framerate {} -pattern_type glob -i '*.png' -crf {} -c:v libx264 -pix_fmt yuv420p {}",
            self.fps,
            self.crf,
            video_path.display()
        );
        let requirements = command::Requirements {
            args: &args,
            current_dir: input_dir,
        };
        command::run("ffmpeg", requirements, Error::FfmpegCommand)?;
        Ok(())
    }
    /// When you're done extracting frames, call this function and we'll aggregate the
    /// videos + audio + subtitles into the output file provided
    pub fn aggregate(&self, output_file: &Path) -> Result<(), Error> {
        concat_videos(&self.concat_file, &self.videos_dir, &self.video_file)?;

        // Need -max_interleave_delta:
        // https://trac.ffmpeg.org/ticket/6037
        let args = format!("-ignore_unknown -y -i {} -vn -i {} -map 0 -c:v copy -map 1 -c:a copy -c:s copy -map_chapters 1 -max_interleave_delta 0 {}",
            &self.video_file.display(),
            &self.input_file.display(),
            output_file.display()
        );
        ffmpeg(args)?;
        Ok(())
    }
}

/// Give a path to create the concat file for ffmpeg to reference
/// This concat file will have all of the files in the videos_dir provided
/// Then use ffmpeg to concat the videos into the final output_file path
pub fn concat_videos(
    concat_file_path: &Path,
    videos_dir: &Path,
    output_file: &Path,
) -> Result<(), Error> {
    if concat_file_path.exists() {
        fs::remove_file(concat_file_path)?;
    }

    // todo look into not collecting so many times...
    let lines = fs::read_dir(videos_dir)?
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(fs::DirEntry::path)
        .map(fs::canonicalize)
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .map(|path| format!("file {}", path.display()))
        .rev()
        .collect::<Vec<_>>()
        .join("\n");

    let mut concat_file = fs::File::create(concat_file_path)?;
    write!(concat_file, "{lines}")?;

    let args = format!(
        "-y -f concat -safe 0 -i {} -c copy {}",
        concat_file_path.display(),
        output_file.display()
    );
    ffmpeg(args)?;
    Ok(())
}

/// Extracts audio from `input_file` to the audio file you pass in
pub fn extract_audio(input_file: &Path, audio_file: &Path) -> Result<(), Error> {
    let args = format!(
        "-y -i {} -map 0:a -c copy {}",
        input_file.display(),
        audio_file.display()
    );
    ffmpeg(args)?;
    Ok(())
}

fn ffprobe<T: AsRef<str>>(requirements: T) -> Result<String, Error> {
    command::run(
        "ffprobe",
        requirements.as_ref().try_into()?,
        Error::FfprobeCommand,
    )
}

fn parse_timestamps(lines: &str) -> Result<Vec<Decimal>, Error> {
    Regex::new(r"best_effort_timestamp_time=(\d+.\d+)|")
        .unwrap()
        .captures_iter(lines)
        .filter_map(|caps| caps.get(1))
        .map(|m| m.as_str())
        .map(Decimal::from_str_exact)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Error::from)
}

// https://superuser.com/questions/819573/split-up-a-video-using-ffmpeg-through-scene-detection
pub fn find_scene_timestamps(
    input_file: &Path,
    scene_gt: &str,
    scene_file: &Path,
) -> Result<Vec<Decimal>, Error> {
    if scene_file.exists() {
        debug!("{scene_file:?} exists, so using data in that file");
        let decimals = fs::read_to_string(scene_file)?
            .split('\n')
            .map(|s| Decimal::from_str_exact(s).map_err(Error::from))
            .collect::<Result<Vec<_>, Error>>()?;

        return Ok(decimals);
    }
    debug!("creating file: {scene_file:?}");

    let args = format!(
        r#"-show_frames -of compact=p=0 -f lavfi "movie={},select=gt(scene\,{scene_gt})""#,
        input_file.display()
    );

    let stdout = ffprobe(args)?;
    let decimals = parse_timestamps(&stdout)?;
    let mut f = fs::File::create(scene_file)?;
    f.write_all(
        decimals
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join("\n")
            .as_str()
            .as_bytes(),
    )?;
    Ok(decimals)
}

pub fn extract_frames(
    start: &Decimal,
    input_file: &Path,
    duration: &Decimal,
    output_dir: &Path,
) -> Result<(), Error> {
    let args = format!(
        "-ss {start} -i {} -t {duration} -frame_pts true {}",
        input_file.display(),
        output_dir.join("frame_%08d.png").display()
    );
    ffmpeg(args)?;
    Ok(())
}

fn get_extension(path: &Path) -> Result<&str, Error> {
    let extension = path
        .extension()
        .ok_or(Error::MissingExtension(path.to_path_buf()))?;
    let extension = extension
        .to_str()
        .ok_or(Error::InvalidUnicode(extension.to_os_string()))?;
    Ok(extension)
}

fn dir_exists_or_create(path: &Path) -> Result<(), io::Error> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCENE_TIMESTAMPS: &str = "
media_type=video|stream_index=0|key_frame=1|pkt_pts=9760|pkt_pts_time=9.760000|pkt_dts=9760|pkt_dts_time=9.760000|best_effort_timestamp=9760|best_effort_timestamp_time=9.760000|pkt_duration=N/A|pkt_duration_time=N/A|pkt_pos=858320|pkt_size=6220800|width=1920|height=1080|pix_fmt=yuv420p10le|sample_aspect_ratio=1:1|pict_type=I|coded_picture_number=0|display_picture_number=0|interlaced_frame=0|top_field_first=0|repeat_pict=0|color_range=unknown|color_space=unknown|color_primaries=unknown|color_transfer=unknown|chroma_location=unspecified|tag:lavfi.scene_score=0.504959
media_type=video|stream_index=0|key_frame=1|pkt_pts=13513|pkt_pts_time=13.513000|pkt_dts=13513|pkt_dts_time=13.513000|best_effort_timestamp=13513|best_effort_timestamp_time=13.513000|pkt_duration=N/A|pkt_duration_time=N/A|pkt_pos=1176734|pkt_size=6220800|width=1920|height=1080|pix_fmt=yuv420p10le|sample_aspect_ratio=1:1|pict_type=I|coded_picture_number=0|display_picture_number=0|interlaced_frame=0|top_field_first=0|repeat_pict=0|color_range=unknown|color_space=unknown|color_primaries=unknown|color_transfer=unknown|chroma_location=unspecified|tag:lavfi.scene_score=0.428674
media_type=video|stream_index=0|key_frame=1|pkt_pts=18936|pkt_pts_time=18.936000|pkt_dts=18936|pkt_dts_time=18.936000|best_effort_timestamp=18936|best_effort_timestamp_time=18.936000|pkt_duration=N/A|pkt_duration_time=N/A|pkt_pos=1694070|pkt_size=6220800|width=1920|height=1080|pix_fmt=yuv420p10le|sample_aspect_ratio=1:1|pict_type=I|coded_picture_number=0|display_picture_number=0|interlaced_frame=0|top_field_first=0|repeat_pict=0|color_range=unknown|color_space=unknown|color_primaries=unknown|color_transfer=unknown|chroma_location=unspecified|tag:lavfi.scene_score=0.990057
media_type=video|stream_index=0|key_frame=1|pkt_pts=22105|pkt_pts_time=22.105000|pkt_dts=22105|pkt_dts_time=22.105000|best_effort_timestamp=22105|best_effort_timestamp_time=22.105000|pkt_duration=N/A|pkt_duration_time=N/A|pkt_pos=2498438|pkt_size=6220800|width=1920|height=1080|pix_fmt=yuv420p10le|sample_aspect_ratio=1:1|pict_type=I|coded_picture_number=0|display_picture_number=0|interlaced_frame=0|top_field_first=0|repeat_pict=0|color_range=unknown|color_space=unknown|color_primaries=unknown|color_transfer=unknown|chroma_location=unspecified|tag:lavfi.scene_score=0.547889";

    #[test]
    fn scene_time_stamps() {
        let actual = parse_timestamps(SCENE_TIMESTAMPS).unwrap();
        let expected = vec![
            Decimal::from_str_exact("9.76").unwrap(),
            Decimal::from_str_exact("13.513").unwrap(),
            Decimal::from_str_exact("18.936").unwrap(),
            Decimal::from_str_exact("22.105").unwrap(),
        ];
        assert_eq!(actual, expected);
    }
}
