use std::collections::HashMap;
use std::path::Path;

use super::model::*;

fn strip_inline_comment(value: &str) -> &str {
    value.split('#').next().unwrap_or(value).trim()
}

/// Parse a CoreCycler INI config file into a HashMap of section -> (key -> value)
pub fn parse_ini(path: &Path) -> Result<HashMap<String, HashMap<String, String>>, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let mut sections: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut current_section = String::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Skip comments and empty lines
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Section header
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].to_string();
            sections.entry(current_section.clone()).or_default();
            continue;
        }

        // Key = Value
        if let Some(eq_pos) = trimmed.find('=') {
            let key = trimmed[..eq_pos].trim().to_string();
            let raw_value = trimmed[eq_pos + 1..].trim();
            let value = strip_inline_comment(raw_value).to_string();
            if !current_section.is_empty() {
                sections
                    .entry(current_section.clone())
                    .or_default()
                    .insert(key, value);
            }
        }
    }

    Ok(sections)
}

fn get_str(sections: &HashMap<String, HashMap<String, String>>, section: &str, key: &str) -> Option<String> {
    sections.get(section)?.get(key).cloned()
}

fn get_bool(sections: &HashMap<String, HashMap<String, String>>, section: &str, key: &str) -> Option<bool> {
    let v = get_str(sections, section, key)?;
    Some(v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Load config from an INI file, falling back to defaults for missing values
pub fn load_config(path: &Path) -> Result<CoreCyclerConfig, String> {
    let sections = parse_ini(path)?;
    let mut cfg = CoreCyclerConfig::default();

    // [General]
    if let Some(v) = get_str(&sections, "General", "useConfigFile") {
        cfg.general.use_config_file = v;
    }
    if let Some(v) = get_str(&sections, "General", "stressTestProgram") {
        cfg.general.stress_test_program = StressTestProgram::from_str(&v);
    }
    if let Some(v) = get_str(&sections, "General", "runtimePerCore") {
        cfg.general.runtime_per_core = v;
    }
    if let Some(v) = get_bool(&sections, "General", "suspendPeriodically") {
        cfg.general.suspend_periodically = v;
    }
    if let Some(v) = get_str(&sections, "General", "coreTestOrder") {
        cfg.general.core_test_order = CoreTestOrder::from_str(&v);
    }
    if let Some(v) = get_bool(&sections, "General", "skipCoreOnError") {
        cfg.general.skip_core_on_error = v;
    }
    if let Some(v) = get_bool(&sections, "General", "stopOnError") {
        cfg.general.stop_on_error = v;
    }
    if let Some(v) = get_str(&sections, "General", "numberOfThreads") {
        cfg.general.number_of_threads = v.parse().unwrap_or(1);
    }
    if let Some(v) = get_bool(&sections, "General", "assignBothVirtualCoresForSingleThread") {
        cfg.general.assign_both_virtual_cores = v;
    }
    if let Some(v) = get_str(&sections, "General", "maxIterations") {
        cfg.general.max_iterations = v;
    }
    if let Some(v) = get_str(&sections, "General", "coresToIgnore") {
        cfg.general.cores_to_ignore = v;
    }
    if let Some(v) = get_bool(&sections, "General", "restartTestProgramForEachCore") {
        cfg.general.restart_test_program_for_each_core = v;
    }
    if let Some(v) = get_str(&sections, "General", "delayBetweenCores") {
        cfg.general.delay_between_cores = v;
    }
    if let Some(v) = get_bool(&sections, "General", "beepOnError") {
        cfg.general.beep_on_error = v;
    }
    if let Some(v) = get_bool(&sections, "General", "flashOnError") {
        cfg.general.flash_on_error = v;
    }
    if let Some(v) = get_bool(&sections, "General", "lookForWheaErrors") {
        cfg.general.look_for_whea_errors = v;
    }
    if let Some(v) = get_bool(&sections, "General", "treatWheaWarningAsError") {
        cfg.general.treat_whea_warning_as_error = v;
    }

    // [Prime95]
    if let Some(v) = get_str(&sections, "Prime95", "mode") {
        cfg.prime95.mode = Prime95Mode::from_str(&v);
    }
    if let Some(v) = get_str(&sections, "Prime95", "FFTSize") {
        cfg.prime95.fft_size = FFTSize::from_str(&v);
    }

    // [yCruncher]
    if let Some(v) = get_str(&sections, "yCruncher", "mode") {
        cfg.ycruncher.mode = YCruncherMode::from_str(&v);
    }
    if let Some(v) = get_str(&sections, "yCruncher", "tests") {
        cfg.ycruncher.tests = v;
    }
    if let Some(v) = get_str(&sections, "yCruncher", "testDuration") {
        cfg.ycruncher.test_duration = v;
    }
    if let Some(v) = get_str(&sections, "yCruncher", "memory") {
        cfg.ycruncher.memory = v;
    }
    if let Some(v) = get_bool(&sections, "yCruncher", "enableYCruncherLoggingWrapper") {
        cfg.ycruncher.enable_logging_wrapper = v;
    }

    // [Aida64]
    if let Some(v) = get_str(&sections, "Aida64", "mode") {
        cfg.aida64.mode = v;
    }
    if let Some(v) = get_bool(&sections, "Aida64", "useAVX") {
        cfg.aida64.use_avx = v;
    }
    if let Some(v) = get_str(&sections, "Aida64", "maxMemory") {
        cfg.aida64.max_memory = v;
    }

    // [Linpack]
    if let Some(v) = get_str(&sections, "Linpack", "version") {
        cfg.linpack.version = LinpackVersion::from_str(&v);
    }
    if let Some(v) = get_str(&sections, "Linpack", "mode") {
        cfg.linpack.mode = LinpackMode::from_str(&v);
    }
    if let Some(v) = get_str(&sections, "Linpack", "memory") {
        cfg.linpack.memory = v;
    }

    // [AutomaticTestMode]
    if let Some(v) = get_bool(&sections, "AutomaticTestMode", "enableAutomaticAdjustment") {
        cfg.automatic_test_mode.enable_automatic_adjustment = v;
    }
    if let Some(v) = get_str(&sections, "AutomaticTestMode", "startValues") {
        cfg.automatic_test_mode.start_values = v;
    }
    if let Some(v) = get_str(&sections, "AutomaticTestMode", "maxValue") {
        cfg.automatic_test_mode.max_value = v;
    }
    if let Some(v) = get_str(&sections, "AutomaticTestMode", "incrementBy") {
        cfg.automatic_test_mode.increment_by = v;
    }
    if let Some(v) = get_bool(&sections, "AutomaticTestMode", "setVoltageOnlyForTestedCore") {
        cfg.automatic_test_mode.set_voltage_only_for_tested_core = v;
    }
    if let Some(v) = get_bool(&sections, "AutomaticTestMode", "repeatCoreOnError") {
        cfg.automatic_test_mode.repeat_core_on_error = v;
    }
    if let Some(v) = get_bool(&sections, "AutomaticTestMode", "enableResumeAfterUnexpectedExit") {
        cfg.automatic_test_mode.enable_resume_after_unexpected_exit = v;
    }
    if let Some(v) = get_str(&sections, "AutomaticTestMode", "waitBeforeAutomaticResume") {
        cfg.automatic_test_mode.wait_before_automatic_resume = v;
    }
    if let Some(v) = get_bool(&sections, "AutomaticTestMode", "createSystemRestorePoint") {
        cfg.automatic_test_mode.create_system_restore_point = v;
    }
    if let Some(v) = get_bool(&sections, "AutomaticTestMode", "askForSystemRestorePointCreation") {
        cfg.automatic_test_mode.ask_for_system_restore_point_creation = v;
    }

    // [Logging]
    if let Some(v) = get_str(&sections, "Logging", "name") {
        cfg.logging.name = v;
    }
    if let Some(v) = get_str(&sections, "Logging", "logLevel") {
        cfg.logging.log_level = v;
    }
    if let Some(v) = get_bool(&sections, "Logging", "useWindowsEventLog") {
        cfg.logging.use_windows_event_log = v;
    }
    if let Some(v) = get_bool(&sections, "Logging", "flushDiskWriteCache") {
        cfg.logging.flush_disk_write_cache = v;
    }

    // [Update]
    if let Some(v) = get_bool(&sections, "Update", "enableUpdateCheck") {
        cfg.update.enable_update_check = v;
    }
    if let Some(v) = get_str(&sections, "Update", "updateCheckFrequency") {
        cfg.update.update_check_frequency = v;
    }

    // [Prime95Custom]
    if let Some(v) = get_bool(&sections, "Prime95Custom", "CpuSupportsAVX") {
        cfg.prime95_custom.cpu_supports_avx = v;
    }
    if let Some(v) = get_bool(&sections, "Prime95Custom", "CpuSupportsAVX2") {
        cfg.prime95_custom.cpu_supports_avx2 = v;
    }
    if let Some(v) = get_bool(&sections, "Prime95Custom", "CpuSupportsFMA3") {
        cfg.prime95_custom.cpu_supports_fma3 = v;
    }
    if let Some(v) = get_bool(&sections, "Prime95Custom", "CpuSupportsAVX512") {
        cfg.prime95_custom.cpu_supports_avx512 = v;
    }
    if let Some(v) = get_str(&sections, "Prime95Custom", "MinTortureFFT") {
        cfg.prime95_custom.min_torture_fft = v;
    }
    if let Some(v) = get_str(&sections, "Prime95Custom", "MaxTortureFFT") {
        cfg.prime95_custom.max_torture_fft = v;
    }
    if let Some(v) = get_str(&sections, "Prime95Custom", "TortureMem") {
        cfg.prime95_custom.torture_mem = v;
    }
    if let Some(v) = get_str(&sections, "Prime95Custom", "TortureTime") {
        cfg.prime95_custom.torture_time = v;
    }

    // [Debug]
    if let Some(v) = get_bool(&sections, "Debug", "disableCpuUtilizationCheck") {
        cfg.debug.disable_cpu_utilization_check = v;
    }
    if let Some(v) = get_bool(&sections, "Debug", "useWindowsPerformanceCountersForCpuUtilization") {
        cfg.debug.use_windows_perf_counters = v;
    }
    if let Some(v) = get_bool(&sections, "Debug", "enableCpuFrequencyCheck") {
        cfg.debug.enable_cpu_frequency_check = v;
    }
    if let Some(v) = get_str(&sections, "Debug", "tickInterval") {
        cfg.debug.tick_interval = v;
    }
    if let Some(v) = get_str(&sections, "Debug", "delayFirstErrorCheck") {
        cfg.debug.delay_first_error_check = v;
    }
    if let Some(v) = get_str(&sections, "Debug", "stressTestProgramPriority") {
        cfg.debug.stress_test_program_priority = ProcessPriority::from_str(&v);
    }
    if let Some(v) = get_bool(&sections, "Debug", "stressTestProgramWindowToForeground") {
        cfg.debug.stress_test_program_window_to_foreground = v;
    }
    if let Some(v) = get_str(&sections, "Debug", "suspensionTime") {
        cfg.debug.suspension_time = v;
    }
    if let Some(v) = get_str(&sections, "Debug", "modeToUseForSuspension") {
        cfg.debug.mode_to_use_for_suspension = SuspensionMode::from_str(&v);
    }

    Ok(cfg)
}

/// Write config to an INI file (only non-default values, compact format)
pub fn save_config(path: &Path, cfg: &CoreCyclerConfig) -> Result<(), String> {
    let mut out = String::new();
    let d = CoreCyclerConfig::default();

    out.push_str("# CoreCycler config - Generated by CoreCycler GUI\n\n");

    // [General]
    out.push_str("[General]\n");
    write_str(&mut out, "useConfigFile", &cfg.general.use_config_file);
    out.push_str(&format!("stressTestProgram = {}\n", cfg.general.stress_test_program.as_str()));
    write_str(&mut out, "runtimePerCore", &cfg.general.runtime_per_core);
    write_bool(&mut out, "suspendPeriodically", cfg.general.suspend_periodically);
    out.push_str(&format!("coreTestOrder = {}\n", cfg.general.core_test_order.as_str()));
    write_bool(&mut out, "skipCoreOnError", cfg.general.skip_core_on_error);
    write_bool(&mut out, "stopOnError", cfg.general.stop_on_error);
    out.push_str(&format!("numberOfThreads = {}\n", cfg.general.number_of_threads));
    write_bool(&mut out, "assignBothVirtualCoresForSingleThread", cfg.general.assign_both_virtual_cores);
    write_str(&mut out, "maxIterations", &cfg.general.max_iterations);
    write_str(&mut out, "coresToIgnore", &cfg.general.cores_to_ignore);
    write_bool(&mut out, "restartTestProgramForEachCore", cfg.general.restart_test_program_for_each_core);
    write_str(&mut out, "delayBetweenCores", &cfg.general.delay_between_cores);
    write_bool(&mut out, "beepOnError", cfg.general.beep_on_error);
    write_bool(&mut out, "flashOnError", cfg.general.flash_on_error);
    write_bool(&mut out, "lookForWheaErrors", cfg.general.look_for_whea_errors);
    write_bool(&mut out, "treatWheaWarningAsError", cfg.general.treat_whea_warning_as_error);
    out.push('\n');

    // [Prime95]
    out.push_str("[Prime95]\n");
    out.push_str(&format!("mode = {}\n", cfg.prime95.mode.as_str()));
    out.push_str(&format!("FFTSize = {}\n", cfg.prime95.fft_size.as_str()));
    out.push('\n');

    // [yCruncher]
    out.push_str("[yCruncher]\n");
    out.push_str(&format!("mode = {}\n", cfg.ycruncher.mode.as_str()));
    write_str(&mut out, "tests", &cfg.ycruncher.tests);
    write_str(&mut out, "testDuration", &cfg.ycruncher.test_duration);
    write_str(&mut out, "memory", &cfg.ycruncher.memory);
    write_bool(&mut out, "enableYCruncherLoggingWrapper", cfg.ycruncher.enable_logging_wrapper);
    out.push('\n');

    // [Aida64]
    out.push_str("[Aida64]\n");
    write_str(&mut out, "mode", &cfg.aida64.mode);
    write_bool(&mut out, "useAVX", cfg.aida64.use_avx);
    write_str(&mut out, "maxMemory", &cfg.aida64.max_memory);
    out.push('\n');

    // [Linpack]
    out.push_str("[Linpack]\n");
    out.push_str(&format!("version = {}\n", cfg.linpack.version.as_str()));
    out.push_str(&format!("mode = {}\n", cfg.linpack.mode.as_str()));
    write_str(&mut out, "memory", &cfg.linpack.memory);
    out.push('\n');

    // [AutomaticTestMode]
    out.push_str("[AutomaticTestMode]\n");
    write_bool(&mut out, "enableAutomaticAdjustment", cfg.automatic_test_mode.enable_automatic_adjustment);
    write_str(&mut out, "startValues", &cfg.automatic_test_mode.start_values);
    write_str(&mut out, "maxValue", &cfg.automatic_test_mode.max_value);
    write_str(&mut out, "incrementBy", &cfg.automatic_test_mode.increment_by);
    write_bool(&mut out, "setVoltageOnlyForTestedCore", cfg.automatic_test_mode.set_voltage_only_for_tested_core);
    write_bool(&mut out, "repeatCoreOnError", cfg.automatic_test_mode.repeat_core_on_error);
    write_bool(&mut out, "enableResumeAfterUnexpectedExit", cfg.automatic_test_mode.enable_resume_after_unexpected_exit);
    write_str(&mut out, "waitBeforeAutomaticResume", &cfg.automatic_test_mode.wait_before_automatic_resume);
    write_bool(&mut out, "createSystemRestorePoint", cfg.automatic_test_mode.create_system_restore_point);
    write_bool(&mut out, "askForSystemRestorePointCreation", cfg.automatic_test_mode.ask_for_system_restore_point_creation);
    out.push('\n');

    // [Logging]
    out.push_str("[Logging]\n");
    write_str(&mut out, "name", &cfg.logging.name);
    write_str(&mut out, "logLevel", &cfg.logging.log_level);
    write_bool(&mut out, "useWindowsEventLog", cfg.logging.use_windows_event_log);
    write_bool(&mut out, "flushDiskWriteCache", cfg.logging.flush_disk_write_cache);
    out.push('\n');

    // [Update]
    out.push_str("[Update]\n");
    write_bool(&mut out, "enableUpdateCheck", cfg.update.enable_update_check);
    write_str(&mut out, "updateCheckFrequency", &cfg.update.update_check_frequency);
    out.push('\n');

    // [Prime95Custom]
    out.push_str("[Prime95Custom]\n");
    write_bool(&mut out, "CpuSupportsAVX", cfg.prime95_custom.cpu_supports_avx);
    write_bool(&mut out, "CpuSupportsAVX2", cfg.prime95_custom.cpu_supports_avx2);
    write_bool(&mut out, "CpuSupportsFMA3", cfg.prime95_custom.cpu_supports_fma3);
    write_bool(&mut out, "CpuSupportsAVX512", cfg.prime95_custom.cpu_supports_avx512);
    write_str(&mut out, "MinTortureFFT", &cfg.prime95_custom.min_torture_fft);
    write_str(&mut out, "MaxTortureFFT", &cfg.prime95_custom.max_torture_fft);
    write_str(&mut out, "TortureMem", &cfg.prime95_custom.torture_mem);
    write_str(&mut out, "TortureTime", &cfg.prime95_custom.torture_time);
    out.push('\n');

    // [Debug]
    out.push_str("[Debug]\n");
    write_bool(&mut out, "disableCpuUtilizationCheck", cfg.debug.disable_cpu_utilization_check);
    write_bool(&mut out, "useWindowsPerformanceCountersForCpuUtilization", cfg.debug.use_windows_perf_counters);
    write_bool(&mut out, "enableCpuFrequencyCheck", cfg.debug.enable_cpu_frequency_check);
    write_str(&mut out, "tickInterval", &cfg.debug.tick_interval);
    write_str(&mut out, "delayFirstErrorCheck", &cfg.debug.delay_first_error_check);
    out.push_str(&format!("stressTestProgramPriority = {}\n", cfg.debug.stress_test_program_priority.as_str()));
    write_bool(&mut out, "stressTestProgramWindowToForeground", cfg.debug.stress_test_program_window_to_foreground);
    write_str(&mut out, "suspensionTime", &cfg.debug.suspension_time);
    out.push_str(&format!("modeToUseForSuspension = {}\n", cfg.debug.mode_to_use_for_suspension.as_str()));

    std::fs::write(path, out)
        .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

fn write_str(out: &mut String, key: &str, val: &str) {
    out.push_str(&format!("{} = {}\n", key, val));
}

fn write_bool(out: &mut String, key: &str, val: bool) {
    out.push_str(&format!("{} = {}\n", key, if val { "1" } else { "0" }));
}

/// Get list of preset config files from the configs directory
pub fn list_preset_configs(base_dir: &Path) -> Vec<String> {
    let configs_dir = base_dir.join("configs");
    let mut presets = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&configs_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".ini") || name.ends_with(".config.ini") {
                if name != "default.config.ini" {
                    presets.push(name);
                }
            }
        }
    }
    presets.sort();
    presets
}

#[cfg(test)]
mod tests {
    use super::strip_inline_comment;

    #[test]
    fn strips_inline_hash_comments_like_corecycler() {
        assert_eq!(strip_inline_comment("auto # comment"), "auto");
        assert_eq!(strip_inline_comment("auto#comment"), "auto");
        assert_eq!(strip_inline_comment("CurrentValues"), "CurrentValues");
        assert_eq!(strip_inline_comment(""), "");
    }
}
