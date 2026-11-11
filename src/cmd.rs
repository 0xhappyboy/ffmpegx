use crate::ProcessPriority;
use std::process::Command;
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;
#[cfg(target_os = "windows")]
const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x00004000;
#[cfg(target_os = "windows")]
const NORMAL_PRIORITY_CLASS: u32 = 0x00000020;
#[cfg(target_os = "windows")]
const ABOVE_NORMAL_PRIORITY_CLASS: u32 = 0x00008000;
/// Hidden command with no priority override (legacy behaviour).
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
/// Hidden command with the given CPU priority.
/// CREATE_NO_WINDOW is always applied first, priority flags are OR-ed on top.
pub fn priority_cmd(program: &str, priority: ProcessPriority) -> Command {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let priority_flag = match priority {
            ProcessPriority::Low => BELOW_NORMAL_PRIORITY_CLASS,
            ProcessPriority::Normal => NORMAL_PRIORITY_CLASS,
            ProcessPriority::High => ABOVE_NORMAL_PRIORITY_CLASS,
        };
        let mut cmd = Command::new(program);
        cmd.creation_flags(CREATE_NO_WINDOW | priority_flag);
        cmd
    }
    #[cfg(not(target_os = "windows"))]
    {
        match priority {
            ProcessPriority::High => {
                let mut cmd = Command::new("nice");
                cmd.arg("-n").arg("-10").arg(program);
                cmd
            }
            ProcessPriority::Normal => Command::new(program),
            ProcessPriority::Low => {
                let mut cmd = Command::new("nice");
                cmd.arg("-n").arg("10").arg(program);
                cmd
            }
        }
    }
}
/// Create a hidden command with low CPU priority (legacy alias).
pub fn low_priority_cmd(program: &str) -> Command {
    priority_cmd(program, ProcessPriority::Low)
}
pub fn cmd_ffmpeg() -> Command {
    hidden_cmd("ffmpeg")
}
pub fn cmd_ffprobe() -> Command {
    hidden_cmd("ffprobe")
}
