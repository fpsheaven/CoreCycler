use std::path::PathBuf;

use iced::widget::{button, column, container, horizontal_rule, row, text, Space};
use iced::{Element, Length, Subscription, Task};

use crate::config::*;
use crate::runner;
use crate::ui;
pub use crate::ui::monitor_view::CoreState;

use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct CoreStatus {
    pub id: u32,
    pub state: CoreState,
    pub error_count: u32,
    pub times_tested: u32,
}

pub struct App {
    script_dir: PathBuf,
    pub config: CoreCyclerConfig,
    pub config_section: usize,
    pub num_physical_cores: u32,
    pub preset_configs: Vec<String>,

    // Runner state
    pub is_running: bool,
    pub running_program: String,
    pub started_at: Option<std::time::Instant>,
    runner_handle: Option<runner::RunnerHandle>,
    output_rx: Option<mpsc::UnboundedReceiver<String>>,
    pub log_lines: Vec<String>,
    pub cores: Vec<CoreStatus>,
    pub current_iteration: u32,
    pub status_text: String,
    status_message: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    ConfigSectionChanged(usize),
    SaveConfig,
    LoadConfig,
    ResetConfig,
    RunProgram(String),
    StopTest,
    OutputReceived(Vec<String>),
    ProcessExited(i32),
    CopyLog,
    Tick,

    // General config messages
    GeneralUseConfigFile(String),
    GeneralStressTestProgram(String),
    GeneralRuntimePerCore(String),
    GeneralSuspendPeriodically(bool),
    GeneralCoreTestOrder(String),
    GeneralSkipCoreOnError(bool),
    GeneralStopOnError(bool),
    GeneralNumberOfThreads(String),
    GeneralAssignBothVCores(bool),
    GeneralMaxIterations(String),
    GeneralCoresToIgnore(String),
    ToggleCoreIgnore(u32, bool),
    GeneralRestartPerCore(bool),
    GeneralDelayBetweenCores(String),
    GeneralBeepOnError(bool),
    GeneralFlashOnError(bool),
    GeneralLookForWhea(bool),
    GeneralTreatWheaAsError(bool),

    // Prime95
    Prime95Mode(String),
    Prime95FFTSize(String),

    // yCruncher
    YCruncherMode(String),
    YCruncherTests(String),
    YCruncherToggleTest(String, bool),
    YCruncherTestDuration(String),
    YCruncherMemory(String),
    YCruncherLoggingWrapper(bool),

    // Aida64
    Aida64Mode(String),
    Aida64ToggleModeComponent(String, bool),
    Aida64UseAVX(bool),
    Aida64MaxMemory(String),

    // Linpack
    LinpackVersion(String),
    LinpackModeChanged(String),
    LinpackMemory(String),

    // Auto mode
    AutoEnableAdjustment(bool),
    AutoStartValues(String),
    AutoMaxValue(String),
    AutoIncrementBy(String),
    AutoVoltageOnlyTested(bool),
    AutoRepeatCoreOnError(bool),
    AutoEnableResume(bool),
    AutoWaitBeforeResume(String),
    AutoCreateRestore(bool),
    AutoAskRestore(bool),

    // Logging
    LogName(String),
    LogLevel(String),
    LogUseEventLog(bool),
    LogFlushCache(bool),

    // Update
    UpdateEnableCheck(bool),
    UpdateFrequency(String),

    // Prime95 Custom
    P95CustomAVX(bool),
    P95CustomAVX2(bool),
    P95CustomFMA3(bool),
    P95CustomAVX512(bool),
    P95CustomMinFFT(String),
    P95CustomMaxFFT(String),
    P95CustomMem(String),
    P95CustomTime(String),

    // Debug
    DebugDisableCpuCheck(bool),
    DebugUsePerfCounters(bool),
    DebugEnableFreqCheck(bool),
    DebugTickInterval(String),
    DebugDelayFirstCheck(String),
    DebugPriority(String),
    DebugShowWindow(bool),
    DebugSuspensionTime(String),
    DebugSuspensionMode(String),
}

