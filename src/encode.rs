//! Video encoding module for ffmpegx
//!
//! This module provides video encoding capabilities for exporting
//! frame sequences to video files or animated GIFs.
use crate::{Ffmpeg, FileUtils, HwAccel};
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
        if !FileUtils::path_exists(frames_dir) {
            return Err(format!("Frames directory not found: {:?}", frames_dir));
        }
        let mut frame_count = 0;
        let mut pattern = "%06d.png".to_string();
        if let Ok(entries) = FileUtils::read_dir_entries(frames_dir) {
            for entry in entries {
                let path = entry.path();
                if path.is_file() {
                    if let Some(ext) = path.extension() {
                        let ext_str = ext.to_string_lossy().to_lowercase();
                        if ext_str == "png" || ext_str == "jpg" || ext_str == "jpeg" {
                            frame_count += 1;
                            if ext_str == "png" {
                                pattern = "%06d.png".to_string();
                            } else if ext_str == "jpg" || ext_str == "jpeg" {
                                pattern = "%06d.jpg".to_string();
                            }
                        }
                    }
                }
            }
        }
        if frame_count == 0 {
            return Err(format!("No frame files found in: {:?}", frames_dir));
        }
        Ok(Self {
            frames_dir: frames_dir.to_path_buf(),
            pattern,
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
        // Remove existing output file if it exists
        if FileUtils::path_exists(output_path) {
            let _ = FileUtils::remove_file(output_path);
        }
        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            if !FileUtils::path_exists(parent) {
                FileUtils::ensure_dir(parent)
                    .map_err(|e| format!("Failed to create output directory: {:?}", e))?;
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
        if !FileUtils::path_exists(output_path) {
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
        log::debug!(
            "encode_frames_to_gif - START: frames_dir={:?}, frame_count={}, output_path={:?}, fps={}",
            frame_sequence.frames_dir,
            frame_sequence.frame_count,
            output_path,
            options.fps
        );
        if frame_sequence.frame_count == 0 {
            log::error!("encode_frames_to_gif - No frames to encode");
            return Err("No frames to encode".to_string());
        }
        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            if !FileUtils::path_exists(parent) {
                log::debug!(
                    "encode_frames_to_gif - Creating output directory: {:?}",
                    parent
                );
                FileUtils::ensure_dir(parent)
                    .map_err(|e| format!("Failed to create output directory: {:?}", e))?;
            }
        }
        // Remove existing output file if it exists
        if FileUtils::path_exists(output_path) {
            log::debug!(
                "encode_frames_to_gif - Removing existing output file: {:?}",
                output_path
            );
            let _ = FileUtils::remove_file(output_path);
        }
        let palette_path = output_path.with_extension("palette.png");
        log::debug!("encode_frames_to_gif - Palette path: {:?}", palette_path);
        let input_pattern = frame_sequence.get_input_pattern();
        log::debug!("encode_frames_to_gif - Step 1: Generating palette...");
        let palette_filter = if options.max_colors < 256 {
            format!(
                "fps={},scale={}:{}:flags=lanczos,palettegen=max_colors={}:stats_mode=diff",
                options.fps, options.width, options.height, options.max_colors
            )
        } else {
            format!(
                "fps={},scale={}:{}:flags=lanczos,palettegen=stats_mode=diff",
                options.fps, options.width, options.height
            )
        };
        let palette_output_cmd = Command::new(&self.bin_path)
            .arg("-framerate")
            .arg(options.fps.to_string())
            .arg("-i")
            .arg(&input_pattern)
            .arg("-vf")
            .arg(&palette_filter)
            .arg("-y")
            .arg(&palette_path)
            .output()
            .map_err(|e| format!("Failed to generate GIF palette: {}", e))?;
        if !palette_output_cmd.status.success() {
            let stderr = String::from_utf8_lossy(&palette_output_cmd.stderr);
            log::error!(
                "encode_frames_to_gif - Palette generation failed: {}",
                stderr
            );
            return Err(format!("FFmpeg palette generation failed: {}", stderr));
        }
        if !FileUtils::path_exists(&palette_path) {
            log::error!(
                "encode_frames_to_gif - Palette file not created: {:?}",
                palette_path
            );
            return Err(format!("Palette file not created: {:?}", palette_path));
        }
        log::debug!(
            "encode_frames_to_gif - Palette generated: {:?}",
            palette_path
        );
        log::debug!("encode_frames_to_gif - Step 2: Encoding GIF...");
        let gif_filter = if options.dither {
            format!(
                "[0:v]fps={},scale={}:{}:flags=lanczos[scaled];[scaled][1:v]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle",
                options.fps, options.width, options.height
            )
        } else {
            format!(
                "[0:v]fps={},scale={}:{}:flags=lanczos[scaled];[scaled][1:v]paletteuse=dither=none",
                options.fps, options.width, options.height
            )
        };
        let gif_output_cmd = Command::new(&self.bin_path)
            .arg("-framerate")
            .arg(options.fps.to_string())
            .arg("-i")
            .arg(&input_pattern)
            .arg("-i")
            .arg(&palette_path)
            .arg("-filter_complex")
            .arg(&gif_filter)
            .arg("-loop")
            .arg(if options.loop_animation { "0" } else { "-1" })
            .arg("-y")
            .arg(output_path.to_str().unwrap())
            .output()
            .map_err(|e| format!("Failed to encode GIF: {}", e))?;
        // Clean up palette file
        let _ = FileUtils::remove_file(&palette_path);
        log::debug!("encode_frames_to_gif - Palette file cleaned up");
        if !gif_output_cmd.status.success() {
            let stderr = String::from_utf8_lossy(&gif_output_cmd.stderr);
            log::error!("encode_frames_to_gif - GIF encoding failed: {}", stderr);
            return Err(format!("FFmpeg GIF encoding failed: {}", stderr));
        }
        // Verify output file was created
        if !FileUtils::path_exists(output_path) {
            log::error!(
                "encode_frames_to_gif - Output GIF file was not created: {:?}",
                output_path
            );
            return Err("Output GIF file was not created".to_string());
        }
        let file_size = FileUtils::get_file_size(output_path)
            .map_err(|e| format!("Failed to get output file size: {:?}", e))?;
        if file_size == 0 {
            log::error!(
                "encode_frames_to_gif - Output GIF file is empty: {:?}",
                output_path
            );
            return Err("Output GIF file is empty".to_string());
        }
        log::debug!(
            "encode_frames_to_gif - DONE: output_path={:?}, size={} bytes",
            output_path,
            file_size
        );
        Ok(())
    }
    /// Encodes PCM audio to the target format using ffmpeg
    ///
    /// # Arguments
    /// * `input_path` - Path to the PCM file (f32le format)
    /// * `output_path` - Path to the output audio file
    /// * `format` - Output audio format ("aac", "mp3", "flac", "wav", "opus")
    /// * `sample_rate` - Sample rate in Hz
    /// * `channels` - Number of audio channels
    /// * `bitrate` - Bitrate in bits per second
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok on success, Err on failure
    pub fn encode_pcm_to_audio(
        &self,
        input_path: &Path,
        output_path: &Path,
        format: &str,
        sample_rate: u32,
        channels: u16,
        bitrate: u32,
    ) -> Result<(), String> {
        if !FileUtils::path_exists(input_path) {
            return Err(format!("PCM file not found: {:?}", input_path));
        }
        if let Some(parent) = output_path.parent() {
            if !FileUtils::path_exists(parent) {
                FileUtils::ensure_dir(parent)
                    .map_err(|e| format!("Failed to create output directory: {:?}", e))?;
            }
        }
        let mut cmd = Command::new(&self.bin_path);
        cmd.arg("-y")
            .arg("-f")
            .arg("f32le")
            .arg("-ar")
            .arg(sample_rate.to_string())
            .arg("-ac")
            .arg(channels.to_string())
            .arg("-i")
            .arg(input_path.to_str().unwrap());
        match format {
            "mp3" => {
                let bitrate_k = if bitrate > 0 { bitrate / 1000 } else { 192 };
                cmd.arg("-acodec")
                    .arg("libmp3lame")
                    .arg("-b:a")
                    .arg(format!("{}k", bitrate_k));
            }
            "aac" => {
                let bitrate_k = if bitrate > 0 { bitrate / 1000 } else { 192 };
                cmd.arg("-acodec")
                    .arg("aac")
                    .arg("-b:a")
                    .arg(format!("{}k", bitrate_k));
            }
            "wav" => {
                cmd.arg("-acodec").arg("pcm_s16le");
            }
            "flac" => {
                cmd.arg("-acodec")
                    .arg("flac")
                    .arg("-compression_level")
                    .arg("8");
            }
            "opus" => {
                let bitrate_k = if bitrate > 0 { bitrate / 1000 } else { 128 };
                cmd.arg("-acodec")
                    .arg("libopus")
                    .arg("-b:a")
                    .arg(format!("{}k", bitrate_k));
            }
            _ => {
                let bitrate_k = if bitrate > 0 { bitrate / 1000 } else { 192 };
                cmd.arg("-acodec")
                    .arg("aac")
                    .arg("-b:a")
                    .arg(format!("{}k", bitrate_k));
            }
        }
        cmd.arg(output_path.to_str().unwrap());
        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute ffmpeg for PCM encoding: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("FFmpeg PCM encoding failed: {}", stderr));
        }
        if !FileUtils::path_exists(output_path) {
            return Err("Encoded audio file was not created".to_string());
        }
        Ok(())
    }
    /// Encode PCM audio to target audio format
    ///
    /// This function reads raw PCM data (f32le format) and encodes it to
    /// the specified audio format using FFmpeg.
    ///
    /// # Arguments
    /// * `input_path` - Path to the PCM file (f32le format)
    /// * `output_path` - Path where the encoded audio file will be saved
    /// * `format` - Target audio format: "aac", "mp3", "flac", "wav", or "opus"
    /// * `sample_rate` - Sample rate in Hz (e.g., 44100)
    /// * `channels` - Number of audio channels (1=mono, 2=stereo)
    /// * `bitrate` - Bitrate in bits per second (e.g., 192000)
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok on success, Err on failure
    pub fn encode_audio_to_format(
        &self,
        input_path: &Path,
        output_path: &Path,
        format: &str,
        sample_rate: u32,
        channels: u16,
        bitrate: u32,
    ) -> Result<(), String> {
        if !FileUtils::path_exists(input_path) {
            return Err(format!("PCM file not found: {:?}", input_path));
        }
        if let Some(parent) = output_path.parent() {
            if !FileUtils::path_exists(parent) {
                FileUtils::ensure_dir(parent)
                    .map_err(|e| format!("Failed to create output directory: {:?}", e))?;
            }
        }
        let mut cmd = Command::new(&self.bin_path);
        cmd.arg("-y")
            .arg("-f")
            .arg("f32le")
            .arg("-ar")
            .arg(sample_rate.to_string())
            .arg("-ac")
            .arg(channels.to_string())
            .arg("-i")
            .arg(input_path.to_str().unwrap());
        match format {
            "mp3" => {
                let bitrate_k = if bitrate > 0 { bitrate / 1000 } else { 192 };
                cmd.arg("-acodec")
                    .arg("libmp3lame")
                    .arg("-b:a")
                    .arg(format!("{}k", bitrate_k));
            }
            "aac" => {
                let bitrate_k = if bitrate > 0 { bitrate / 1000 } else { 192 };
                cmd.arg("-acodec")
                    .arg("aac")
                    .arg("-b:a")
                    .arg(format!("{}k", bitrate_k));
            }
            "wav" => {
                cmd.arg("-acodec").arg("pcm_s16le");
            }
            "flac" => {
                cmd.arg("-acodec")
                    .arg("flac")
                    .arg("-compression_level")
                    .arg("8");
            }
            "opus" => {
                let bitrate_k = if bitrate > 0 { bitrate / 1000 } else { 128 };
                cmd.arg("-acodec")
                    .arg("libopus")
                    .arg("-b:a")
                    .arg(format!("{}k", bitrate_k));
            }
            _ => {
                let bitrate_k = if bitrate > 0 { bitrate / 1000 } else { 192 };
                cmd.arg("-acodec")
                    .arg("aac")
                    .arg("-b:a")
                    .arg(format!("{}k", bitrate_k));
            }
        }
        cmd.arg(output_path.to_str().unwrap());
        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("FFmpeg encoding failed: {}", stderr));
        }
        if !FileUtils::path_exists(output_path) {
            return Err("Encoded audio file was not created".to_string());
        }
        Ok(())
    }
    /// Merge audio stream into video file
    ///
    /// Takes a video file and an audio file, and produces a new file
    /// with both streams combined.
    ///
    /// # Arguments
    /// * `video_path` - Path to the video file (without audio)
    /// * `audio_path` - Path to the audio file (must be a recognized format like AAC, MP3, etc.)
    /// * `output_path` - Path where the merged file will be saved
    ///
    /// # Returns
    /// * `Ok(())` on success
    /// * `Err(String)` - Error message on failure
    ///
    /// # Note
    /// This function expects the audio file to be in a format that FFmpeg can
    /// recognize automatically (e.g., AAC, MP3, WAV). For raw PCM data, use
    /// `merge_pcm_into_video` instead.
    pub fn merge_audio_into_video(
        &self,
        video_path: &Path,
        audio_path: &Path,
        output_path: &Path,
    ) -> Result<(), String> {
        if !FileUtils::path_exists(video_path) {
            return Err(format!("Video file not found: {:?}", video_path));
        }
        if !FileUtils::path_exists(audio_path) {
            return Err(format!("Audio file not found: {:?}", audio_path));
        }
        if let Some(parent) = output_path.parent() {
            if !FileUtils::path_exists(parent) {
                FileUtils::ensure_dir(parent)
                    .map_err(|e| format!("Failed to create output directory: {:?}", e))?;
            }
        }
        let output = Command::new(&self.bin_path)
            .args([
                "-y",
                "-i",
                video_path.to_str().unwrap(),
                "-i",
                audio_path.to_str().unwrap(),
                "-c:v",
                "copy",
                "-c:a",
                "aac",
                "-map",
                "0:v:0",
                "-map",
                "1:a:0",
                "-shortest",
                output_path.to_str().unwrap(),
            ])
            .output()
            .map_err(|e| format!("Failed to execute ffmpeg for audio merge: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("FFmpeg audio merge failed: {}", stderr));
        }
        if !FileUtils::path_exists(output_path) {
            return Err("Merged output file was not created".to_string());
        }
        Ok(())
    }
    /// Directly merge PCM audio into a video file without intermediate encoding
    ///
    /// This function takes a video file and a PCM audio file (f32le format) and
    /// merges them into a single output file using FFmpeg. The PCM audio is
    /// encoded directly to the target format during the merge process, eliminating
    /// the need for a separate encoding step.
    ///
    /// # Arguments
    /// * `video_path` - Path to the video file (without audio)
    /// * `pcm_path` - Path to the PCM audio file (f32le format)
    /// * `output_path` - Path where the merged file will be saved
    /// * `audio_format` - Target audio format as string ("aac", "mp3", "flac", "wav", "opus")
    /// * `sample_rate` - Audio sample rate in Hz (e.g., 44100)
    /// * `channels` - Number of audio channels (1=mono, 2=stereo)
    /// * `bitrate` - Audio bitrate in bits per second (e.g., 192000)
    ///
    /// # Returns
    /// * `Result<(), String>` - Ok on success, Err on failure
    pub fn merge_pcm_into_video(
        &self,
        video_path: &Path,
        pcm_path: &Path,
        output_path: &Path,
        audio_format: &str,
        sample_rate: u32,
        channels: u16,
        bitrate: u32,
    ) -> Result<(), String> {
        // Validate input files exist
        if !FileUtils::path_exists(video_path) {
            return Err(format!("Video file not found: {:?}", video_path));
        }
        if !FileUtils::path_exists(pcm_path) {
            return Err(format!("PCM file not found: {:?}", pcm_path));
        }
        // Ensure output directory exists
        if let Some(parent) = output_path.parent() {
            if !FileUtils::path_exists(parent) {
                FileUtils::ensure_dir(parent)
                    .map_err(|e| format!("Failed to create output directory: {:?}", e))?;
            }
        }
        // If output path is the same as video path, use a temporary file first
        let temp_output = if output_path == video_path {
            let temp_path = output_path.with_extension("tmp_merge.mp4");
            Some(temp_path)
        } else {
            None
        };
        let final_output = if let Some(temp) = &temp_output {
            temp.as_path()
        } else {
            output_path
        };
        // Get audio codec and bitrate
        let (codec, bitrate_k) = match audio_format {
            "mp3" => ("libmp3lame", if bitrate > 0 { bitrate / 1000 } else { 192 }),
            "aac" => ("aac", if bitrate > 0 { bitrate / 1000 } else { 192 }),
            "wav" => ("pcm_s16le", 0),
            "flac" => ("flac", 0),
            "opus" => ("libopus", if bitrate > 0 { bitrate / 1000 } else { 128 }),
            _ => ("aac", if bitrate > 0 { bitrate / 1000 } else { 192 }),
        };
        // Build FFmpeg command
        // Input order: 0=video, 1=PCM audio
        let mut cmd = std::process::Command::new(&self.bin_path);
        cmd.arg("-y")
            .arg("-i")
            .arg(video_path.to_str().unwrap())
            .arg("-f")
            .arg("f32le")
            .arg("-ar")
            .arg(sample_rate.to_string())
            .arg("-ac")
            .arg(channels.to_string())
            .arg("-i")
            .arg(pcm_path.to_str().unwrap())
            .arg("-c:v")
            .arg("copy")
            .arg("-c:a")
            .arg(codec);
        if bitrate_k > 0 {
            cmd.arg("-b:a").arg(format!("{}k", bitrate_k));
        }
        cmd.arg("-map")
            .arg("0:v:0")
            .arg("-map")
            .arg("1:a:0")
            .arg("-shortest")
            .arg(final_output.to_str().unwrap());
        // Execute FFmpeg command
        let output = cmd
            .output()
            .map_err(|e| format!("Failed to execute ffmpeg for PCM merge: {}", e))?;
        // Check if FFmpeg succeeded
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if let Some(temp_path) = temp_output {
                let _ = FileUtils::remove_file(&temp_path);
            }
            return Err(format!("FFmpeg PCM merge failed: {}", stderr));
        }
        // If we used a temporary file, move it to the final destination
        if let Some(temp_path) = temp_output {
            if FileUtils::path_exists(output_path) && output_path != video_path {
                let _ = FileUtils::remove_file(output_path);
            }
            std::fs::rename(&temp_path, output_path)
                .map_err(|e| format!("Failed to move temp file to output: {}", e))?;
        }
        // Verify the output file was created
        if !FileUtils::path_exists(output_path) {
            return Err("Output file was not created".to_string());
        }
        Ok(())
    }
}
