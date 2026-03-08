#![allow(unused)]

mod app;
mod config;
mod runner;
mod ui;

use std::path::PathBuf;

use iced::Theme;

fn main() -> iced::Result {
    // Determine the script directory (parent of corecycler-gui)
    let script_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    // If script-corecycler.ps1 isn't there, check current dir and parent
    let script_dir = find_script_dir(script_dir);

    let result = iced::application("CoreCycler GUI", app::App::update, app::App::view)
        .theme(|_app| Theme::Dark)
        .subscription(app::App::subscription)
        .window_size((1400.0, 800.0))
        .run_with(move || app::App::new(script_dir.clone()));

    // Force exit when window closes - kills tokio tasks and any child processes
    std::process::exit(0);
}

fn find_script_dir(initial: PathBuf) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let candidates = [
        initial.clone(),
        initial.join(".."),
        initial.join("../.."),       // cargo run: target/ -> corecycler-gui/ -> CoreCycler root
        initial.join("../../.."),    // cargo run: target/debug/ via 2-parent -> up 3 more
        cwd.clone(),
        cwd.join(".."),             // if cwd is corecycler-gui/, parent is CoreCycler root
    ];

    for dir in &candidates {
        let canonical = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.clone());
        if canonical.join("script-corecycler.ps1").exists() {
            return canonical;
        }
    }

    initial
}
