<h1 align="center">
    Ffmpegx
</h1>
<h4 align="center">
FFmpeg的Rust绑定,提供帧序列解码和PCM数据编解码等常用功能.
</h4>
<p align="center">
  <a href="https://github.com/0xhappyboy/ffmpegx/LICENSE"><img src="https://img.shields.io/badge/Apache2.0-d1d1f6.svg?style=flat&labelColor=1C2C2E&color=BEC5C9&logo=googledocs&label=license&logoColor=BEC5C9" alt="License"></a>
</p>
<p align="center">
<a href="./README_zh-CN.md">简体中文</a> | <a href="./README.md">English</a>
</p>

## 例子

### 获取视频元数据并生成缩略图

```rust
use ffmpegx::{Ffmpeg, ThumbnailOptions};

fn process_video(video_path: &str) -> Result<(), String> {
// 创建 Ffmpeg 实例（自动检测 ffmpeg 路径）
let ffmpeg = Ffmpeg::new();

// 获取视频元数据
let metadata = ffmpeg.get_video_metadata(video_path)?;

println!("视频信息:");
println!(" 时长: {:.2}s", metadata.duration);
println!(" 分辨率: {}x{}", metadata.width, metadata.height);
println!(" 帧率: {:.2}", metadata.fps);
println!(" 编码: {}", metadata.codec);
println!(" 包含音频: {}", metadata.has_audio);

// 在视频 10% 位置生成缩略图
let thumb_options = ThumbnailOptions {
time: metadata.duration * 0.1,
width: Some(640),
height: Some(360),
output_path: Some("thumbnail.jpg".to_string()),
};

let thumb_path = ffmpeg.generate_thumbnail(video_path, &thumb_options)?;
println!("缩略图已保存: {}", thumb_path);

Ok(())
}
```

### 从视频提取帧序列

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
println!("已提取 {} 帧", frame_count);

Ok(frame_paths)
}
```

### 提取 PCM 音频数据并生成波形图

```rust
use ffmpegx::{
extract_audio_pcm_data_from_path,
generate_waveform_image,
Ffmpeg,
};
use std::path::Path;

fn process_audio(audio_path: &str) -> Result<Vec<f32>, String> {
// 从音频提取 PCM 样本
let pcm_samples = extract_audio_pcm_data_from_path(
Path::new(audio_path),
0.0, // 开始时间
30.0, // 持续时间
)?;

println!("PCM 样本数: {}", pcm_samples.len());

// 分析音频
if !pcm_samples.is_empty() {
let max_peak = pcm_samples.iter()
.map(|&s| s.abs())
.fold(0.0_f32, |a, b| a.max(b));

println!("最大峰值: {:.4}", max_peak);
}

// 生成波形图
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

### 完整的视频处理流程

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
println!("FFmpeg 路径: {}", ffmpeg.bin_path);

// 获取元数据
let metadata = ffmpeg.get_video_metadata(video_path)?;
println!("\n=== 视频元数据 ===");
println!("时长: {:.2}s", metadata.duration);
println!("分辨率: {}x{}", metadata.width, metadata.height);
println!("帧率: {}", metadata.fps);

// 生成缩略图
let thumb_options = ThumbnailOptions {
time: 5.0,
width: Some(640),
height: Some(360),
output_path: Some("thumbnail.jpg".to_string()),
};
ffmpeg.generate_thumbnail(video_path, &thumb_options)?;

// 提取音频 PCM 数据
let pcm_data = extract_audio_pcm_data_from_path(Path::new(video_path), 0.0, 10.0)?;

if !pcm_data.is_empty() {
let avg = pcm_data.iter().map(|&s| s.abs()).sum::<f32>() / pcm_data.len() as f32;
let max = pcm_data.iter().map(|&s| s.abs()).fold(0.0, f32::max);
println!("\n=== 音频分析 ===");
println!("样本数: {}", pcm_data.len());
println!("平均振幅: {:.4}", avg);
println!("峰值振幅: {:.4}", max);
}

// 生成波形图
generate_waveform_image(&ffmpeg, video_path, Path::new("waveform.png"), 1200, 300, "#00ff00",)?;

Ok(())
}
```
