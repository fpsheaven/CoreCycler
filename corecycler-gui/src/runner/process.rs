use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, watch};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use crate::config::{CoreCyclerConfig, LinpackVersion, StressTestProgram};

const CREATE_NO_WINDOW: u32 = 0x08000000;

// Windows Job Object FFI - ensures all child processes die when GUI closes
#[cfg(windows)]
mod job {
    use std::ptr;

    // Constants
    const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x00002000;
    const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION: u32 = 9;

    type HANDLE = *mut std::ffi::c_void;
    type BOOL = i32;
    type DWORD = u32;

    #[repr(C)]
    #[derive(Default)]
    struct IO_COUNTERS {
        read_operations: u64,
        write_operations: u64,
        other_operations: u64,
        read_transfer_count: u64,
        write_transfer_count: u64,
        other_transfer_count: u64,
    }

    #[repr(C)]
    #[derive(Default)]
    struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
        per_process_user_time_limit: i64,
        per_job_user_time_limit: i64,
        limit_flags: DWORD,
        minimum_working_set_size: usize,
        maximum_working_set_size: usize,
        active_process_limit: DWORD,
        affinity: usize,
        priority_class: DWORD,
        scheduling_class: DWORD,
    }

    #[repr(C)]
    #[derive(Default)]
    struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
        basic: JOBOBJECT_BASIC_LIMIT_INFORMATION,
        io_info: IO_COUNTERS,
        process_memory_limit: usize,
        job_memory_limit: usize,
        peak_process_memory_used: usize,
        peak_job_memory_used: usize,
    }

    extern "system" {
        fn CreateJobObjectW(attrs: *mut u8, name: *const u16) -> HANDLE;
        fn SetInformationJobObject(
            job: HANDLE,
            class: DWORD,
            info: *const u8,
            len: DWORD,
        ) -> BOOL;
        fn AssignProcessToJobObject(job: HANDLE, process: HANDLE) -> BOOL;
        fn CloseHandle(handle: HANDLE) -> BOOL;
        fn OpenProcess(access: DWORD, inherit: BOOL, pid: DWORD) -> HANDLE;
    }

    pub struct JobObject {
        handle: HANDLE,
    }

    // SAFETY: The job handle is only used from the main thread context
    unsafe impl Send for JobObject {}
    unsafe impl Sync for JobObject {}

    impl JobObject {
        pub fn new() -> Option<Self> {
            unsafe {
                let handle = CreateJobObjectW(ptr::null_mut(), ptr::null());
                if handle.is_null() {
                    return None;
                }

                let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
                info.basic.limit_flags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;

                let ok = SetInformationJobObject(
                    handle,
                    JOB_OBJECT_EXTENDED_LIMIT_INFORMATION as DWORD,
                    &info as *const _ as *const u8,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as DWORD,
                );

                if ok == 0 {
                    CloseHandle(handle);
                    return None;
                }

                Some(JobObject { handle })
            }
        }

        pub fn assign_pid(&self, pid: u32) -> bool {
            unsafe {
                let process = OpenProcess(0x001F0FFF, 0, pid); // PROCESS_ALL_ACCESS
                if process.is_null() {
                    return false;
                }
                let ok = AssignProcessToJobObject(self.handle, process);
                CloseHandle(process);
                ok != 0
            }
        }
    }

    impl Drop for JobObject {
        fn drop(&mut self) {
            // Closing the job handle with KILL_ON_JOB_CLOSE kills ALL processes in the job
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }
}

#[derive(Clone, Debug)]
struct LogMonitorHint {
    prefix: String,
    program: String,
    mode_suffix: String,
}

