use crate::{OutputFormatTypeEnum, PreviewQuality};
/// Default decode frame quality for frame extraction
pub const DEFAULT_DECODE_FRAME_QUALITY: PreviewQuality = PreviewQuality::Excellent;
/// Output frame product format
pub const OUTPUT_FRAME_PROFUCT_FORMAT: OutputFormatTypeEnum = OutputFormatTypeEnum::Jpg;
/// default fps
pub const DEFAULT_FPS: f64 = 30.0;
/// Preview Canvas Width
/// 854
/// 1280
/// 1920
pub const DEFAULT_FRAME_WIDTH: f64 = 1280.0;
/// Preview Canvas Height
/// 480
/// 720
/// 1080
pub const DEFAULT_FRAME_HEIGHT: f64 = 720.0;
/// Default audio sample rate
pub const DEFAULT_AUDIO_SAMPLING_RATE: u32 = 44100;
/// gif max frames
pub const GIF_MAX_FRAMES: u64 = 300;
