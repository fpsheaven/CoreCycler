use iced::widget::{
    button, checkbox, column, container, pick_list, row, scrollable, text, text_input,
    Column, Space,
};
use iced::{Element, Length};

use crate::app::Message;
use crate::config::*;
use crate::ui::style;

/// Build the config editor view
pub fn config_view(app: &crate::app::App) -> Element<'_, Message> {
    let cfg = &app.config;
    let config_section = app.config_section;
    let is_running = app.is_running;

    let tabs_row1: &[(usize, &str)] = &[
        (0, "General"), (1, "Prime95"), (2, "yCruncher"), (3, "yCruncher Old"), (4, "Aida64"),
    ];
    let tabs_row2: &[(usize, &str)] = &[
        (5, "Linpack"), (6, "Auto Mode"), (7, "Logging"), (8, "Debug"),
    ];

    let row1 = iced::widget::Row::with_children(
        tabs_row1.iter().map(|(i, label)| tab_btn(label, *i, config_section)).collect::<Vec<_>>(),
    ).spacing(4).width(Length::Fill);
    let row2 = iced::widget::Row::with_children(
        tabs_row2.iter().map(|(i, label)| tab_btn(label, *i, config_section)).collect::<Vec<_>>(),
    ).spacing(4).width(Length::Fill);

    let section_buttons = column![row1, row2].spacing(4);

    let section_content: Element<'_, Message> = match config_section {
        0 => general_section(&cfg.general, app.num_physical_cores, &app.preset_configs),
        1 => prime95_section(&cfg.prime95, &cfg.prime95_custom, is_running),
        2 => ycruncher_section(&cfg.ycruncher, is_running, false),
        3 => ycruncher_section(&cfg.ycruncher, is_running, true),
        4 => aida64_section(&cfg.aida64, is_running),
        5 => linpack_section(&cfg.linpack, is_running),
        6 => auto_mode_section(&cfg.automatic_test_mode),
        7 => logging_section(&cfg.logging, &cfg.update),
        8 => debug_section(&cfg.debug),
        _ => text("Unknown section").into(),
    };

    let save_row = row![
        button(text("SAVE").size(14).color(style::BG_DARK))
            .on_press(Message::SaveConfig)
            .style(iced::widget::button::primary)
            .padding([10, 28]),
        Space::with_width(10),
        button(text("RELOAD").size(14))
            .on_press(Message::LoadConfig)
            .style(iced::widget::button::secondary)
            .padding([10, 28]),
        Space::with_width(10),
        button(text("RESET").size(14))
            .on_press(Message::ResetConfig)
            .style(iced::widget::button::secondary)
            .padding([10, 28]),
    ]
    .spacing(0);

    column![
        container(section_buttons).padding([8, 16]),
        divider(),
        scrollable(
            container(section_content)
                .padding([20, 28])
                .width(Length::Fill)
        )
        .height(Length::Fill),
        divider(),
        container(save_row).padding([14, 28]),
    ]
    .spacing(0)
    .into()
}

fn run_button<'a>(program: &str, is_running: bool) -> Element<'a, Message> {
    if is_running {
        button(text("RUNNING...").size(14).color(style::TEXT_MUTED))
            .style(iced::widget::button::secondary)
            .padding([12, 36])
            .into()
    } else {
        let prog = program.to_string();
        button(text("RUN").size(15).color(style::BG_DARK))
            .on_press(Message::RunProgram(prog))
            .style(iced::widget::button::success)
            .padding([12, 36])
            .into()
    }
}

fn divider<'a>() -> Element<'a, Message> {
    container(Space::with_height(1))
        .width(Length::Fill)
        .style(|_: &iced::Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(style::BORDER)),
            ..Default::default()
        })
        .into()
}

fn tab_btn(label: &str, idx: usize, current: usize) -> Element<'_, Message> {
    let is_active = idx == current;
    let btn_style = if is_active { style::tab_active } else { style::tab_inactive };
    let txt_color = if is_active { iced::Color::WHITE } else { style::TEXT_SECONDARY };
    button(text(label).size(14).color(txt_color).center())
        .on_press(Message::ConfigSectionChanged(idx))
        .style(btn_style)
        .padding([9, 16])
        .width(Length::Fill)
        .into()
}

fn section_header(title: &str) -> Element<'_, Message> {
    text(title).size(14).color(style::TEXT_MUTED).into()
}