impl LogMonitorHint {
    fn from_config(config: &CoreCyclerConfig) -> Self {
        let prefix = if config.logging.name.trim().is_empty() {
            "CoreCycler".to_string()
        } else {
            config.logging.name.trim().to_string()
        };

        let mode_suffix = match config.general.stress_test_program {
            StressTestProgram::Prime95 => config.prime95.mode.as_str().to_string(),
            StressTestProgram::Aida64 => normalize_mode_list(&config.aida64.mode),
            StressTestProgram::YCruncher | StressTestProgram::YCruncherOld => {
                config.ycruncher.mode.as_str().to_uppercase()
            }
            StressTestProgram::Linpack => {
                let effective_mode = match config.linpack.version {
                    LinpackVersion::V2018 | LinpackVersion::V2019 => {
                        config.linpack.mode.as_str().to_string()
                    }
                    LinpackVersion::V2021 | LinpackVersion::V2024 => "FASTEST".to_string(),
                };
                format!("{}_{}", config.linpack.version.as_str(), effective_mode)
            }
        };

        Self {
            prefix,
            program: config.general.stress_test_program.as_str().to_string(),
            mode_suffix,
        }
    }

    fn matches_file_name(&self, file_name: &str) -> bool {
        file_name.starts_with(&format!("{}_", self.prefix))
            && file_name.contains(&format!("_{}_", self.program))
            && file_name.ends_with(&format!("_{}.log", self.mode_suffix))
    }

    fn extract_path_from_output_line(&self, line: &str, logs_dir: &Path) -> Option<PathBuf> {
        for token in line.split_whitespace() {
            let cleaned = token.trim_matches(|c: char| {
                matches!(c, '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';')
            });
            if !cleaned.ends_with(".log") {
                continue;
            }

            let file_name = Path::new(cleaned).file_name()?.to_str()?;
            if self.matches_file_name(file_name) {
                return Some(logs_dir.join(file_name));
            }
        }

        None
    }
}

fn normalize_mode_list(mode: &str) -> String {
    let normalized = mode
        .split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| entry.to_uppercase())
        .collect::<Vec<_>>()
        .join("-");

    if normalized.is_empty() {
        "CACHE".to_string()
    } else {
        normalized
    }
}

fn find_matching_log_file(
    logs_dir: &Path,
    hint: &LogMonitorHint,
    spawn_time: std::time::SystemTime,
) -> Option<PathBuf> {
    let mut logs: Vec<_> = std::fs::read_dir(logs_dir)
        .ok()?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !hint.matches_file_name(&file_name) {
                return false;
            }

            let Ok(metadata) = entry.metadata() else {
                return false;
            };
            let Ok(modified) = metadata.modified() else {
                return false;
            };
            modified >= spawn_time
        })
        .collect();

    logs.sort_by(|a, b| {
        let ta = a.metadata().and_then(|m| m.modified()).ok();
        let tb = b.metadata().and_then(|m| m.modified()).ok();
        tb.cmp(&ta)
    });

    logs.first().map(|entry| entry.path())
}

pub struct RunnerHandle {
    child: Child,
    pub pid: u32,
    #[cfg(windows)]
    _job: Option<job::JobObject>,
}

impl RunnerHandle {
    /// Stop the running process tree. Uses /T /F to kill the whole tree
    /// and also kills known stress test programs as fallback.
    pub fn stop(&mut self) {
        if let Some(pid) = self.child.id() {
            // Kill the entire process tree forcefully
            let _ = std::process::Command::new("taskkill")
                .args(["/PID", &pid.to_string(), "/T", "/F"])
                .creation_flags(CREATE_NO_WINDOW)
                .spawn();
        }
        // Also kill any lingering stress test programs
        self.kill_stress_test_programs();
    }

    /// Force kill - uses sync start_kill() instead of async kill()
    pub fn kill(&mut self) {
        self.stop();
        let _ = self.child.start_kill();
    }