impl App {
    pub fn new(script_dir: PathBuf) -> (Self, Task<Message>) {
        // Detect physical cores
        let sys = sysinfo::System::new_all();
        let num_cores = sys.physical_core_count().unwrap_or(8) as u32;

        let cores: Vec<CoreStatus> = (0..num_cores)
            .map(|i| CoreStatus {
                id: i,
                state: CoreState::Idle,
                error_count: 0,
                times_tested: 0,
            })
            .collect();

        // Try to load existing config
        let config_path = script_dir.join("config.ini");
        let default_path = script_dir.join("configs").join("default.config.ini");

        let config = if config_path.exists() {
            load_config(&config_path).unwrap_or_default()
        } else if default_path.exists() {
            load_config(&default_path).unwrap_or_default()
        } else {
            CoreCyclerConfig::default()
        };

        let preset_configs = list_preset_configs(&script_dir);

        let app = Self {
            script_dir,
            config,
            config_section: 0,
            num_physical_cores: num_cores,
            preset_configs,
            is_running: false,
            running_program: String::new(),
            started_at: None,
            runner_handle: None,
            output_rx: None,
            log_lines: Vec::new(),
            cores,
            current_iteration: 0,
            status_text: "Ready".to_string(),
            status_message: None,
        };

        (app, Task::none())
    }

