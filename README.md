<h1 align="center">
    Ffmpegx
</h1>
<h4 align="center">
Rust bindings for FFmpeg, providing common features such as frame sequence decoding and PCM data encoding/decoding.
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/ffmpegx/LICENSE"><img src="https://img.shields.io/badge/Apache2.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## Examples

### 1. Get Video Metadata and Generate Thumbnail

```rust
use ffmpegx::{Ffmpeg, ThumbnailOptions};

fn process_video(video_path: &str) -> Result<(), String> {
// Create Ffmpeg instance (automatically detects ffmpeg path)
let ffmpeg = Ffmpeg::new();

// Get video metadata
let metadata = ffmpeg.get_video_metadata(video_path)?;

println!("Video Information:");
println!(" Duration: {:.2}s", metadata.duration);
println!(" Resolution: {}x{}", metadata.width, metadata.height);
println!(" FPS: {:.2}", metadata.fps);
println!(" Codec: {}", metadata.codec);
println!(" Has Audio: {}", metadata.has_audio);

// Generate thumbnail at 10% of video duration
let thumb_options = ThumbnailOptions {
time: metadata.duration \* 0.1,
width: Some(640),
height: Some(360),
output_path: Some("thumbnail.jpg".to_string()),
};

let thumb_path = ffmpeg.generate_thumbnail(video_path, &thumb_options)?;
println!("Thumbnail saved: {}", thumb_path);

Ok(())
}
```

### 2. Extract Frame Sequence from Video

```rust
use ffmpegx::{Ffmpeg, DecodeVideoOptions};
use std::path::Path;

fn extract_video_frames(video_path: &str, output_dir: &str) -> Result<Vec<String>, String> {
let ffmpeg = Ffmpeg::new();

let frames_dir = Path::new(output_dir).join("frames");
let audio_path = Path::new(output_dir).join("audio.wav");

let options = DecodeVideoOptions {
fps: 30.0,
duration: 10.0,
width: 1280.0,
height: 720.0,
quality: 8,
extract_audio: true,
start_time: None,
};

ffmpeg.decode_video(video_path, &frames_dir, &audio_path, &options)?;

let (frame_count, frame_paths) = ffmpeg.get_frame_sequence_info(&frames_dir)?;
println!("Extracted {} frames", frame_count);

Ok(frame_paths)
}
```

### 3. Extract PCM Audio Data and Generate Waveform

```rust
use ffmpegx::{
extract_audio_pcm_data_from_path,
generate_waveform_image,
Ffmpeg,
};
use std::path::Path;

fn process_audio(audio_path: &str) -> Result<Vec<f32>, String> {
// Extract PCM samples from audio
let pcm_samples = extract_audio_pcm_data_from_path(
Path::new(audio_path),
0.0, // start_time
30.0, // duration
)?;

println!("PCM Samples extracted: {}", pcm_samples.len());

// Analyze audio
if !pcm_samples.is_empty() {
let max_peak = pcm_samples.iter()
.map(|&s| s.abs())
.fold(0.0_f32, |a, b| a.max(b));

println!("Max Peak: {:.4}", max_peak);
}

// Generate waveform image
let ffmpeg = Ffmpeg::new();
generate_waveform_image(
&ffmpeg,
audio_path,
Path::new("waveform.png"),
1200,
200,
"#2d8a6e",
)?;

Ok(pcm_samples)
}
```

### 4. Full Video Processing Pipeline

```rust
use ffmpegx::{
extract_audio_pcm_data_from_path,
generate_waveform_image,
Ffmpeg,
ThumbnailOptions,
DecodeVideoOptions,
};
use std::path::Path;

fn process_video_complete(video_path: &str) -> Result<(), String> {
let ffmpeg = Ffmpeg::new();
println!("FFmpeg path: {}", ffmpeg.bin_path);

// 1. Get metadata
let metadata = ffmpeg.get_video_metadata(video_path)?;
println!("\n=== Video Metadata ===");
println!("Duration: {:.2}s", metadata.duration);
println!("Resolution: {}x{}", metadata.width, metadata.height);
println!("FPS: {}", metadata.fps);

// 2. Generate thumbnail
let thumb_options = ThumbnailOptions {
time: 5.0,
width: Some(640),
height: Some(360),
output_path: Some("thumbnail.jpg".to_string()),
};
ffmpeg.generate_thumbnail(video_path, &thumb_options)?;

// 3. Extract audio PCM data
let pcm_data = extract_audio_pcm_data_from_path(
Path::new(video_path),
0.0,
10.0,
)?;

if !pcm_data.is_empty() {
let avg = pcm_data.iter().map(|&s| s.abs()).sum::<f32>() / pcm_data.len() as f32;
let max = pcm_data.iter().map(|&s| s.abs()).fold(0.0, f32::max);
println!("\n=== Audio Analysis ===");
println!("Samples: {}", pcm_data.len());
println!("Average amplitude: {:.4}", avg);
println!("Peak amplitude: {:.4}", max);
}

// 4. Generate waveform
generate_waveform_image(&ffmpeg, video_path, Path::new("waveform.png"), 1200, 300, "#00ff00")?;

Ok(())
}
```
