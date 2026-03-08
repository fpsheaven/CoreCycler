use iced::widget::{
    button, column, container, horizontal_rule, row, scrollable, text, Column, Row, Space,
};
use iced::{Element, Length};

use crate::app::{CoreStatus, Message};
use crate::ui::style;

/// Build the monitor panel (right side)
pub fn monitor_view(app: &crate::app::App) -> Element<'_, Message> {
    let cores = &app.cores;
    let log_lines = &app.log_lines;
    let current_iteration = app.current_iteration;
    let status_text = &app.status_text;
    let is_running = app.is_running;

    let elapsed = app.elapsed_display();
    let program = &app.running_program;

    let status_line = if is_running {
        row![
            text(format!("{}", program)).size(14).color(style::SUCCESS),
            Space::with_width(16),
            text(format!("{}  |  Iteration {}", elapsed, current_iteration))
                .size(14)
                .color(style::TEXT_SECONDARY),
            Space::with_width(16),
            text(status_text).size(14).color(style::TEXT_PRIMARY),
        ]
        .align_y(iced::Alignment::Center)
    } else {
        row![
            text(status_text).size(14).color(style::TEXT_SECONDARY),
        ]
        .align_y(iced::Alignment::Center)
    };

    // Core status grid
    let core_grid = container(
        column![
            text("CORES").size(14).color(style::TEXT_MUTED),
            Space::with_height(12),
            build_core_grid(cores),
        ],
    )
    .padding(20)
    .width(Length::Fill);

    // Log output
    let log_header = container(
        row![
            text("OUTPUT").size(14).color(style::TEXT_MUTED),
            Space::with_width(Length::Fill),
            button(text("COPY").size(13).color(style::TEXT_SECONDARY))
                .on_press(Message::CopyLog)
                .style(iced::widget::button::secondary)
                .padding([6, 16]),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([12, 20]);

    let log_content: Element<'_, Message> = if log_lines.is_empty() {
        container(
            text("Waiting for output...").size(14).color(style::TEXT_MUTED),
        )
        .padding([0, 20])
        .into()
    } else {
        let log_col = Column::with_children(
            log_lines
                .iter()
                .rev()
                .take(500)
                .rev()
                .map(|line| {
                    let color = if line.contains("No core has thrown an error") {
                        style::SUCCESS
                    } else if line.contains("has thrown an error") {
                        // RED: only real errors - core failure
                        style::ERROR
                    } else if line.contains("WHEA error while") || line.contains("WHEA errors while") {
                        // YELLOW: only real WHEA errors during testing
                        style::WARNING
                    } else if line.contains("No new WHEA")
                        || line.contains("Checking for stress test errors")
                        || line.contains("Looking for new WHEA")
                        || line.contains("Stored WHEA Error Date")
                        || line.contains("Last WHEA Error Date")
                        || line.contains("passed")
                        || line.contains("completed")
                    {
                        // GREEN: diagnostic/info lines and success
                        style::SUCCESS
                    } else if line.contains("[STDERR]") {
                        // Dimmed for stderr - not necessarily an error
                        style::TEXT_MUTED
                    } else if line.contains("Iteration") && line.contains(" - ") {
                        style::ACCENT
                    } else if line.contains("Set to Core") {
                        style::TEXT_PRIMARY
                    } else {
                        style::TEXT_SECONDARY
                    };
                    text(line.as_str())
                        .size(13)
                        .color(color)
                        .into()
                })
                .collect::<Vec<Element<'_, Message>>>(),
        )
        .spacing(3);
        container(log_col).padding([0, 20]).into()
    };

    column![
        container(status_line).padding([12, 20]),
        container(Space::with_height(1)).width(Length::Fill).style(|_: &iced::Theme| {
            iced::widget::container::Style {
                background: Some(iced::Background::Color(style::BORDER)),
                ..Default::default()
            }
        }),
        core_grid,
        container(Space::with_height(1)).width(Length::Fill).style(|_: &iced::Theme| {
            iced::widget::container::Style {
                background: Some(iced::Background::Color(style::BORDER)),
                ..Default::default()
            }
        }),
        log_header,
        scrollable(
            container(log_content).width(Length::Fill)
        )
        .height(Length::Fill)
        .anchor_bottom(),
    ]
    .spacing(0)
    .into()
}

fn build_core_grid(cores: &[CoreStatus]) -> Element<'_, Message> {
    if cores.is_empty() {
        return text("Detecting cores...").size(14).color(style::TEXT_MUTED).into();
    }

    let cores_per_row = 8;
    let rows: Vec<Element<'_, Message>> = cores
        .chunks(cores_per_row)
        .map(|chunk| {
            let cells: Vec<Element<'_, Message>> = chunk
                .iter()
                .map(|core| {
                    let (bg_color, text_color, status_label) = match core.state {
                        CoreState::Idle => (style::CORE_IDLE, style::TEXT_MUTED, "IDLE"),
                        CoreState::Testing => (style::CORE_TESTING, style::CORE_TESTING_TEXT, "TESTING"),
                        CoreState::Passed => (style::CORE_PASSED, style::SUCCESS, "PASS"),
                        CoreState::Error => (style::CORE_ERROR, style::ERROR, "ERROR"),
                        CoreState::Skipped => (style::CORE_SKIPPED, style::TEXT_MUTED, "SKIP"),
                    };

                    let error_info: Element<'_, Message> = if core.error_count > 0 {
                        text(format!("{} err", core.error_count))
                            .size(11)
                            .color(style::ERROR)
                            .into()
                    } else {
                        Space::with_height(0).into()
                    };

                    container(
                        column![
                            text(format!("{}", core.id))
                                .size(22)
                                .color(text_color),
                            text(status_label)
                                .size(11)
                                .color(if core.state == CoreState::Testing {
                                    style::CORE_TESTING_TEXT
                                } else {
                                    style::TEXT_MUTED
                                }),
                            error_info,
                        ]
                        .align_x(iced::Alignment::Center)
                        .spacing(3),
                    )
                    .width(80)
                    .padding([12, 10])
                    .center_x(80)
                    .style(style::core_box_style(bg_color))
                    .into()
                })
                .collect();
            Row::with_children(cells).spacing(6).into()
        })
        .collect();

    Column::with_children(rows).spacing(6).into()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoreState {
    Idle,
    Testing,
    Passed,
    Error,
    Skipped,
}
