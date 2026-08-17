//! Video encoding module for ffmpegx
//!
//! This module provides video encoding capabilities for exporting
//! frame sequences to video files or animated GIFs.
use crate::{Ffmpeg, HwAccel};
use std::path::{Path, PathBuf};
use std::process::Command;
/// Video encoding format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    H264,
    H265,
    VP9,
    AV1,
    ProRes,
    DNxHD,
}
impl VideoCodec {
    pub fn as_str(&self) -> &'static str {
        match self {
            VideoCodec::H264 => "h264",
            VideoCodec::H265 => "h265",
            VideoCodec::VP9 => "vp9",
            VideoCodec::AV1 => "av1",
            VideoCodec::ProRes => "prores",
            VideoCodec::DNxHD => "dnxhd",
        }
    }
    pub fn file_extension(&self) -> &'static str {
        match self {
            VideoCodec::H264 | VideoCodec::H265 => "mp4",
            VideoCodec::VP9 => "webm",
            VideoCodec::AV1 => "mp4",
            VideoCodec::ProRes => "mov",
            VideoCodec::DNxHD => "mxf",
        }
    }
    pub fn ffmpeg_codec_name(&self, hwaccel: Option<HwAccel>) -> &'static str {
        match self {
            VideoCodec::H264 => match hwaccel {
                Some(HwAccel::Cuda) => "h264_nvenc",
                Some(HwAccel::Qsv) => "h264_qsv",
                Some(HwAccel::Vaapi) => "h264_vaapi",
                Some(HwAccel::Amf) => "h264_amf",
                _ => "libx264",
            },
            VideoCodec::H265 => match hwaccel {
                Some(HwAccel::Cuda) => "hevc_nvenc",
                Some(HwAccel::Qsv) => "hevc_qsv",
                Some(HwAccel::Vaapi) => "hevc_vaapi",
                Some(HwAccel::Amf) => "hevc_amf",
                _ => "libx265",
            },
            VideoCodec::VP9 => "libvpx-vp9",
            VideoCodec::AV1 => "libaom-av1",
            VideoCodec::ProRes => "prores_ks",
            VideoCodec::DNxHD => "dnxhd",
        }
    }
}
/// Encoder preset for speed vs compression trade-off
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncoderPreset {
    UltraFast,
    SuperFast,
    VeryFast,
    Faster,
    Fast,
    Medium,
    Slow,
    Slower,
    VerySlow,
    Placebo,
}
impl EncoderPreset {
    pub fn as_str(&self) -> &'static str {
        match self {
            EncoderPreset::UltraFast => "ultrafast",
            EncoderPreset::SuperFast => "superfast",
            EncoderPreset::VeryFast => "veryfast",
            EncoderPreset::Faster => "faster",
            EncoderPreset::Fast => "fast",
            EncoderPreset::Medium => "medium",
            EncoderPreset::Slow => "slow",
            EncoderPreset::Slower => "slower",
            EncoderPreset::VerySlow => "veryslow",
            EncoderPreset::Placebo => "placebo",
        }
    }
}
impl Default for EncoderPreset {
    fn default() -> Self {
        EncoderPreset::Medium
    }
}
/// Video encoding options for export
#[derive(Debug, Clone)]
pub struct VideoEncodeOptions {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub codec: VideoCodec,
    pub bit_rate: u64,
    pub crf: u8,
    pub preset: EncoderPreset,
    pub hwaccel: HwAccel,
    pub pixel_format: String,
    pub extra_args: Vec<String>,
}
impl Default for VideoEncodeOptions {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 30.0,
            codec: VideoCodec::H264,
            bit_rate: 5_000_000,
            crf: 23,
            preset: EncoderPreset::Medium,
            hwaccel: HwAccel::None,
            pixel_format: "yuv420p".to_string(),
            extra_args: Vec::new(),
        }
    }
}
/// GIF encoding options for export
#[derive(Debug, Clone)]
pub struct GifEncodeOptions {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub quality: String,
    pub dither: bool,
    pub loop_animation: bool,
    pub max_colors: u16,
}
impl Default for GifEncodeOptions {
    fn default() -> Self {
        Self {
            width: 854,
            height: 480,
            fps: 24.0,
            quality: "standard".to_string(),
            dither: true,
            loop_animation: true,
            max_colors: 256,
        }
    }
}
/// Helper struct for frame sequence encoding
pub struct FrameSequence {
    pub frames_dir: PathBuf,
    pub pattern: String,
    pub frame_count: u64,
}
impl FrameSequence {
    /// Create a new frame sequence from a directory containing PNG frames
    pub fn from_dir(frames_dir: &Path) -> Result<Self, String> {
        if !frames_dir.exists() {
            return Err(format!("Frames directory not found: {:?}", frames_dir));
        }
        let mut frame_count = 0;
        if let Ok(entries) = std::fs::read_dir(frames_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        if ext == "png" {
                            frame_count += 1;
                        }
                    }
                }
            }
        }
        if frame_count == 0 {
            return Err(format!("No PNG frame files found in: {:?}", frames_dir));
        }
        Ok(Self {
            frames_dir: frames_dir.to_path_buf(),
            pattern: "%06d.png".to_string(),
            frame_count,
        })
    }
    /// Get the frame input pattern for ffmpeg
    pub fn get_input_pattern(&self) -> String {
        self.frames_dir
            .join(&self.pattern)
            .to_string_lossy()
            .to_string()
    }
}
impl Ffmpeg {
    /// Encode a frame sequence to a video file
    pub fn encode_frames_to_video(
        &self,
        frame_sequence: &FrameSequence,
        output_path: &Path,
        options: &VideoEncodeOptions,
    ) -> Result<(), String> {
        if frame_sequence.frame_count == 0 {
            return Err("No frames to encode".to_string());
        }
        if output_path.exists() {
            let _ = std::fs::remove_file(output_path);
        }
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create output directory: {}", e))?;
            }
        }
        let mut cmd = Command::new(&self.bin_path);
        if options.hwaccel != HwAccel::None {
            cmd.arg("-hwaccel").arg(options.hwaccel.as_str());
        }
        cmd.arg("-framerate").arg(options.fps.to_string());
        cmd.arg("-i").arg(frame_sequence.get_input_pattern());
        let mut vf_parts = Vec::new();
        vf_parts.push(format!(
            "scale={}:{}:flags=lanczos",
            options.width, options.height
        ));
        vf_parts.push(format!("format={}", options.pixel_format));
        if !vf_parts.is_empty() {
            cmd.arg("-vf").arg(vf_parts.join(","));
        }
        let codec_name = options.codec.ffmpeg_codec_name(Some(options.hwaccel));
        cmd.arg("-c:v").arg(codec_name);
        match options.codec {
            VideoCodec::H264 | VideoCodec::H265 | VideoCodec::VP9 | VideoCodec::AV1 => {
                cmd.arg("-preset").arg(options.preset.as_str());
                cmd.arg("-crf").arg(options.crf.to_string());
                if options.bit_rate > 0 {
                    cmd.arg("-b:v").arg(options.bit_rate.to_string());
                }
            }
            VideoCodec::ProRes => {
                cmd.arg("-profile:v").arg("3");
                if options.bit_rate > 0 {
                    cmd.arg("-b:v").arg(options.bit_rate.to_string());
                }
            }
            VideoCodec::DNxHD => {
                let bitrate_k = options.bit_rate / 1000;
                let profile = if bitrate_k >= 100_000 {
                    "dnxhr_hqx"
                } else if bitrate_k >= 50_000 {
                    "dnxhr_hq"
                } else {
                    "dnxhd"
                };
                cmd.arg("-profile:v").arg(profile);
                if options.bit_rate > 0 {
                    cmd.arg("-b:v").arg(options.bit_rate.to_string());
                }
            }
        }
        for arg in &options.extra_args {
            cmd.arg(arg);
        }
        cmd.arg("-y").arg(output_path.to_str().unwrap());
        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute ffmpeg for video encoding: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("FFmpeg video encoding failed: {}", stderr));
        }
        if !output_path.exists() {
            return Err("Output video file was not created".to_string());
        }
        Ok(())
    }
    /// Encode a frame sequence to an animated GIF
    pub fn encode_frames_to_gif(
        &self,
        frame_sequence: &FrameSequence,
        output_path: &Path,
        options: &GifEncodeOptions,
    ) -> Result<(), String> {
        if frame_sequence.frame_count == 0 {
            return Err("No frames to encode".to_string());
        }
        if output_path.exists() {
            let _ = std::fs::remove_file(output_path);
        }
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create output directory: {}", e))?;
            }
        }
        let palette_path = output_path.with_extension("palette.png");
        // Step 1: Generate palette
        let mut palette_cmd = Command::new(&self.bin_path);
        palette_cmd
            .arg("-framerate")
            .arg(options.fps.to_string())
            .arg("-i")
            .arg(frame_sequence.get_input_pattern());
        let mut filter_parts = Vec::new();
        filter_parts.push(format!("fps={}", options.fps));
        filter_parts.push(format!(
            "scale={}:{}:flags=lanczos",
            options.width, options.height
        ));
        let palettegen_filter = if options.max_colors < 256 {
            format!(
                "palettegen=max_colors={}:stats_mode=diff",
                options.max_colors
            )
        } else {
            "palettegen=stats_mode=diff".to_string()
        };
        filter_parts.push(palettegen_filter);
        palette_cmd
            .arg("-vf")
            .arg(filter_parts.join(","))
            .arg("-y")
            .arg(&palette_path);
        let palette_output = palette_cmd
            .output()
            .map_err(|e| format!("Failed to generate GIF palette: {}", e))?;
        if !palette_output.status.success() {
            let stderr = String::from_utf8_lossy(&palette_output.stderr);
            return Err(format!("FFmpeg palette generation failed: {}", stderr));
        }
        // Step 2: Encode GIF
        let mut gif_cmd = Command::new(&self.bin_path);
        gif_cmd
            .arg("-framerate")
            .arg(options.fps.to_string())
            .arg("-i")
            .arg(frame_sequence.get_input_pattern())
            .arg("-i")
            .arg(&palette_path);
        let mut gif_filter_parts = Vec::new();
        gif_filter_parts.push(format!("fps={}", options.fps));
        gif_filter_parts.push(format!(
            "scale={}:{}:flags=lanczos",
            options.width, options.height
        ));
        let paletteuse_filter = if options.dither {
            "paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle".to_string()
        } else {
            "paletteuse=dither=none".to_string()
        };
        gif_filter_parts.push(paletteuse_filter);
        gif_cmd.arg("-vf").arg(gif_filter_parts.join(","));
        if options.loop_animation {
            gif_cmd.arg("-loop").arg("0");
        } else {
            gif_cmd.arg("-loop").arg("-1");
        }
        gif_cmd.arg("-y").arg(output_path.to_str().unwrap());
        let gif_output = gif_cmd
            .output()
            .map_err(|e| format!("Failed to encode GIF: {}", e))?;
        let _ = std::fs::remove_file(&palette_path);
        if !gif_output.status.success() {
            let stderr = String::from_utf8_lossy(&gif_output.stderr);
            return Err(format!("FFmpeg GIF encoding failed: {}", stderr));
        }
        if !output_path.exists() {
            return Err("Output GIF file was not created".to_string());
        }
        Ok(())
    }
    /// Encode a frame sequence to video or GIF based on file extension
    pub fn encode_frames(
        &self,
        frame_sequence: &FrameSequence,
        output_path: &Path,
        video_options: &VideoEncodeOptions,
        gif_options: &GifEncodeOptions,
        force_gif: bool,
    ) -> Result<(), String> {
        let is_gif = force_gif
            || output_path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_lowercase() == "gif")
                .unwrap_or(false);
        if is_gif {
            self.encode_frames_to_gif(frame_sequence, output_path, gif_options)
        } else {
            self.encode_frames_to_video(frame_sequence, output_path, video_options)
        }
    }
}