fn field_row<'a>(label: &'a str, widget: Element<'a, Message>) -> Element<'a, Message> {
    row![
        container(text(label).size(15).color(style::TEXT_SECONDARY)).width(300),
        container(widget).width(Length::Fill),
    ]
    .spacing(20)
    .align_y(iced::Alignment::Center)
    .into()
}

fn bool_field(
    label: &str,
    value: bool,
    msg: fn(bool) -> Message,
) -> Element<'_, Message> {
    row![
        container(text(label).size(15).color(style::TEXT_SECONDARY)).width(300),
        checkbox("", value).on_toggle(msg).size(20),
    ]
    .spacing(20)
    .align_y(iced::Alignment::Center)
    .into()
}

fn strvec(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn pick_or_first(value: &str, options: &[String]) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return options.first().cloned();
    }
    // Always return the actual value - even if it's not in our preset list.
    // The dropdown function will inject it into the options if needed.
    Some(trimmed.to_string())
}

fn dropdown<'a>(
    label: &'a str,
    mut options: Vec<String>,
    selected: Option<String>,
    on_select: impl Fn(String) -> Message + 'a,
    width: f32,
) -> Element<'a, Message> {
    // If the selected value isn't in the options list, prepend it so it shows in the dropdown
    if let Some(ref sel) = selected {
        if !options.iter().any(|o| o == sel) {
            options.insert(0, sel.clone());
        }
    }
    field_row(
        label,
        pick_list(options, selected, on_select)
            .text_size(15)
            .padding(10)
            .width(width)
            .into(),
    )
}

fn hint_text(msg: &str) -> Element<'_, Message> {
    text(msg).size(13).color(style::TEXT_MUTED).into()
}

fn text_field<'a>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a,
    width: f32,
) -> Element<'a, Message> {
    field_row(
        label,
        text_input(placeholder, value)
            .on_input(on_change)
            .size(15)
            .padding(10)
            .width(width)
            .into(),
    )
}

