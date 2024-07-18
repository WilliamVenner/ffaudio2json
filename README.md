[![crates.io](https://img.shields.io/crates/v/ffaudio2json.svg)](https://crates.io/crates/ffaudio2json)
[![docs.rs](https://docs.rs/ffaudio2json/badge.svg)](https://docs.rs/ffaudio2json/)
![license](https://img.shields.io/crates/l/ffaudio2json)
[![CI Status](https://github.com/WilliamVenner/ffaudio2json/workflows/ci/badge.svg)](https://github.com/WilliamVenner/ffaudio2json/actions?query=workflow%3A%22ci%22)

# FFAudio2JSON

Convert audio files to JSON waveforms!

Based on [wav2json](https://github.com/beschulz/wav2json)

<details>
  <summary><h2>Example</h2></summary>

```sh
ffaudio2json song.wav --channels "left right mid side min max" -o song.json
```

```json
{
  "_generator": "ffaudio2json version 0.1.0 on x86_64-pc-windows-msvc (https://github.com/WilliamVenner/ffaudio2json)",
  "left": [
    0.947125, 0.901331, 0.76628, 0.578968, 0.744371, 0.57511, 0.624754, 0.7391,
    0.534745, 0.561727, 0.565447, 0.777101, 0.633872, 0.443988, 0.451541
  ],
  "right": [
    0.895935, 0.869228, 0.782387, 0.58325, 0.80669, 0.592015, 0.599639,
    0.731451, 0.472213, 0.571442, 0.524964, 0.792326, 0.549566, 0.50713,
    0.494696
  ],
  "mid": [
    0.92153, 0.74639, 0.774334, 0.494298, 0.775531, 0.508056, 0.601378,
    0.735276, 0.393787, 0.566585, 0.459236, 0.784713, 0.426951, 0.43994,
    0.462662
  ],
  "side": [
    0.218711, 0.460376, 0.446356, 0.467093, 0.535112, 0.556382, 0.327098,
    0.455026, 0.365384, 0.321987, 0.514186, 0.492502, 0.398223, 0.324473,
    0.365328
  ],
  "min": [
    0.947125, 0.869228, 0.76628, 0.567466, 0.744371, 0.592015, 0.60579,
    0.731451, 0.534745, 0.571442, 0.565447, 0.792326, 0.633872, 0.49244,
    0.494696
  ],
  "max": [
    0.895935, 0.901331, 0.782387, 0.58325, 0.80669, 0.537654, 0.624754, 0.7391,
    0.465301, 0.563098, 0.524964, 0.777101, 0.549566, 0.50713, 0.475655
  ],
  "duration": 168.552
}
```

</details>

## Usage

FFAudio2JSON can be used as a [Rust library](https://docs.rs/ffaudio2json) or as a standalone binary.

```sh
Convert audio files to JSON waveforms using FFmpeg

Usage: ffaudio2json.exe [OPTIONS] <INPUT>

Arguments:
  <INPUT>

Options:
  -s, --samples <SAMPLES>      Number of samples to generate [default: 800]
      --db-min <DB_MIN>        Minimum value of the signal in dB that will be visible in the waveform [default: -48]
      --db-max <DB_MAX>        Maximum value of the signal in dB that will be visible in the waveform. Useful,if you know that your signal peaks at a certain level [default: -48]
  -d, --db-scale               Use logarithmic (e.g. decibel) scale instead of linear scale
  -p, --precision <PRECISION>  Precision of the floats that are generated. Reduce for smaller sized files. Usually 2 should be sufficient [default: 6]
  -n, --no-header              Do not include the version info banner in the output
  -o, --output <OUTPUT>        Name of output file, defaults to <name of inputfile>.json
      --channels <CHANNELS>    Channels to compute: left, right, mid, side, min, max [default: "left right"]
  -q, --quiet                  Suppress all output
  -h, --help                   Print help
  -V, --version                Print version
```

## Downloads

ffaudio2json offers many different builds that are suitable for different needs.

**Shared** linking means the build does not include ffmpeg and you will need to install it to your system as a shared library. [Instructions](#shared-linking)

**Static** linking means the build should be "portable" and ran without having to install additional dependencies (including ffmpeg)

**Bold = Recommended**

| OS          | Arch               | Linking    | Download                                                                                                                                          |
| ----------- | ------------------ | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Linux**   | **x86-64**         | **Shared** | [**`ffaudio2json_linux_x86-64_shared`**](https://github.com/WilliamVenner/ffaudio2json/releases/latest/download/ffaudio2json_linux_x86-64_shared) |
| Linux       | x86-64             | Static     | [`ffaudio2json_linux_x86-64_static`](https://github.com/WilliamVenner/ffaudio2json/releases/latest/download/ffaudio2json_linux_x86-64_static)     |
| **Linux**   | **arm64**          | **Shared** | [**`ffaudio2json_linux_arm64_shared`**](https://github.com/WilliamVenner/ffaudio2json/releases/latest/download/ffaudio2json_linux_arm64_shared)   |
| Linux       | arm64              | Static     | [`ffaudio2json_linux_arm64_static`](https://github.com/WilliamVenner/ffaudio2json/releases/latest/download/ffaudio2json_linux_arm64_static)       |
| **macOS**   | **arm64 (M1)**     | **Static** | [**`ffaudio2json_macos_arm64`**](https://github.com/WilliamVenner/ffaudio2json/releases/latest/download/ffaudio2json_macos_arm64)                 |
| macOS       | arm64 (M1)         | Shared     | [`ffaudio2json_macos_arm64_shared`](https://github.com/WilliamVenner/ffaudio2json/releases/latest/download/ffaudio2json_macos_arm64_shared)       |
| **macOS**   | **x86-64 (Intel)** | **Static** | [**`ffaudio2json_macos_x86-64`**](https://github.com/WilliamVenner/ffaudio2json/releases/latest/download/ffaudio2json_macos_x86-64)               |
| macOS       | x86-64 (Intel)     | Shared     | [`ffaudio2json_macos_x86-64_shared`](https://github.com/WilliamVenner/ffaudio2json/releases/latest/download/ffaudio2json_macos_x86-64_shared)     |
| **Windows** | **x86-64**         | **Static** | [**`ffaudio2json_win_x86-64.exe`**](https://github.com/WilliamVenner/ffaudio2json/releases/latest/download/ffaudio2json_win_x86-64.exe)           |
| Windows     | x86-64             | Shared     | [`ffaudio2json_win_x86-64_shared.exe`](https://github.com/WilliamVenner/ffaudio2json/releases/latest/download/ffaudio2json_win_x86-64_shared.exe) |

### Shared Linking

#### Linux

For Debian-based systems:

```sh
sudo apt update
sudo apt install -y ffmpeg
```

#### macOS

```
brew install ffmpeg
```

## Building

### Got Rust?

Before attempting compilation of this program, [install Rust](https://rustup.rs/).

### Linux

```sh
git clone https://github.com/WilliamVenner/ffaudio2json
cd ffaudio2json
sudo apt update
sudo apt install -y clang libavcodec-dev libavformat-dev libavutil-dev pkg-config
cargo build --release
```

Your compiled binary can be found in `target/release/ffaudio2json`

### macOS

```sh
git clone https://github.com/WilliamVenner/ffaudio2json
cd ffaudio2json
brew install ffmpeg
cargo build --release
```

Your compiled binary can be found in `target/release/ffaudio2json`

### Windows

```pwsh
git clone https://github.com/WilliamVenner/ffaudio2json
cd ffaudio2json
curl -L -o ffmpeg-release-full-shared.7z "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-full-shared.7z"
7z x ffmpeg-release-full-shared.7z
mkdir ffaudio2json-ffmpeg-release-full-shared
mv ffmpeg-*/* ffaudio2json-ffmpeg-release-full-shared/
set FFMPEG_DIR=%cd%\ffaudio2json-ffmpeg-release-full-shared\
cargo build --release
```

Your compiled binary can be found in `target/release/ffaudio2json.exe`

You will probably need to move the following DLLs from `ffaudio2json-ffmpeg-release-full-shared/bin/` to the same folder as `ffaudio2json.exe`:

- `avcodec-61.dll`
- `avutil-59.dll`
- `avformat-61.dll`
- `swresample-5.dll`

## Feature Flags

| Feature         | Default | Description                                                     |
| --------------- | ------- | --------------------------------------------------------------- |
| `ffmpeg-static` | No      | Statically links ffmpeg instead of linking to shared libraries. |
| `ffmpeg-build`  | No      | Builds ffmpeg from source and statically links to it.           |
