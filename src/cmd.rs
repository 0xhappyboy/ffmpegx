use std::process::Command;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;
/// Windows: BELOW_NORMAL_PRIORITY_CLASS.
/// Makes the child process yield CPU to the UI / WebView2 process.
#[cfg(target_os = "windows")]
const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x00004000;
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
/// Create a hidden command with low CPU priority.
pub fn low_priority_cmd(program: &str) -> Command {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new(program);
        cmd.creation_flags(CREATE_NO_WINDOW | BELOW_NORMAL_PRIORITY_CLASS);
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    {
        let mut cmd = Command::new("nice");
        cmd.arg("-n").arg("10").arg(program);
        cmd
    }
}
pub fn cmd_ffmpeg() -> Command {
    hidden_cmd("ffmpeg")
}
pub fn cmd_ffprobe() -> Command {
    hidden_cmd("ffprobe")
}