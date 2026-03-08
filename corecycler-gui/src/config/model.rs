/// All configuration sections for CoreCycler

#[derive(Debug, Clone)]
pub struct CoreCyclerConfig {
    pub general: GeneralConfig,
    pub prime95: Prime95Config,
    pub ycruncher: YCruncherConfig,
    pub aida64: Aida64Config,
    pub linpack: LinpackConfig,
    pub automatic_test_mode: AutomaticTestModeConfig,
    pub logging: LoggingConfig,
    pub update: UpdateConfig,
    pub prime95_custom: Prime95CustomConfig,
    pub debug: DebugConfig,
}

impl Default for CoreCyclerConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            prime95: Prime95Config::default(),
            ycruncher: YCruncherConfig::default(),
            aida64: Aida64Config::default(),
            linpack: LinpackConfig::default(),
            automatic_test_mode: AutomaticTestModeConfig::default(),
            logging: LoggingConfig::default(),
            update: UpdateConfig::default(),
            prime95_custom: Prime95CustomConfig::default(),
            debug: DebugConfig::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StressTestProgram {
    Prime95,
    Aida64,
    YCruncher,
    YCruncherOld,
    Linpack,
}

impl StressTestProgram {
    pub const ALL: &'static [StressTestProgram] = &[
        Self::Prime95,
        Self::Aida64,
        Self::YCruncher,
        Self::YCruncherOld,
        Self::Linpack,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Prime95 => "PRIME95",
            Self::Aida64 => "AIDA64",
            Self::YCruncher => "YCRUNCHER",
            Self::YCruncherOld => "YCRUNCHER_OLD",
            Self::Linpack => "LINPACK",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "AIDA64" => Self::Aida64,
            "YCRUNCHER" => Self::YCruncher,
            "YCRUNCHER_OLD" => Self::YCruncherOld,
            "LINPACK" => Self::Linpack,
            _ => Self::Prime95,
        }
    }
}

impl std::fmt::Display for StressTestProgram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreTestOrder {
    Default,
    Alternate,
    CorePairs,
    Random,
    Sequential,
    Custom(String),
}

