# clip-recreate

A frame-by-frame vector recreation of a 20.7-second motion-design clip, rendered in Rust with
[fframes](https://github.com/dmtrKovalenko/fframes) ([fframes.studio](https://fframes.studio)).

![Eight frames from the rendered video](media/preview.jpg)

Rendered result: [media/v2.mp4](media/v2.mp4) (960×720, 30 fps, 620 frames, with audio).

## How it works

1. Every frame of the reference clip was traced into flat colour shapes by
   [`tools/vectorize.py`](tools/vectorize.py). Each shot gets one fixed palette of 64 colours, so
   colours do not flicker between frames. The result is one text file per frame in
   [`assets/roto/`](assets/roto), one `#rrggbb<TAB>svg-path` line per colour layer.
2. [`src/lib.rs`](src/lib.rs) loads those 620 files and returns frame N's shapes as SVG for
   video frame N. It also splits the timeline into the clip's 23 shots and plays
   `assets/soundtrack.wav`.
3. [`src/main.rs`](src/main.rs) renders the frames on the GPU (Skia on Vulkan) and encodes
   H.264 (CRF 14). The audio is passed through untouched.

## Requirements

- Rust, from <https://rustup.rs>
- A GPU with Vulkan drivers
- Windows:
  - FFmpeg 9 shared build: `ffmpeg-n9.0-latest-win64-gpl-shared` from
    [BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds/releases), unzipped anywhere
  - LLVM, for libclang: `winget install LLVM.LLVM`
- Linux and macOS: the system libraries listed in the
  [fframes README](https://github.com/dmtrKovalenko/fframes). For example, on Debian or Ubuntu:

  ```bash
  sudo apt-get install -y yasm nasm ffmpeg libx264-dev libx265-dev libopus-dev libclang-dev clang ninja-build libvpx-dev libasound2-dev
  ```

## Build and render

### Windows (PowerShell)

`r.ps1` builds in release mode, then runs the CLI with your arguments. Point it at your FFmpeg
and LLVM folders if they are not at the defaults (`C:\ffmpeg-n9.0-latest-win64-gpl-shared-9.0`
and `C:\Program Files\LLVM\bin`):

```powershell
git clone https://github.com/4regab/clip-recreate
cd clip-recreate
$env:FFMPEG_DIR = 'C:\path\to\ffmpeg-n9.0-latest-win64-gpl-shared-9.0'
$env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin'
.\r.ps1 render -o out\v2.mp4
```

The first build compiles Skia and every dependency, which takes a while. Later builds take
seconds.

### Linux and macOS

```bash
git clone https://github.com/4regab/clip-recreate
cd clip-recreate
cargo run --release -- render -o out/v2.mp4
```

### Other commands

```text
.\r.ps1 timeline                 # list the 23 shots and the audio track
.\r.ps1 frame 480 -o out_test    # render one frame to PNG
.\r.ps1 strip all -n 12          # contact sheet of 12 evenly spaced frames
.\r.ps1 preview                  # play it in a real-time GPU window
.\r.ps1 --help                   # everything else
```

On Linux and macOS, use `cargo run --release -- <command>` instead of `.\r.ps1 <command>`.

## Re-tracing the frames (optional)

`assets/roto/` is already in the repository, so you only need this to trace a different clip
or to change the tracing. You need Python with `opencv-python` and `numpy`, plus the reference
clip at 960×720 and 30 fps:

```bash
mkdir work && cd work
mkdir frames
ffmpeg -i reference.mp4 -start_number 0 frames/f%04d.png
python ../tools/vectorize.py ../assets/roto 64
ffmpeg -i reference.mp4 -vn -ac 2 -ar 48000 -c:a pcm_s16le ../assets/soundtrack.wav
```

The shot boundaries live in two places: `SHOTS` in `tools/vectorize.py` and `CUTS` in
`src/lib.rs`. Change both if your clip has different cuts.

## Layout

```text
src/lib.rs          the video: loads the traced frames, shot list, audio
src/main.rs         renderer and encoder settings (Skia/Vulkan, libx264 CRF 14)
assets/roto/        620 traced frames, f0000.txt to f0619.txt
assets/soundtrack.wav  the clip's audio (48 kHz stereo)
tools/vectorize.py  the tracing script that made assets/roto
media/v2.mp4        the rendered video
r.ps1               Windows build-and-run helper
```

## Note

The traced frames and the audio come from a third-party clip. They are included only so the
render can be reproduced. They are not covered by any licence for this code.
