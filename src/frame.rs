/// Frame processing module
use crate::Ffmpeg;
use std::path::Path;
impl Ffmpeg {
    /// Extract the first and last frames of a video file.
    ///
    /// # Arguments
    /// * `source_path` - Path to the source video file
    /// * `output_path` - Output directory path where frames will be saved
    ///
    /// # Returns
    /// * `Ok((String, String))` - Paths of the first and last frame files
    /// * `Err(String)` - Error message if extraction fails
    pub fn extract_first_last_frames(
        &self,
        source_path: &str,
        output_path: &str,
    ) -> Result<(String, String), String> {
        let first_frame = self.extract_first_frame(source_path, output_path)?;
        let last_frame = self.extract_last_frame(source_path, output_path)?;
        Ok((first_frame, last_frame))
    }
    /// Extract the first frame of a video file.
    ///
    /// # Arguments
    /// * `source_path` - Path to the source video file
    /// * `output_path` - Output directory path where the frame will be saved
    ///
    /// # Returns
    /// * `Ok(String)` - Path of the extracted first frame file
    /// * `Err(String)` - Error message if extraction fails
    pub fn extract_first_frame(
        &self,
        source_path: &str,
        output_path: &str,
    ) -> Result<String, String> {
        if !Path::new(source_path).exists() {
            return Err(format!("Source file not found: {}", source_path));
        }
        let frame_path = Self::resolve_frame_path(output_path)?;
        self.get_first_frame(source_path, frame_path.to_str().unwrap(), None, None)?;
        Ok(frame_path.to_string_lossy().to_string())
    }
    /// Extract the last frame of a video file.
    ///
    /// # Arguments
    /// * `source_path` - Path to the source video file
    /// * `output_path` - Output directory path where the frame will be saved
    ///
    /// # Returns
    /// * `Ok(String)` - Path of the extracted last frame file
    /// * `Err(String)` - Error message if extraction fails
    pub fn extract_last_frame(
        &self,
        source_path: &str,
        output_path: &str,
    ) -> Result<String, String> {
        if !Path::new(source_path).exists() {
            return Err(format!("Source file not found: {}", source_path));
        }
        let frame_path = Self::resolve_frame_path(output_path)?;
        self.get_last_frame(source_path, frame_path.to_str().unwrap(), None, None)?;
        Ok(frame_path.to_string_lossy().to_string())
    }
    fn resolve_frame_path(output_path: &str) -> Result<std::path::PathBuf, String> {
        let output = Path::new(output_path);
        let is_directory = output.is_dir()
            || output_path.ends_with(std::path::MAIN_SEPARATOR)
            || output_path.ends_with('/')
            || output_path.ends_with('\\');
        if is_directory {
            if !output.exists() {
                std::fs::create_dir_all(output)
                    .map_err(|e| format!("Failed to create output directory: {}", e))?;
            }
            Ok(output.join(format!("{}.png", uuid::Uuid::new_v4().to_string())))
        } else {
            if let Some(parent) = output.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create output directory: {}", e))?;
                }
            }
            Ok(output.to_path_buf())
        }
    }
}