impl CoreTestOrder {
    pub const PRESETS: &'static [&'static str] =
        &["Default", "Alternate", "CorePairs", "Random", "Sequential"];

    pub fn as_str(&self) -> &str {
        match self {
            Self::Default => "Default",
            Self::Alternate => "Alternate",
            Self::CorePairs => "CorePairs",
            Self::Random => "Random",
            Self::Sequential => "Sequential",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "Default" => Self::Default,
            "Alternate" => Self::Alternate,
            "CorePairs" => Self::CorePairs,
            "Random" => Self::Random,
            "Sequential" => Self::Sequential,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl std::fmt::Display for CoreTestOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct GeneralConfig {
    pub use_config_file: String,
    pub stress_test_program: StressTestProgram,
    pub runtime_per_core: String,
    pub suspend_periodically: bool,
    pub core_test_order: CoreTestOrder,
    pub skip_core_on_error: bool,
    pub stop_on_error: bool,
    pub number_of_threads: u8,
    pub assign_both_virtual_cores: bool,
    pub max_iterations: String,
    pub cores_to_ignore: String,
    pub restart_test_program_for_each_core: bool,
    pub delay_between_cores: String,
    pub beep_on_error: bool,
    pub flash_on_error: bool,
    pub look_for_whea_errors: bool,
    pub treat_whea_warning_as_error: bool,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            use_config_file: String::new(),
            stress_test_program: StressTestProgram::Prime95,
            runtime_per_core: "6m".to_string(),
            suspend_periodically: true,
            core_test_order: CoreTestOrder::Default,
            skip_core_on_error: true,
            stop_on_error: false,
            number_of_threads: 1,
            assign_both_virtual_cores: false,
            max_iterations: "10000".to_string(),
            cores_to_ignore: String::new(),
            restart_test_program_for_each_core: false,
            delay_between_cores: "15".to_string(),
            beep_on_error: true,
            flash_on_error: true,
            look_for_whea_errors: true,
            treat_whea_warning_as_error: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prime95Mode {
    SSE,
    AVX,
    AVX2,
    AVX512,
    Custom,
}

impl Prime95Mode {
    pub const ALL: &'static [Prime95Mode] = &[
        Self::SSE,
        Self::AVX,
        Self::AVX2,
        Self::AVX512,
        Self::Custom,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SSE => "SSE",
            Self::AVX => "AVX",
            Self::AVX2 => "AVX2",
            Self::AVX512 => "AVX512",
            Self::Custom => "CUSTOM",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "AVX" => Self::AVX,
            "AVX2" => Self::AVX2,
            "AVX512" => Self::AVX512,
            "CUSTOM" => Self::Custom,
            _ => Self::SSE,
        }
    }
}

impl std::fmt::Display for Prime95Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FFTSize {
    Smallest,
    Small,
    Large,
    Huge,
    All,
    Moderate,
    Heavy,
    HeavyShort,
    Custom(String),
}

impl FFTSize {
    pub const PRESETS: &'static [&'static str] = &[
        "Smallest",
        "Small",
        "Large",
        "Huge",
        "All",
        "Moderate",
        "Heavy",
        "HeavyShort",
    ];

    pub fn as_str(&self) -> &str {
        match self {
            Self::Smallest => "Smallest",
            Self::Small => "Small",
            Self::Large => "Large",
            Self::Huge => "Huge",
            Self::All => "All",
            Self::Moderate => "Moderate",
            Self::Heavy => "Heavy",
            Self::HeavyShort => "HeavyShort",
            Self::Custom(s) => s.as_str(),
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "Smallest" => Self::Smallest,
            "Small" => Self::Small,
            "Large" => Self::Large,
            "Huge" => Self::Huge,
            "All" => Self::All,
            "Moderate" => Self::Moderate,
            "Heavy" => Self::Heavy,
            "HeavyShort" => Self::HeavyShort,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl std::fmt::Display for FFTSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Prime95Config {
    pub mode: Prime95Mode,
    pub fft_size: FFTSize,
}

impl Default for Prime95Config {
    fn default() -> Self {
        Self {
            mode: Prime95Mode::SSE,
            fft_size: FFTSize::Huge,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YCruncherMode {
    Auto,
    X86,
    P4P,
    A64,
    NHM,
    SNB,
    BD1,
    BD2,
    HSW,
    BDW,
    SKX,
    ZN1,
    CNL,
    ZN2,
    ZN3,
    ZN4,
    ZN5,
}

impl YCruncherMode {
    /// Modes available for the new y-cruncher (v0.8+)
    pub const NEW: &'static [YCruncherMode] = &[
        Self::Auto,
        Self::X86,
        Self::P4P,
        Self::A64,
        Self::NHM,
        Self::SNB,
        Self::BD2,
        Self::HSW,
        Self::BDW,
        Self::SKX,
        Self::ZN1,
        Self::CNL,
        Self::ZN2,
        Self::ZN4,
        Self::ZN5,
    ];

    /// Modes available for the old y-cruncher (v0.7.10)
    pub const OLD: &'static [YCruncherMode] = &[
        Self::Auto,
        Self::X86,
        Self::P4P,
        Self::A64,
        Self::NHM,
        Self::BD1,
        Self::SNB,
        Self::HSW,
        Self::BDW,
        Self::SKX,
        Self::ZN1,
        Self::CNL,
        Self::ZN2,
        Self::ZN3,
        Self::ZN4,
    ];

    /// All modes (superset)
    pub const ALL: &'static [YCruncherMode] = &[
        Self::Auto,
        Self::X86,
        Self::P4P,
        Self::A64,
        Self::NHM,
        Self::SNB,
        Self::BD1,
        Self::BD2,
        Self::HSW,
        Self::BDW,
        Self::SKX,
        Self::ZN1,
        Self::CNL,
        Self::ZN2,
        Self::ZN3,
        Self::ZN4,
        Self::ZN5,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::X86 => "00-x86",
            Self::P4P => "04-P4P",
            Self::A64 => "05-A64",
            Self::NHM => "08-NHM",
            Self::SNB => "11-SNB",
            Self::BD1 => "11-BD1",
            Self::BD2 => "12-BD2",
            Self::HSW => "13-HSW",
            Self::BDW => "14-BDW",
            Self::SKX => "17-SKX",
            Self::ZN1 => "17-ZN1",
            Self::CNL => "18-CNL",
            Self::ZN2 => "19-ZN2",
            Self::ZN3 => "20-ZN3",
            Self::ZN4 => "22-ZN4",
            Self::ZN5 => "24-ZN5",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Auto => "auto (Auto-detect)",
            Self::X86 => "00-x86 (Legacy)",
            Self::P4P => "04-P4P (Pentium 4)",
            Self::A64 => "05-A64 (Athlon 64)",
            Self::NHM => "08-NHM (Nehalem)",
            Self::SNB => "11-SNB (Sandy Bridge)",
            Self::BD1 => "11-BD1 (Bulldozer, old yCruncher)",
            Self::BD2 => "12-BD2 (Piledriver)",
            Self::HSW => "13-HSW (Haswell)",
            Self::BDW => "14-BDW (Broadwell)",
            Self::SKX => "17-SKX (Skylake-X AVX512)",
            Self::ZN1 => "17-ZN1 (Zen 1)",
            Self::CNL => "18-CNL (Cannon Lake AVX512)",
            Self::ZN2 => "19-ZN2 (Zen 2/3)",
            Self::ZN3 => "20-ZN3 (Zen 3, old yCruncher)",
            Self::ZN4 => "22-ZN4 (Zen 4 AVX512)",
            Self::ZN5 => "24-ZN5 (Zen 5 AVX512)",
        }
    }

    pub fn from_str(s: &str) -> Self {
        let s = s.trim();
        if s.eq_ignore_ascii_case("auto") {
            return Self::Auto;
        }
        // Match on the prefix before any " ~" or " ("
        let key = s.split(" ~").next().unwrap_or(s).split(" (").next().unwrap_or(s).trim();
        match key {
            "04-P4P" => Self::P4P,
            "05-A64" => Self::A64,
            "08-NHM" => Self::NHM,
            "11-SNB" => Self::SNB,
            "11-BD1" => Self::BD1,
            "12-BD2" => Self::BD2,
            "13-HSW" => Self::HSW,
            "14-BDW" => Self::BDW,
            "17-SKX" => Self::SKX,
            "17-ZN1" => Self::ZN1,
            "18-CNL" => Self::CNL,
            "19-ZN2" => Self::ZN2,
            "20-ZN3" => Self::ZN3,
            "22-ZN4" => Self::ZN4,
            "24-ZN5" => Self::ZN5,
            _ => Self::X86,
        }
    }
}

impl std::fmt::Display for YCruncherMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

#[derive(Debug, Clone)]
pub struct YCruncherConfig {
    pub mode: YCruncherMode,
    pub tests: String,
    pub test_duration: String,
    pub memory: String,
    pub enable_logging_wrapper: bool,
}

impl YCruncherConfig {
    /// New y-cruncher (v0.8+) test names
    pub const NEW_TESTS: &'static [(&'static str, &'static str)] = &[
        ("BKT", "Basecase + Karatsuba (Scalar Int)"),
        ("BBP", "BBP Digit Extraction (AVX2 Float)"),
        ("SFTv4", "Small In-Cache FFTv4 (AVX2 Float)"),
        ("SNT", "Small In-Cache N63 (AVX2 Int)"),
        ("SVT", "Small In-Cache VT3 (AVX2 Float)"),
        ("FFTv4", "Fast Fourier Transform v4 (AVX2 Float)"),
        ("N63", "Classic NTT v2 (AVX2 Int)"),
        ("VT3", "Vector Transform v3 (AVX2 Float)"),
    ];

    /// Old y-cruncher (v0.7.10) test names
    pub const OLD_TESTS: &'static [(&'static str, &'static str)] = &[
        ("BKT", "Basecase + Karatsuba"),
        ("BBP", "BBP Digit Extraction"),
        ("SFT", "Small In-Cache FFT"),
        ("FFT", "Fast Fourier Transform"),
        ("N32", "Classic NTT 32-bit"),
        ("N64", "Classic NTT 64-bit"),
        ("HNT", "Hybrid NTT"),
        ("VST", "Vector Transform"),
        ("C17", "Component 17"),
    ];

    pub const NEW_DEFAULT_TESTS: &'static str = "BKT, BBP, SFTv4, SNT, SVT, FFTv4, N63, VT3";
    pub const OLD_DEFAULT_TESTS: &'static str = "BKT, BBP, SFT, FFT, N32, N64, HNT, VST";
}