fn general_section<'a>(cfg: &'a GeneralConfig, num_physical_cores: u32, preset_configs: &'a [String]) -> Element<'a, Message> {
    let threads_options: Vec<String> = vec!["1".to_string(), "2".to_string()];
    let selected_threads = Some(if cfg.number_of_threads == 2 { "2".to_string() } else { "1".to_string() });


    // Stress test program dropdown
    let programs: Vec<String> = StressTestProgram::ALL.iter().map(|p| p.as_str().to_string()).collect();
    let selected_program = Some(cfg.stress_test_program.as_str().to_string());

    // useConfigFile dropdown - list preset configs
    let mut config_file_options: Vec<String> = vec!["(disabled)".to_string()];
    for name in preset_configs {
        config_file_options.push(format!("configs\\{}", name));
    }
    let selected_config_file = if cfg.use_config_file.is_empty() {
        Some("(disabled)".to_string())
    } else {
        Some(cfg.use_config_file.clone())
    };

    // Parse currently ignored cores from comma-separated string
    let ignored_cores: Vec<u32> = cfg.cores_to_ignore
        .split(',')
        .filter_map(|s| s.trim().parse::<u32>().ok())
        .collect();

    // Build core ignore checkboxes using physical core indices
    let core_ignore_checkboxes = iced::widget::Row::with_children(
        (0..num_physical_cores).map(|i| {
            let is_ignored = ignored_cores.contains(&i);
            let core_id = i;
            container(
                checkbox(format!("{}", i), is_ignored)
                    .on_toggle(move |checked| {
                        Message::ToggleCoreIgnore(core_id, checked)
                    })
                    .text_size(13)
                    .size(18)
                    .spacing(4),
            )
            .padding([0, 6])
            .into()
        }).collect::<Vec<Element<'_, Message>>>(),
    )
    .spacing(2);

    column![
        section_header("GENERAL"),
        Space::with_height(12),
        dropdown("Stress Test Program", programs, selected_program, |v| Message::GeneralStressTestProgram(v), 260.0),
        hint_text("Clicking RUN on a stress test tab overrides this. This shows the saved value."),
        Space::with_height(4),
        dropdown("Use Config File", config_file_options, selected_config_file,
            |v| Message::GeneralUseConfigFile(if v == "(disabled)" { String::new() } else { v }), 360.0),
        hint_text("When set, all settings below are overridden by that file. Cleared on RUN."),
        Space::with_height(8),
        text_field("Runtime Per Core (minutes)", "6", cfg.runtime_per_core.trim_end_matches('m'), |v| Message::GeneralRuntimePerCore(v), 260.0),
        hint_text("Enter minutes only (e.g. 6 = 6 minutes). Always saved as minutes."),
        text_field("Core Test Order", "Default", cfg.core_test_order.as_str(), |v| Message::GeneralCoreTestOrder(v), 260.0),
        hint_text("Default, Alternate, CorePairs, Random, Sequential, or comma-separated list (e.g. 5,4,0,7)."),
        dropdown("Threads", threads_options, selected_threads, |v| Message::GeneralNumberOfThreads(v), 260.0),
        text_field("Max Iterations", "10000", &cfg.max_iterations, |v| Message::GeneralMaxIterations(v), 260.0),
        text_field("Delay Between Cores (s)", "15", &cfg.delay_between_cores, |v| Message::GeneralDelayBetweenCores(v), 260.0),
        Space::with_height(8),
        section_header("CORES TO IGNORE"),
        hint_text("Check cores to skip during testing (physical core indices)."),
        Space::with_height(4),
        core_ignore_checkboxes,
        Space::with_height(8),
        section_header("BEHAVIOR"),
        Space::with_height(8),
        bool_field("Suspend Periodically", cfg.suspend_periodically, Message::GeneralSuspendPeriodically),
        bool_field("Skip Core on Error", cfg.skip_core_on_error, Message::GeneralSkipCoreOnError),
        bool_field("Stop on Error", cfg.stop_on_error, Message::GeneralStopOnError),
        bool_field("Assign Both Virtual Cores", cfg.assign_both_virtual_cores, Message::GeneralAssignBothVCores),
        bool_field("Restart Test Per Core", cfg.restart_test_program_for_each_core, Message::GeneralRestartPerCore),
        Space::with_height(8),
        section_header("NOTIFICATIONS"),
        Space::with_height(8),
        bool_field("Beep on Error", cfg.beep_on_error, Message::GeneralBeepOnError),
        bool_field("Flash on Error", cfg.flash_on_error, Message::GeneralFlashOnError),
        bool_field("Look for WHEA Errors", cfg.look_for_whea_errors, Message::GeneralLookForWhea),
        bool_field("Treat WHEA Warning as Error", cfg.treat_whea_warning_as_error, Message::GeneralTreatWheaAsError),
    ]
    .spacing(10)
    .into()
}

fn prime95_section<'a>(cfg: &'a Prime95Config, custom: &'a Prime95CustomConfig, is_running: bool) -> Element<'a, Message> {
    let modes: Vec<String> = Prime95Mode::ALL.iter().map(|m| m.as_str().to_string()).collect();
    let selected_mode = Some(cfg.mode.as_str().to_string());
    column![
        row![
            section_header("PRIME95"),
            Space::with_width(Length::Fill),
            run_button("PRIME95", is_running),
        ].align_y(iced::Alignment::Center),
        Space::with_height(12),
        dropdown("Mode", modes, selected_mode, |v| Message::Prime95Mode(v), 260.0),
        text_field("FFT Size", "Huge", cfg.fft_size.as_str(), |v| Message::Prime95FFTSize(v), 260.0),
        hint_text("Presets: Smallest, Small, Large, Huge, All, Moderate, Heavy, HeavyShort. Or custom range e.g. 36-1344"),
        Space::with_height(8),
        hint_text("SSE = lightest load / highest boost. AVX2 = heaviest. Huge tests memory controller."),
        Space::with_height(16),
        section_header("CUSTOM MODE SETTINGS"),
        hint_text("These only apply when Mode is set to CUSTOM."),
        Space::with_height(8),
        bool_field("CPU Supports AVX", custom.cpu_supports_avx, Message::P95CustomAVX),
        bool_field("CPU Supports AVX2", custom.cpu_supports_avx2, Message::P95CustomAVX2),
        bool_field("CPU Supports FMA3", custom.cpu_supports_fma3, Message::P95CustomFMA3),
        bool_field("CPU Supports AVX512", custom.cpu_supports_avx512, Message::P95CustomAVX512),
        text_field("Min FFT Size (K)", "4", &custom.min_torture_fft, |v| Message::P95CustomMinFFT(v), 260.0),
        text_field("Max FFT Size (K)", "8192", &custom.max_torture_fft, |v| Message::P95CustomMaxFFT(v), 260.0),
        text_field("Memory (MB, 0=In-Place)", "0", &custom.torture_mem, |v| Message::P95CustomMem(v), 260.0),
        text_field("Time Per FFT (min)", "1", &custom.torture_time, |v| Message::P95CustomTime(v), 260.0),
    ]
    .spacing(10)
    .into()
}