    pub fn subscription(&self) -> Subscription<Message> {
        if self.is_running {
            iced::time::every(std::time::Duration::from_millis(100)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ConfigSectionChanged(idx) => {
                self.config_section = idx;
            }
            Message::SaveConfig => {
                let path = self.script_dir.join("config.ini");
                match save_config(&path, &self.config) {
                    Ok(_) => self.status_message = Some("Config saved to config.ini".to_string()),
                    Err(e) => self.status_message = Some(format!("Save failed: {}", e)),
                }
            }
            Message::LoadConfig => {
                let config_path = self.script_dir.join("config.ini");
                let path = if config_path.exists() {
                    config_path
                } else {
                    self.script_dir.join("configs").join("default.config.ini")
                };
                match load_config(&path) {
                    Ok(cfg) => {
                        self.config = cfg;
                        self.status_message = Some(format!("Loaded {}", path.display()));
                    }
                    Err(e) => self.status_message = Some(format!("Load failed: {}", e)),
                }
            }
            Message::ResetConfig => {
                self.config = CoreCyclerConfig::default();
                self.status_message = Some("Config reset to defaults".to_string());
            }
            Message::RunProgram(program) => {
                if !self.is_running {
                    // Set the stress test program
                    self.config.general.stress_test_program = StressTestProgram::from_str(&program);
                    // Clear useConfigFile so the script uses our config.ini directly
                    // (otherwise the script loads the referenced file and overrides our settings)
                    self.config.general.use_config_file = String::new();

                    // Normalize yCruncher tests for old vs new to prevent script abort
                    if program == "YCRUNCHER_OLD" {
                        let old_tags: Vec<&str> = YCruncherConfig::OLD_TESTS.iter().map(|(t, _)| *t).collect();
                        let current: Vec<String> = self.config.ycruncher.tests
                            .split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                        let valid: Vec<String> = current.into_iter().filter(|t| old_tags.contains(&t.as_str())).collect();
                        if valid.is_empty() {
                            self.config.ycruncher.tests = YCruncherConfig::OLD_DEFAULT_TESTS.to_string();
                        } else {
                            self.config.ycruncher.tests = valid.join(", ");
                        }
                        // Normalize mode: BD2 -> BD1, ZN5 -> ZN4 for old version
                        let mode = &self.config.ycruncher.mode;
                        if !YCruncherMode::OLD.contains(mode) {
                            self.config.ycruncher.mode = YCruncherMode::X86;
                        }
                    } else if program == "YCRUNCHER" {
                        let new_tags: Vec<&str> = YCruncherConfig::NEW_TESTS.iter().map(|(t, _)| *t).collect();
                        let current: Vec<String> = self.config.ycruncher.tests
                            .split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                        let valid: Vec<String> = current.into_iter().filter(|t| new_tags.contains(&t.as_str())).collect();
                        if valid.is_empty() {
                            self.config.ycruncher.tests = YCruncherConfig::NEW_DEFAULT_TESTS.to_string();
                        } else {
                            self.config.ycruncher.tests = valid.join(", ");
                        }
                        // Normalize mode: BD1 -> X86, ZN3 -> ZN2 for new version
                        let mode = &self.config.ycruncher.mode;
                        if !YCruncherMode::NEW.contains(mode) {
                            self.config.ycruncher.mode = YCruncherMode::X86;
                        }
                    }

                    // Save config first
                    let path = self.script_dir.join("config.ini");
                    if let Err(e) = save_config(&path, &self.config) {
                        self.status_message = Some(format!("Failed to save config: {}", e));
                        return Task::none();
                    }

                    match runner::spawn_corecycler(self.script_dir.clone(), &self.config) {
                        Ok((handle, rx)) => {
                            self.is_running = true;
                            self.running_program = program.clone();
                            self.started_at = Some(std::time::Instant::now());
                            self.runner_handle = Some(handle);
                            self.output_rx = Some(rx);
                            self.log_lines.clear();
                            self.log_lines.push(format!("[GUI] Starting {}", program));
                            self.current_iteration = 0;
                            self.status_text = format!("Starting {}...", program);
                            self.status_message = None;

                            // Reset core states
                            for core in &mut self.cores {
                                core.state = CoreState::Idle;
                                core.error_count = 0;
                                core.times_tested = 0;
                            }
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Failed to start: {}", e));
                        }
                    }
                }
            }
            Message::StopTest => {
                if let Some(ref mut handle) = self.runner_handle {
                    handle.stop();
                    self.status_text = "Stopping...".to_string();
                }
            }
            Message::Tick => {
                // Drain output from the receiver
                if let Some(ref mut rx) = self.output_rx {
                    let mut new_lines = Vec::new();
                    while let Ok(line) = rx.try_recv() {
                        new_lines.push(line);
                    }
                    for line in &new_lines {
                        self.parse_output_line(line);
                    }
                    self.log_lines.extend(new_lines);

                    // Keep log lines bounded
                    if self.log_lines.len() > 5000 {
                        self.log_lines.drain(0..self.log_lines.len() - 3000);
                    }
                }

                // Check if process exited
                if let Some(ref mut handle) = self.runner_handle {
                    if let Some(code) = handle.try_wait() {
                        let elapsed = self.elapsed_str();
                        self.is_running = false;
                        self.status_text = format!("Finished {} after {} (exit code: {})", self.running_program, elapsed, code);
                        self.runner_handle = None;
                        self.output_rx = None;
                        self.started_at = None;
                    }
                }
            }
            Message::CopyLog => {
                let log_text = self.log_lines.join("\n");
                return iced::clipboard::write(log_text);
            }
            Message::OutputReceived(_) | Message::ProcessExited(_) => {}

            // General config updates
            Message::GeneralUseConfigFile(v) => self.config.general.use_config_file = v,
            Message::GeneralStressTestProgram(v) => {
                self.config.general.stress_test_program = StressTestProgram::from_str(&v);
            }
            Message::GeneralRuntimePerCore(v) => {
                let digits: String = v.chars().filter(|c| c.is_ascii_digit()).collect();
                self.config.general.runtime_per_core = if digits.is_empty() {
                    String::new()
                } else {
                    format!("{}m", digits)
                };
            }
            Message::GeneralSuspendPeriodically(v) => self.config.general.suspend_periodically = v,
            Message::GeneralCoreTestOrder(v) => {
                self.config.general.core_test_order = CoreTestOrder::from_str(&v);
            }
            Message::GeneralSkipCoreOnError(v) => self.config.general.skip_core_on_error = v,
            Message::GeneralStopOnError(v) => self.config.general.stop_on_error = v,
            Message::GeneralNumberOfThreads(v) => {
                self.config.general.number_of_threads = v.parse().unwrap_or(1);
            }
            Message::GeneralAssignBothVCores(v) => self.config.general.assign_both_virtual_cores = v,
            Message::GeneralMaxIterations(v) => self.config.general.max_iterations = v,
            Message::GeneralCoresToIgnore(v) => self.config.general.cores_to_ignore = v,
            Message::ToggleCoreIgnore(core_id, checked) => {
                let mut cores: Vec<u32> = self.config.general.cores_to_ignore
                    .split(',')
                    .filter_map(|s| s.trim().parse::<u32>().ok())
                    .collect();
                if checked {
                    if !cores.contains(&core_id) {
                        cores.push(core_id);
                        cores.sort();
                    }
                } else {
                    cores.retain(|&c| c != core_id);
                }
                self.config.general.cores_to_ignore = cores
                    .iter()
                    .map(|c| c.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
            }
            Message::GeneralRestartPerCore(v) => {
                self.config.general.restart_test_program_for_each_core = v;
            }
            Message::GeneralDelayBetweenCores(v) => self.config.general.delay_between_cores = v,
            Message::GeneralBeepOnError(v) => self.config.general.beep_on_error = v,
            Message::GeneralFlashOnError(v) => self.config.general.flash_on_error = v,
            Message::GeneralLookForWhea(v) => self.config.general.look_for_whea_errors = v,
            Message::GeneralTreatWheaAsError(v) => {
                self.config.general.treat_whea_warning_as_error = v;
            }

            // Prime95
            Message::Prime95Mode(v) => {
                self.config.prime95.mode = Prime95Mode::from_str(&v);
            }
            Message::Prime95FFTSize(v) => {
                self.config.prime95.fft_size = FFTSize::from_str(&v);
            }

            // yCruncher
            Message::YCruncherMode(v) => {
                // Parse from label
                self.config.ycruncher.mode = YCruncherMode::from_str(
                    v.split(" (").next().unwrap_or(&v),
                );
            }
            Message::YCruncherTests(v) => self.config.ycruncher.tests = v,
            Message::YCruncherToggleTest(test_name, enabled) => {
                let mut tests: Vec<String> = self.config.ycruncher.tests
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                if enabled {
                    if !tests.iter().any(|t| t == &test_name) {
                        tests.push(test_name);
                    }
                } else {
                    tests.retain(|t| t != &test_name);
                }
                self.config.ycruncher.tests = tests.join(", ");
            }
            Message::YCruncherTestDuration(v) => self.config.ycruncher.test_duration = v,
            Message::YCruncherMemory(v) => self.config.ycruncher.memory = v,
            Message::YCruncherLoggingWrapper(v) => self.config.ycruncher.enable_logging_wrapper = v,

            // Aida64
            Message::Aida64Mode(v) => self.config.aida64.mode = v,
            Message::Aida64ToggleModeComponent(component, enabled) => {
                let mut parts: Vec<String> = self.config.aida64.mode
                    .split(',')
                    .map(|s| s.trim().to_uppercase())
                    .filter(|s| !s.is_empty())
                    .collect();
                let comp = component.to_uppercase();
                if enabled {
                    if !parts.contains(&comp) {
                        parts.push(comp);
                    }
                } else {
                    parts.retain(|p| p != &comp);
                }
                // Maintain canonical order
                let order = ["CACHE", "CPU", "FPU", "RAM"];
                parts.sort_by_key(|p| order.iter().position(|o| *o == p.as_str()).unwrap_or(99));
                self.config.aida64.mode = if parts.is_empty() {
                    "CACHE".to_string()
                } else {
                    parts.join(",")
                };
            }
            Message::Aida64UseAVX(v) => self.config.aida64.use_avx = v,
            Message::Aida64MaxMemory(v) => self.config.aida64.max_memory = v,

            // Linpack
            Message::LinpackVersion(v) => {
                let ver = LinpackVersion::from_str(&v);
                // 2021 and 2024 always use FASTEST (AVX2) per the script
                if matches!(ver, LinpackVersion::V2021 | LinpackVersion::V2024) {
                    self.config.linpack.mode = LinpackMode::Fastest;
                }
                self.config.linpack.version = ver;
            }
            Message::LinpackModeChanged(v) => {
                self.config.linpack.mode = LinpackMode::from_str(&v);
            }
            Message::LinpackMemory(v) => self.config.linpack.memory = v,

            // Auto mode
            Message::AutoEnableAdjustment(v) => {
                self.config.automatic_test_mode.enable_automatic_adjustment = v;
            }
            Message::AutoStartValues(v) => self.config.automatic_test_mode.start_values = v,
            Message::AutoMaxValue(v) => self.config.automatic_test_mode.max_value = v,
            Message::AutoIncrementBy(v) => self.config.automatic_test_mode.increment_by = v,
            Message::AutoVoltageOnlyTested(v) => {
                self.config.automatic_test_mode.set_voltage_only_for_tested_core = v;
            }
            Message::AutoRepeatCoreOnError(v) => {
                self.config.automatic_test_mode.repeat_core_on_error = v;
            }
            Message::AutoEnableResume(v) => {
                self.config.automatic_test_mode.enable_resume_after_unexpected_exit = v;
            }
            Message::AutoWaitBeforeResume(v) => {
                self.config.automatic_test_mode.wait_before_automatic_resume = v;
            }
            Message::AutoCreateRestore(v) => {
                self.config.automatic_test_mode.create_system_restore_point = v;
            }
            Message::AutoAskRestore(v) => {
                self.config.automatic_test_mode.ask_for_system_restore_point_creation = v;
            }

            // Logging
            Message::LogName(v) => self.config.logging.name = v,
            Message::LogLevel(v) => self.config.logging.log_level = v,
            Message::LogUseEventLog(v) => self.config.logging.use_windows_event_log = v,
            Message::LogFlushCache(v) => self.config.logging.flush_disk_write_cache = v,

            // Update
            Message::UpdateEnableCheck(v) => self.config.update.enable_update_check = v,
            Message::UpdateFrequency(v) => self.config.update.update_check_frequency = v,

            // Prime95 Custom
            Message::P95CustomAVX(v) => self.config.prime95_custom.cpu_supports_avx = v,
            Message::P95CustomAVX2(v) => self.config.prime95_custom.cpu_supports_avx2 = v,
            Message::P95CustomFMA3(v) => self.config.prime95_custom.cpu_supports_fma3 = v,
            Message::P95CustomAVX512(v) => self.config.prime95_custom.cpu_supports_avx512 = v,
            Message::P95CustomMinFFT(v) => self.config.prime95_custom.min_torture_fft = v,
            Message::P95CustomMaxFFT(v) => self.config.prime95_custom.max_torture_fft = v,
            Message::P95CustomMem(v) => self.config.prime95_custom.torture_mem = v,
            Message::P95CustomTime(v) => self.config.prime95_custom.torture_time = v,

            // Debug
            Message::DebugDisableCpuCheck(v) => {
                self.config.debug.disable_cpu_utilization_check = v;
            }
            Message::DebugUsePerfCounters(v) => self.config.debug.use_windows_perf_counters = v,
            Message::DebugEnableFreqCheck(v) => {
                self.config.debug.enable_cpu_frequency_check = v;
            }
            Message::DebugTickInterval(v) => self.config.debug.tick_interval = v,
            Message::DebugDelayFirstCheck(v) => self.config.debug.delay_first_error_check = v,
            Message::DebugPriority(v) => {
                self.config.debug.stress_test_program_priority = ProcessPriority::from_str(&v);
            }
            Message::DebugShowWindow(v) => {
                self.config.debug.stress_test_program_window_to_foreground = v;
            }
            Message::DebugSuspensionTime(v) => self.config.debug.suspension_time = v,
            Message::DebugSuspensionMode(v) => {
                self.config.debug.mode_to_use_for_suspension = SuspensionMode::from_str(&v);
            }
        }
        Task::none()
    }

    pub fn view(&self) -> Element<'_, Message> {
        let status_bar: Element<Message> = if let Some(ref msg) = self.status_message {
            container(
                text(msg.as_str()).size(13).color(crate::ui::style::TEXT_SECONDARY),
            )
            .padding([6, 16])
            .width(Length::Fill)
            .style(crate::ui::style::surface_style)
            .into()
        } else {
            Space::with_height(0).into()
        };

        let header = container(
            row![
                column![
                    text("CORECYCLER").size(22).color(crate::ui::style::TEXT_PRIMARY),
                    text("A vibecoded GUI wrapper for sp00n's CoreCycler, by FPSHEAVEN").size(13).color(iced::Color::WHITE),
                ],
                Space::with_width(Length::Fill),
                if self.is_running {
                    row![
                        text(format!("{} | {}", self.running_program, self.elapsed_display()))
                            .size(15)
                            .color(crate::ui::style::SUCCESS),
                        Space::with_width(20),
                        button(text("STOP").size(14).color(crate::ui::style::BG_DARK))
                            .on_press(Message::StopTest)
                            .style(iced::widget::button::danger)
                            .padding([8, 24]),
                    ]
                    .align_y(iced::Alignment::Center)
                } else if !self.status_text.is_empty() && self.status_text != "Ready" {
                    row![
                        text(&self.status_text).size(15).color(crate::ui::style::TEXT_SECONDARY),
                    ]
                    .align_y(iced::Alignment::Center)
                } else {
                    row![
                        text("Ready").size(15).color(crate::ui::style::TEXT_MUTED),
                    ]
                    .align_y(iced::Alignment::Center)
                },
            ]
            .align_y(iced::Alignment::Center),
        )
        .padding([16, 24])
        .width(Length::Fill)
        .style(crate::ui::style::surface_style);

        let divider = container(Space::with_height(1))
            .width(Length::Fill)
            .style(|_: &iced::Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(crate::ui::style::BORDER)),
                ..Default::default()
            });