impl Default for YCruncherConfig {
    fn default() -> Self {
        Self {
            mode: YCruncherMode::X86,
            tests: Self::NEW_DEFAULT_TESTS.to_string(),
            test_duration: "60".to_string(),
            memory: "Default".to_string(),
            enable_logging_wrapper: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Aida64Config {
    pub mode: String,
    pub use_avx: bool,
    pub max_memory: String,
}

impl Default for Aida64Config {
    fn default() -> Self {
        Self {
            mode: "CACHE".to_string(),
            use_avx: false,
            max_memory: "90".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinpackVersion {
    V2018,
    V2019,
    V2021,
    V2024,
}

impl LinpackVersion {
    pub const ALL: &'static [LinpackVersion] = &[Self::V2018, Self::V2019, Self::V2021, Self::V2024];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V2018 => "2018",
            Self::V2019 => "2019",
            Self::V2021 => "2021",
            Self::V2024 => "2024",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "2019" => Self::V2019,
            "2021" => Self::V2021,
            "2024" => Self::V2024,
            _ => Self::V2018,
        }
    }
}

impl std::fmt::Display for LinpackVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LinpackMode {
    Slowest,
    Slow,
    Medium,
    Fast,
    Fastest,
}

impl LinpackMode {
    pub const ALL: &'static [LinpackMode] = &[
        Self::Slowest,
        Self::Slow,
        Self::Medium,
        Self::Fast,
        Self::Fastest,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Slowest => "SLOWEST",
            Self::Slow => "SLOW",
            Self::Medium => "MEDIUM",
            Self::Fast => "FAST",
            Self::Fastest => "FASTEST",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_uppercase().as_str() {
            "SLOWEST" => Self::Slowest,
            "SLOW" => Self::Slow,
            "FAST" => Self::Fast,
            "FASTEST" => Self::Fastest,
            _ => Self::Medium,
        }
    }
}

impl std::fmt::Display for LinpackMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct LinpackConfig {
    pub version: LinpackVersion,
    pub mode: LinpackMode,
    pub memory: String,
}

impl Default for LinpackConfig {
    fn default() -> Self {
        Self {
            version: LinpackVersion::V2018,
            mode: LinpackMode::Medium,
            memory: "2GB".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AutomaticTestModeConfig {
    pub enable_automatic_adjustment: bool,
    pub start_values: String,
    pub max_value: String,
    pub increment_by: String,
    pub set_voltage_only_for_tested_core: bool,
    pub repeat_core_on_error: bool,
    pub enable_resume_after_unexpected_exit: bool,
    pub wait_before_automatic_resume: String,
    pub create_system_restore_point: bool,
    pub ask_for_system_restore_point_creation: bool,
}

impl Default for AutomaticTestModeConfig {
    fn default() -> Self {
        Self {
            enable_automatic_adjustment: false,
            start_values: "CurrentValues".to_string(),
            max_value: "0".to_string(),
            increment_by: "Default".to_string(),
            set_voltage_only_for_tested_core: false,
            repeat_core_on_error: true,
            enable_resume_after_unexpected_exit: false,
            wait_before_automatic_resume: "120".to_string(),
            create_system_restore_point: true,
            ask_for_system_restore_point_creation: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub name: String,
    pub log_level: String,
    pub use_windows_event_log: bool,
    pub flush_disk_write_cache: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            name: "CoreCycler".to_string(),
            log_level: "2".to_string(),
            use_windows_event_log: true,
            flush_disk_write_cache: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpdateConfig {
    pub enable_update_check: bool,
    pub update_check_frequency: String,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            enable_update_check: true,
            update_check_frequency: "24".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Prime95CustomConfig {
    pub cpu_supports_avx: bool,
    pub cpu_supports_avx2: bool,
    pub cpu_supports_fma3: bool,
    pub cpu_supports_avx512: bool,
    pub min_torture_fft: String,
    pub max_torture_fft: String,
    pub torture_mem: String,
    pub torture_time: String,
}

impl Default for Prime95CustomConfig {
    fn default() -> Self {
        Self {
            cpu_supports_avx: false,
            cpu_supports_avx2: false,
            cpu_supports_fma3: false,
            cpu_supports_avx512: false,
            min_torture_fft: "4".to_string(),
            max_torture_fft: "8192".to_string(),
            torture_mem: "0".to_string(),
            torture_time: "1".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessPriority {
    Idle,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
    RealTime,
}

impl ProcessPriority {
    pub const ALL: &'static [ProcessPriority] = &[
        Self::Idle,
        Self::BelowNormal,
        Self::Normal,
        Self::AboveNormal,
        Self::High,
        Self::RealTime,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::BelowNormal => "BelowNormal",
            Self::Normal => "Normal",
            Self::AboveNormal => "AboveNormal",
            Self::High => "High",
            Self::RealTime => "RealTime",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "Idle" => Self::Idle,
            "BelowNormal" => Self::BelowNormal,
            "AboveNormal" => Self::AboveNormal,
            "High" => Self::High,
            "RealTime" => Self::RealTime,
            _ => Self::Normal,
        }
    }
}

impl std::fmt::Display for ProcessPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuspensionMode {
    Debugger,
    Threads,
}

impl SuspensionMode {
    pub const ALL: &'static [SuspensionMode] = &[Self::Debugger, Self::Threads];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Debugger => "Debugger",
            Self::Threads => "Threads",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim() {
            "Debugger" => Self::Debugger,
            _ => Self::Threads,
        }
    }
}

impl std::fmt::Display for SuspensionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct DebugConfig {
    pub disable_cpu_utilization_check: bool,
    pub use_windows_perf_counters: bool,
    pub enable_cpu_frequency_check: bool,
    pub tick_interval: String,
    pub delay_first_error_check: String,
    pub stress_test_program_priority: ProcessPriority,
    pub stress_test_program_window_to_foreground: bool,
    pub suspension_time: String,
    pub mode_to_use_for_suspension: SuspensionMode,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            disable_cpu_utilization_check: false,
            use_windows_perf_counters: false,
            enable_cpu_frequency_check: false,
            tick_interval: "10".to_string(),
            delay_first_error_check: "0".to_string(),
            stress_test_program_priority: ProcessPriority::Normal,
            stress_test_program_window_to_foreground: false,
            suspension_time: "1000".to_string(),
            mode_to_use_for_suspension: SuspensionMode::Threads,
        }
    }
}