fn ycruncher_section(cfg: &YCruncherConfig, is_running: bool, is_old: bool) -> Element<'_, Message> {
    let mode_list = if is_old { YCruncherMode::OLD } else { YCruncherMode::NEW };
    let modes: Vec<String> = mode_list.iter().map(|m| m.label().to_string()).collect();
    let selected = Some(cfg.mode.label().to_string());

    let all_tests = if is_old { YCruncherConfig::OLD_TESTS } else { YCruncherConfig::NEW_TESTS };

    let active_tests: Vec<String> = cfg.tests
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let test_checkboxes = Column::with_children(
        all_tests.iter().map(|(tag, desc)| {
            let is_active = active_tests.iter().any(|t| t == *tag);
            let tag_owned = tag.to_string();
            row![
                checkbox(
                    format!("{: <7} {}", tag, desc),
                    is_active,
                )
                .on_toggle(move |v| Message::YCruncherToggleTest(tag_owned.clone(), v))
                .text_size(14)
                .size(20)
                .spacing(10),
            ]
            .into()
        }).collect::<Vec<Element<'_, Message>>>(),
    )
    .spacing(4);

    let program_name = if is_old { "YCRUNCHER_OLD" } else { "YCRUNCHER" };
    let header_text = if is_old { "Y-CRUNCHER OLD (v0.7.10)" } else { "Y-CRUNCHER" };

    column![
        row![
            section_header(header_text),
            Space::with_width(Length::Fill),
            run_button(program_name, is_running),
        ].align_y(iced::Alignment::Center),
        Space::with_height(12),
        dropdown("Mode", modes, selected, |v| Message::YCruncherMode(v), 360.0),
        Space::with_height(8),
        section_header("TESTS"),
        Space::with_height(4),
        test_checkboxes,
        Space::with_height(8),
        text_field("Test Duration (s)", "60", &cfg.test_duration, |v| Message::YCruncherTestDuration(v), 260.0),
        text_field("Memory", "Default", &cfg.memory, |v| Message::YCruncherMemory(v), 260.0),
        hint_text("Memory: 'Default' or value like 64MB, 256MB, 1GB, etc."),
        bool_field("Enable Logging Wrapper", cfg.enable_logging_wrapper, Message::YCruncherLoggingWrapper),
        Space::with_height(8),
        if is_old {
            hint_text("Old y-cruncher v0.7.10. 11-BD1 = Bulldozer. 20-ZN3 = Zen 3.")
        } else {
            hint_text("auto = let y-cruncher pick the best mode for your CPU.")
        },
        if is_old {
            hint_text("Tests differ from new version. C17 is optional and slower.")
        } else {
            hint_text("00-x86 = lightest load. 19-ZN2 = good for Zen 2/3/4 stability testing.")
        },
        if is_old {
            Space::with_height(0).into()
        } else {
            hint_text("AVX512 modes (17-SKX, 18-CNL, 22-ZN4, 24-ZN5) only work on CPUs that support AVX512.")
        },
    ]
    .spacing(10)
    .into()
}

