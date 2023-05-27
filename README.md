# More FPS

Using an AI model that can interpolate frames, we can make our videos smoother.

This project is aimed to automate the process.

## ✔️ Requirements ✔️

- [ffmpeg](https://ffmpeg.org/download.html)
  - extracting video frames
  - extracting audio
- [ffprobe](https://ffmpeg.org/download.html)
  - extract scenes
- An AI Model that supports targeting a frame count (via the "-n" option):
  - (rife v4.6)[https://github.com/nihui/rife-ncnn-vulkan]

*⚠️ You'll need ffmpeg and ffprobe added to your PATH. Windows users, you may need to [add them to your path manually](https://www.howtogeek.com/118594/how-to-edit-your-system-path-for-easy-command-line-access/).*

## Installation

Install [rust](https://www.rust-lang.org/tools/install)

Clone the repo and build the project:
```
cargo b --release
```

## Usage:

First we need to set a few environment variables (use the correct path):
- `INTERPOLATION_MODEL`
  - The path to the model folder
- `INTERPOLATION_BINARY`
  - The path to the interpolation binary (which is a file!)

As an example, assuming the ai model and binary are in your current directory:

### Linux:
```
export INTERPOLATION_MODEL=models/rife-v4.6
export INTERPOLATION_BINARY=rife-ncnn-vulkan
RUST_LOG=INFO cargo run --release -- -t /tmp/more_fps/ input.mkv output.mkv
```

### Windows:

TODO


---

```
Usage: more-fps [OPTIONS] -t <TEMP_DIR> <INPUT> <OUTPUT> <INTERPOLATION_BINARY> <INTERPOLATION_MODEL>

Arguments:
  <INPUT>
          Path to the file for which we'll increase the frame rate

  <OUTPUT>
          final output path if it exists, we'll try to build on-top of it

  <INTERPOLATION_BINARY>
          AIModel used to generate interpolated frames
          
          [env: INTERPOLATION_BINARY=]

  <INTERPOLATION_MODEL>
          [env: INTERPOLATION_MODEL=]

Options:
      --fps <FPS>
          The number target frame count for the interpolation binary The default will have the interpolation binary change your (most likely 24fps) video to 60fps
          
          [default: sixty]

          Possible values:
          - sixty: 60 fps

  -t <TEMP_DIR>
          Path to put temporary/intermediate data like ffmpeg generated frames and interpolated frames If the path doesn't exist, it will be created Perferably a fast m.2 ssd or ramdisk because they are fast

  -m <MAX_STEP_SIZE>
          Maximum number of seconds to extract (assuming the scene splits are too big)
          
          [default: 50]

      --interpolation-args <INTERPOLATION_ARGS>
          Extra args you may want to pass to the interpolation binary
          
          [default: "-g 0,-1 -j 8:8,16:32:16"]

  -r <RESET>
          Clears cached data
          
          [default: everything]

          Possible values:
          - everything: Delete the entire temp_directory which contains a few building blocks: "ffmpeg" - used for storing extracted frames "interpolation" - used for storing interpolated frames "scene_data.txt" - holds scene timestamps
          - nothing:    Nothing will be deleted... meaning we try to continue from where we left off

  -s <SCENE_GT>
          How should we split the video up before interpolating If there is a big difference between frames, the interpolation will generate bad frames
          
          [default: .2]

      --crf <CRF>
          https://trac.ffmpeg.org/wiki/Encode/H.264#a1.ChooseaCRFvalue
          
          [default: 18]

  -h, --help
          Print help (see a summary with '-h')

```

## 🧠 Pro tips 🧠

- Have your temporary directory target a RAM disk. This will significantly speed up the process because we do A LOT of writes.
- If you want to continue where you left off, set the reset option (`-r`) to `nothing`. With this option set to `nothing`, we will simply continue extracting from where we left off last time.

## 💡 How it works 💡

1. Identify the video's scene changes
1. Extract frames
1. Interpolate frames to match the target FPS (default: 60)
1. Aggregate the interpolated frames into a new video
1. Include audio + subtitle from the original video

### Identify the scene changes

If two consecutive frames are different enough, the model could generate a very bad intermediate frame. To avoid this, we use ffmpeg to identify the timestamps which have large-enough scene differences. You can change this scene value with the `-s` option (see help text above):

https://www.ffmpeg.org/ffmpeg-filters.html#select_002c-aselect

We then use these timestamps as start/end times when extracting frames.

*⚠️ If you notice these bad frames, I recommend decreasing the scene cut threshold with the `-s` option.*

### Extract the frames

We want to extract the frames from our original video file, but we don't want to extract all of the frames in one shot for two reasons:
1. As mentioned above, the AI model could be generating bad intermediate frames.
1. Extracting all of frames could require hundreds of gigabytes of data which should exceed the capacity of your disk.

So instead of extracting all of the frames, we extract according to the intervals given by the timestamps.

*These time intervals may still be very large... To avoid hitting the capacity of the disk, I set a default MAX_STEP_SIZE. This option is used to set a limit on the number of seconds a frame extraction will use at a time.*

### Interpolate the frames to match the target FPS

This is where the AI model is used. We need a model that supports the "-n" option mentioned above so we can tell the model how many frames to generate per frame extraction. 

The simplest example is if we have 1 second (which is usually 23.998 aka 24 fps), the AI model will be told to generate 60 frames. Not all scene cuts are this nice, so decimals are involved...

*⚠️ I assume you have one CPU and GPU you want to use... If this is not the case, feel free to change the `--interpolation-args` option*

### Aggregate the interpolated frames

Now that we have the AI generated frames, we use ffmpeg to generate the video.

### Include audio + subtitle from the original video

After we're done extracting frames, we will copy the audio + subtitles from the original file to our output file.

---


## Other

If you're looking to support me, you can send any amount of Monero:

```
42NTT1Q91P2TcG7vzN3oi2cYEJEaJ6QbzH3pwXGDKiSQiaBVAeZkYnBX6SijCxgKpc5tTUeVW5AuwDWBNdewZia9AJ5TgLT
```

## TODO
  - add a pipeline for tests
  - cli should include preset setting so we can target ultrafast/veryslow
    - https://trac.ffmpeg.org/wiki/Encode/H.264#Preset
  - support higher FPS like 75, 90, 120, 144, 165, 180, 240, etc.
  - put this on crates.io so we can `cargo install`
  - support windows

