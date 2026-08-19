use std::process::Command;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;
pub fn hidden_cmd(program: &str) -> Command {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new(program);
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new(program)
    }
}
pub fn cmd_ffmpeg() -> Command {
    hidden_cmd("ffmpeg")
}
pub fn cmd_ffprobe() -> Command {
    hidden_cmd("ffprobe")
}