    /// Kill known stress test programs that CoreCycler may have spawned
    /// Names match what the CoreCycler script actually launches
    fn kill_stress_test_programs(&self) {
        let programs = [
            "prime95.exe",
            "prime95_dev.exe",
            "y-cruncher.exe",
            // y-cruncher new (v0.8+) binary names from script testModes
            "00-x86.exe",
            "04-P4P.exe",
            "05-A64.exe",
            "08-NHM.exe",
            "11-SNB.exe",
            "12-BD2.exe",
            "13-HSW.exe",
            "14-BDW.exe",
            "17-SKX.exe",
            "17-ZN1.exe",
            "18-CNL.exe",
            "19-ZN2.exe",
            "22-ZN4.exe",
            "24-ZN5.exe",
            // y-cruncher old (v0.7.10) binary names
            "11-BD1.exe",
            "20-ZN3.exe",
            // y-cruncher logging wrapper
            "WriteConsoleToWriteFileWrapper.exe",
            // Linpack - patched binary used by script
            "linpack_patched.exe",
            // Linpack - legacy names just in case
            "linpack_xeon64.exe",
            "linpack2021_xeon64.exe",
            "linpack2024_xeon64.exe",
            // AIDA64
            "aida_bench64.dll",
            "aida64.exe",
        ];
        for prog in &programs {
            let _ = std::process::Command::new("taskkill")
                .args(["/IM", prog, "/F"])
                .creation_flags(CREATE_NO_WINDOW)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
        }
    }

    /// Check if still running
    pub fn try_wait(&mut self) -> Option<i32> {
        match self.child.try_wait() {
            Ok(Some(status)) => Some(status.code().unwrap_or(-1)),
            _ => None,
        }
    }
}

impl Drop for RunnerHandle {
    fn drop(&mut self) {
        // Kill stress test programs first
        self.kill_stress_test_programs();
        // Then kill process tree
        self.kill();
        // Job Object with KILL_ON_JOB_CLOSE will also kill everything when _job drops
    }
}

/// Spawn the CoreCycler PowerShell script and return a handle + output receiver.
/// Uses a Job Object to ensure all child processes are killed when the handle is dropped.
/// Monitors both stdout and the log file for output.
pub fn spawn_corecycler(
    script_dir: PathBuf,
    config: &CoreCyclerConfig,
) -> Result<(RunnerHandle, mpsc::UnboundedReceiver<String>), String> {
    let script_path = script_dir.join("script-corecycler.ps1");
    if !script_path.exists() {
        return Err(format!(
            "script-corecycler.ps1 not found at {}",
            script_path.display()
        ));
    }

    let (tx, rx) = mpsc::unbounded_channel();
    let logs_dir = script_dir.join("logs");
    let log_monitor_hint = LogMonitorHint::from_config(config);
    let (log_path_tx, log_path_rx) = watch::channel(None::<PathBuf>);

    // Use -Command with *>&1 to capture Write-Host output (information stream 6)
    let command_str = format!(
        "& '{}' *>&1",
        script_path.to_string_lossy().replace('\'', "''")
    );

    let mut cmd = Command::new("powershell.exe");
    cmd.args([
        "-ExecutionPolicy",
        "Bypass",
        "-NoProfile",
        "-NoLogo",
        "-Command",
        &command_str,
    ])
    .current_dir(&script_dir)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .stdin(Stdio::null());

    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start PowerShell: {}", e))?;

    let pid = child.id().unwrap_or(0);

    // Create Job Object and assign the PowerShell process to it
    // This ensures ALL child processes (Prime95, y-cruncher, etc.) die when we drop the handle
    #[cfg(windows)]
    let job_obj = {
        let j = job::JobObject::new();
        if let Some(ref j) = j {
            if pid != 0 {
                j.assign_pid(pid);
            }
        }
        j
    };

    // Spawn stdout reader
    if let Some(stdout) = child.stdout.take() {
        let tx_out = tx.clone();
        let logs_dir = logs_dir.clone();
        let log_monitor_hint = log_monitor_hint.clone();
        let log_path_tx = log_path_tx.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Some(log_path) = log_monitor_hint.extract_path_from_output_line(&line, &logs_dir)
                {
                    let _ = log_path_tx.send(Some(log_path));
                }
                if tx_out.send(line).is_err() {
                    break;
                }
            }
        });
    }

    // Spawn stderr reader
    if let Some(stderr) = child.stderr.take() {
        let tx_err = tx.clone();
        tokio::spawn(async move {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if tx_err.send(format!("[STDERR] {}", line)).is_err() {
                    break;
                }
            }
        });
    }

    // Spawn log file monitor - polls the newest CoreCycler log file for new lines
    // This is the primary way we get output since Write-Host doesn't always pipe to stdout
    {
        let tx_log = tx.clone();
        let logs_dir = logs_dir.clone();
        let spawn_time = std::time::SystemTime::now();
        let log_monitor_hint = log_monitor_hint.clone();
        tokio::spawn(async move {
            monitor_log_file(logs_dir, tx_log, spawn_time, log_monitor_hint, log_path_rx).await;
        });
    }

    let handle = RunnerHandle {
        child,
        pid,
        #[cfg(windows)]
        _job: job_obj,
    };
    Ok((handle, rx))
}