fn aida64_section(cfg: &Aida64Config, is_running: bool) -> Element<'_, Message> {
    let mode_upper = cfg.mode.to_uppercase();
    let has_cache = mode_upper.contains("CACHE");
    let has_cpu = mode_upper.contains("CPU");
    let has_fpu = mode_upper.contains("FPU");
    let has_ram = mode_upper.contains("RAM");

    let mode_checkboxes = row![
        checkbox("CACHE", has_cache)
            .on_toggle(|v| Message::Aida64ToggleModeComponent("CACHE".to_string(), v))
            .text_size(14).size(20).spacing(8),
        Space::with_width(16),
        checkbox("CPU", has_cpu)
            .on_toggle(|v| Message::Aida64ToggleModeComponent("CPU".to_string(), v))
            .text_size(14).size(20).spacing(8),
        Space::with_width(16),
        checkbox("FPU", has_fpu)
            .on_toggle(|v| Message::Aida64ToggleModeComponent("FPU".to_string(), v))
            .text_size(14).size(20).spacing(8),
        Space::with_width(16),
        checkbox("RAM", has_ram)
            .on_toggle(|v| Message::Aida64ToggleModeComponent("RAM".to_string(), v))
            .text_size(14).size(20).spacing(8),
    ]
    .spacing(4);

    column![
        row![
            section_header("AIDA64"),
            Space::with_width(Length::Fill),
            run_button("AIDA64", is_running),
        ].align_y(iced::Alignment::Center),
        Space::with_height(12),
        section_header("STRESS TEST MODE"),
        hint_text("Select one or more test types. At least one must be enabled."),
        Space::with_height(4),
        mode_checkboxes,
        Space::with_height(8),
        bool_field("Use AVX", cfg.use_avx, Message::Aida64UseAVX),
        text_field("Max Memory (%)", "90", &cfg.max_memory, |v| Message::Aida64MaxMemory(v), 260.0),
        Space::with_height(12),
        hint_text("RAM consumes most available memory. Max Memory only applies to RAM test."),
        hint_text("Requires the portable Engineer version in test_programs/aida64/"),
    ]
    .spacing(10)
    .into()
}

fn linpack_section(cfg: &LinpackConfig, is_running: bool) -> Element<'_, Message> {
    let versions: Vec<String> = LinpackVersion::ALL.iter().map(|v| v.as_str().to_string()).collect();
    let selected_ver = Some(cfg.version.as_str().to_string());

    let mode_locked = matches!(cfg.version, LinpackVersion::V2021 | LinpackVersion::V2024);

    let mut col = Column::new().spacing(10);
    col = col.push(
        row![
            section_header("LINPACK"),
            Space::with_width(Length::Fill),
            run_button("LINPACK", is_running),
        ].align_y(iced::Alignment::Center),
    );
    col = col.push(Space::with_height(12));
    col = col.push(dropdown("Version", versions, selected_ver, |v| Message::LinpackVersion(v), 260.0));

    if mode_locked {
        col = col.push(field_row(
            "Mode",
            text("FASTEST (forced by version)").size(15).color(style::TEXT_MUTED).into(),
        ));
    } else {
        let modes: Vec<String> = LinpackMode::ALL.iter().map(|m| m.as_str().to_string()).collect();
        let selected_mode = Some(cfg.mode.as_str().to_string());
        col = col.push(dropdown("Mode", modes, selected_mode, |v| Message::LinpackModeChanged(v), 260.0));
    }

    col = col.push(text_field("Memory", "2GB", &cfg.memory, |v| Message::LinpackMemory(v), 260.0));
    col = col.push(Space::with_height(12));

    if mode_locked {
        col = col.push(hint_text("Versions 2021/2024 always use FASTEST (AVX2). Mode cannot be changed."));
    }
    col = col.push(hint_text("More memory = longer per test. Higher memory may help find RAM/IMC issues."));

    col.into()
}

fn auto_mode_section(cfg: &AutomaticTestModeConfig) -> Element<'_, Message> {
    let inc_options = strvec(&["Default", "1", "2", "3", "5", "10"]);
    let selected_inc = pick_or_first(&cfg.increment_by, &inc_options);

    column![
        section_header("AUTOMATIC CO / VOLTAGE ADJUSTMENT"),
        Space::with_height(12),
        bool_field("Enable Automatic Adjustment", cfg.enable_automatic_adjustment, Message::AutoEnableAdjustment),
        text_field("Start Values", "CurrentValues", &cfg.start_values, |v| Message::AutoStartValues(v), 360.0),
        hint_text("CurrentValues, Minimum, a single value (e.g. -20), or per-core list (e.g. -15,-10,-15,-8,2,-20,0,-30)."),
        text_field("Max Value", "0", &cfg.max_value, |v| Message::AutoMaxValue(v), 260.0),
        dropdown("Increment By", inc_options, selected_inc, |v| Message::AutoIncrementBy(v), 260.0),
        bool_field("Voltage Only for Tested Core", cfg.set_voltage_only_for_tested_core, Message::AutoVoltageOnlyTested),
        bool_field("Repeat Core on Error", cfg.repeat_core_on_error, Message::AutoRepeatCoreOnError),
        Space::with_height(8),
        hint_text("Negative = undervolt (CO/voltage offset). Adjusted values are TEMPORARY and reset on reboot."),
        Space::with_height(8),
        section_header("CRASH RECOVERY"),
        Space::with_height(8),
        bool_field("Resume After Crash/Reboot", cfg.enable_resume_after_unexpected_exit, Message::AutoEnableResume),
        text_field("Wait Before Resume (s)", "120", &cfg.wait_before_automatic_resume, |v| Message::AutoWaitBeforeResume(v), 260.0),
        bool_field("Create System Restore Point", cfg.create_system_restore_point, Message::AutoCreateRestore),
        bool_field("Ask Before Creating Restore", cfg.ask_for_system_restore_point_creation, Message::AutoAskRestore),
        Space::with_height(12),
        hint_text("Adjusted values are TEMPORARY and reset on reboot."),
    ]
    .spacing(10)
    .into()
}