        // Config section on left, monitor on right
        let config_panel = ui::config_view(self);
        let monitor_panel = ui::monitor_view(self);

        let main_content = row![
            container(config_panel).width(Length::FillPortion(3)),
            container(Space::with_width(1))
                .height(Length::Fill)
                .style(|_: &iced::Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(crate::ui::style::BORDER)),
                    ..Default::default()
                }),
            container(monitor_panel).width(Length::FillPortion(2)),
        ];

        column![
            header,
            divider,
            container(main_content).height(Length::Fill).width(Length::Fill),
            status_bar,
        ]
        .into()
    }

    fn parse_output_line(&mut self, line: &str) {
        // Strip [LOG] prefix if present (from log file monitor)
        let line = line.strip_prefix("[LOG] ").unwrap_or(line);

        // Actual CoreCycler output patterns (from script-corecycler.ps1):
        // "HH:mm:ss - Iteration N"
        // "HH:mm:ss - Set to Core N (CPU X)"
        // "HH:mm:ss - Core N (CPU X) has previously thrown an error, skipping"
        // "Progress X/Y | Iteration N/M | Runtime ..."
        // "has thrown an error"
        // "Test completed in ..."
        // "WHEA"

        // Detect iteration: "- Iteration N" or "Iteration N/"
        if line.contains("Iteration") {
            if let Some(num) = extract_number_after(line, "Iteration ") {
                self.current_iteration = num;
                // Only update status if it's the main iteration line (not progress)
                if line.contains("- Iteration") && !line.contains("Progress") {
                    self.status_text = format!("Iteration {}", num);
                }
            }
        }

        // Detect "Set to Core N" - this is THE primary indicator of which core is being tested
        // Pattern: "HH:mm:ss - Set to Core N (CPU X)"
        if line.contains("Set to Core") {
            if let Some(core_num) = extract_number_after(line, "Set to Core ") {
                // Mark previous testing core as passed
                for core in &mut self.cores {
                    if core.state == CoreState::Testing {
                        core.state = CoreState::Passed;
                        core.times_tested += 1;
                    }
                }
                // Set new core to testing
                if let Some(core) = self.cores.iter_mut().find(|c| c.id == core_num) {
                    core.state = CoreState::Testing;
                    self.status_text = format!("Testing Core {}", core_num);
                }
            }
        }

        // Detect progress line: "Progress X/Y | Iteration N/M | Runtime ..."
        if line.contains("Progress") && line.contains("Runtime") {
            self.status_text = line
                .trim()
                .trim_start_matches(|c: char| !c.is_ascii_alphabetic())
                .to_string();
        }

        // Detect "Running for X..."
        if line.contains("Running for") {
            let clean = line.trim().trim_start_matches(|c: char| c.is_whitespace());
            if !clean.is_empty() {
                // Keep existing status but note runtime
            }
        }

        // Detect test completed - mark current testing core as passed
        if line.contains("Test completed in") {
            for core in &mut self.cores {
                if core.state == CoreState::Testing {
                    core.state = CoreState::Passed;
                    core.times_tested += 1;
                }
            }
        }

        // Detect errors: "has thrown an error" is the definitive CoreCycler error pattern
        // Avoid false positives from debug lines like "Checking for stress test errors"
        if line.contains("has thrown an error") {
            if let Some(core_num) = extract_core_number(line) {
                if let Some(core) = self.cores.iter_mut().find(|c| c.id == core_num) {
                    core.state = CoreState::Error;
                    core.error_count += 1;
                }
            }
        }

        // Detect WHEA errors - only real ones, not debug lines like
        // "Looking for new WHEA errors" or "No new WHEA error"
        if line.contains("WHEA error while") || line.contains("WHEA errors while") {
            if let Some(core_num) = extract_core_number(line) {
                if let Some(core) = self.cores.iter_mut().find(|c| c.id == core_num) {
                    core.error_count += 1;
                    core.state = CoreState::Error;
                }
            }
        }

        // Detect skipped cores: "has previously thrown an error, skipping"
        if line.contains("skipping") {
            if let Some(core_num) = extract_core_number(line) {
                if let Some(core) = self.cores.iter_mut().find(|c| c.id == core_num) {
                    core.state = CoreState::Skipped;
                }
            }
        }

        // Detect "No core has thrown an error" in summary - mark all testing cores as passed
        if line.contains("No core has thrown an error") {
            for core in &mut self.cores {
                if core.state == CoreState::Testing {
                    core.state = CoreState::Passed;
                    core.times_tested += 1;
                }
            }
        }
    }

    fn elapsed_str(&self) -> String {
        if let Some(start) = self.started_at {
            let secs = start.elapsed().as_secs();
            let h = secs / 3600;
            let m = (secs % 3600) / 60;
            let s = secs % 60;
            if h > 0 {
                format!("{}h {:02}m {:02}s", h, m, s)
            } else if m > 0 {
                format!("{}m {:02}s", m, s)
            } else {
                format!("{}s", s)
            }
        } else {
            String::new()
        }
    }

    pub fn elapsed_display(&self) -> String {
        self.elapsed_str()
    }
}

fn extract_number_after(line: &str, keyword: &str) -> Option<u32> {
    if let Some(pos) = line.find(keyword) {
        let after = &line[pos + keyword.len()..];
        let num_str: String = after.chars().skip_while(|c| !c.is_ascii_digit()).take_while(|c| c.is_ascii_digit()).collect();
        num_str.parse().ok()
    } else {
        None
    }
}

fn extract_core_number(line: &str) -> Option<u32> {
    // Try "Core X" pattern
    if let Some(pos) = line.find("Core ") {
        let after = &line[pos + 5..];
        let num_str: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
        return num_str.parse().ok();
    }
    None
}