/// Monitor the newest CoreCycler log file and send new lines through the channel
/// Only picks up log files that match the current launch configuration.
async fn monitor_log_file(
    logs_dir: PathBuf,
    tx: mpsc::UnboundedSender<String>,
    spawn_time: std::time::SystemTime,
    hint: LogMonitorHint,
    log_path_rx: watch::Receiver<Option<PathBuf>>,
) {
    use std::io::BufRead;

    let mut log_path_rx = log_path_rx;

    let log_path = loop {
        if let Some(path) = log_path_rx.borrow().clone() {
            if path.exists() {
                break path;
            }
        }

        if let Some(path) = find_matching_log_file(&logs_dir, &hint, spawn_time) {
            break path;
        }

        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        if tx.is_closed() {
            return;
        }
    };

    let _ = tx.send(format!("[LOG] Monitoring: {}", log_path.display()));

    // Open the file and seek to the beginning to capture everything
    let mut file = match std::fs::File::open(&log_path) {
        Ok(f) => f,
        Err(_) => return,
    };

    // Read from the start to get all existing content
    let mut reader = std::io::BufReader::new(&mut file);
    let mut line_buf = String::new();

    loop {
        line_buf.clear();
        match reader.read_line(&mut line_buf) {
            Ok(0) => {
                // No more data right now, wait and try again
                drop(reader);
                tokio::time::sleep(std::time::Duration::from_millis(300)).await;
                if tx.is_closed() {
                    return;
                }
                reader = std::io::BufReader::new(&mut file);
            }
            Ok(_) => {
                let trimmed = line_buf.trim_end().to_string();
                if !trimmed.is_empty() {
                    if tx.send(format!("[LOG] {}", trimmed)).is_err() {
                        return;
                    }
                }
            }
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                if tx.is_closed() {
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LogMonitorHint;
    use crate::config::{CoreCyclerConfig, LinpackVersion, Prime95Mode, StressTestProgram};

    #[test]
    fn extracts_matching_core_log_name_from_output_line() {
        let mut config = CoreCyclerConfig::default();
        config.logging.name = "StressTest".to_string();
        config.general.stress_test_program = StressTestProgram::Prime95;
        config.prime95.mode = Prime95Mode::AVX2;

        let hint = LogMonitorHint::from_config(&config);
        let logs_dir = std::path::Path::new(r"E:\logs");
        let line = " - CoreCycler: StressTest_2026-03-08_12-00-00_PRIME95_AVX2.log";

        assert_eq!(
            hint.extract_path_from_output_line(line, logs_dir),
            Some(logs_dir.join("StressTest_2026-03-08_12-00-00_PRIME95_AVX2.log"))
        );
    }

    #[test]
    fn linpack_newer_versions_expect_fastest_in_core_log_name() {
        let mut config = CoreCyclerConfig::default();
        config.general.stress_test_program = StressTestProgram::Linpack;
        config.linpack.version = LinpackVersion::V2024;

        let hint = LogMonitorHint::from_config(&config);

        assert!(hint.matches_file_name(
            "CoreCycler_2026-03-08_12-00-00_LINPACK_2024_FASTEST.log"
        ));
        assert!(!hint.matches_file_name(
            "CoreCycler_2026-03-08_12-00-00_LINPACK_2024_MEDIUM.log"
        ));
    }
}