fn logging_section<'a>(cfg: &'a LoggingConfig, update: &'a UpdateConfig) -> Element<'a, Message> {
    let log_levels: Vec<String> = vec![
        "0 - None".to_string(),
        "1 - Verbose".to_string(),
        "2 - Debug".to_string(),
        "3 - Verbose + Console".to_string(),
        "4 - Debug + Console".to_string(),
    ];
    let selected_level = Some(match cfg.log_level.as_str() {
        "0" => "0 - None".to_string(),
        "1" => "1 - Verbose".to_string(),
        "3" => "3 - Verbose + Console".to_string(),
        "4" => "4 - Debug + Console".to_string(),
        _ => "2 - Debug".to_string(),
    });

    column![
        section_header("LOGGING"),
        Space::with_height(12),
        text_field("Log File Name", "CoreCycler", &cfg.name, |v| Message::LogName(v), 260.0),
        dropdown(
            "Log Level",
            log_levels,
            selected_level,
            |v| Message::LogLevel(v.chars().next().unwrap_or('2').to_string()),
            280.0,
        ),
        bool_field("Windows Event Log", cfg.use_windows_event_log, Message::LogUseEventLog),
        bool_field("Flush Disk Write Cache", cfg.flush_disk_write_cache, Message::LogFlushCache),
        Space::with_height(16),
        section_header("UPDATES"),
        Space::with_height(8),
        bool_field("Enable Update Check", update.enable_update_check, Message::UpdateEnableCheck),
        text_field("Check Frequency (hours)", "24", &update.update_check_frequency, |v| Message::UpdateFrequency(v), 260.0),
    ]
    .spacing(10)
    .into()
}

fn debug_section(cfg: &DebugConfig) -> Element<'_, Message> {
    let priorities: Vec<String> = ProcessPriority::ALL.iter().map(|p| p.as_str().to_string()).collect();
    let selected_priority = Some(cfg.stress_test_program_priority.as_str().to_string());

    let suspension_modes: Vec<String> = SuspensionMode::ALL.iter().map(|m| m.as_str().to_string()).collect();
    let selected_suspension = Some(cfg.mode_to_use_for_suspension.as_str().to_string());

    column![
        section_header("DEBUG"),
        Space::with_height(12),
        bool_field("Disable CPU Utilization Check", cfg.disable_cpu_utilization_check, Message::DebugDisableCpuCheck),
        bool_field("Use Windows Perf Counters", cfg.use_windows_perf_counters, Message::DebugUsePerfCounters),
        bool_field("Enable CPU Frequency Check", cfg.enable_cpu_frequency_check, Message::DebugEnableFreqCheck),
        text_field("Tick Interval (s)", "10", &cfg.tick_interval, |v| Message::DebugTickInterval(v), 260.0),
        text_field("Delay First Error Check (s)", "0", &cfg.delay_first_error_check, |v| Message::DebugDelayFirstCheck(v), 260.0),
        dropdown("Process Priority", priorities, selected_priority, |v| Message::DebugPriority(v), 260.0),
        bool_field("Show Stress Test Window", cfg.stress_test_program_window_to_foreground, Message::DebugShowWindow),
        text_field("Suspension Time (ms)", "1000", &cfg.suspension_time, |v| Message::DebugSuspensionTime(v), 260.0),
        dropdown("Suspension Mode", suspension_modes, selected_suspension, |v| Message::DebugSuspensionMode(v), 260.0),
    ]
    .spacing(10)
    .into()
}
