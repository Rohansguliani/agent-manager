// Reusable UI components
// Provides common UI elements for the application

use eframe::egui;
use crate::state::AgentStatus;

/// Render a status badge with colored text (no background bar)
/// Colors: Idle (gray), Running (green), Stopped (yellow), Error (red)
pub fn status_badge(ui: &mut egui::Ui, status: AgentStatus) {
    let (text, text_color) = match status {
        AgentStatus::Idle => ("Idle", egui::Color32::GRAY),
        AgentStatus::Running => ("Running", egui::Color32::from_rgb(0, 200, 0)), // Green
        AgentStatus::Stopped => ("Stopped", egui::Color32::from_rgb(220, 180, 0)), // Yellow
        AgentStatus::Error => ("Error", egui::Color32::from_rgb(220, 0, 0)), // Red
    };
    
    ui.colored_label(text_color, text);
}

/// Render a primary action button
#[allow(dead_code)] // Prepared for Phase 3 (Agent Management Core) - Dialogs and forms
pub fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.button(egui::RichText::new(text).strong())
}

/// Render a secondary button
#[allow(dead_code)] // Prepared for Phase 3 (Agent Management Core) - Dialogs and forms
pub fn secondary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    ui.button(text)
}

/// Render a start button (typically green)
pub fn start_button(ui: &mut egui::Ui) -> egui::Response {
    ui.button(egui::RichText::new("▶ Start").color(egui::Color32::from_rgb(0, 180, 0)))
}

/// Render a stop button (typically red)
pub fn stop_button(ui: &mut egui::Ui) -> egui::Response {
    ui.button(egui::RichText::new("⏹ Stop").color(egui::Color32::from_rgb(220, 0, 0)))
}

/// Terminal output display area
/// Provides a scrollable text area for displaying terminal output
pub struct TerminalOutput {
    /// Buffer of output lines
    lines: Vec<String>,
    /// Maximum number of lines to keep (0 = unlimited)
    max_lines: usize,
    /// Whether to auto-scroll to bottom
    auto_scroll: bool,
}

impl TerminalOutput {
    /// Create a new terminal output display
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: Vec::new(),
            max_lines,
            auto_scroll: true,
        }
    }

    /// Add a line to the output
    pub fn add_line(&mut self, line: String) {
        self.lines.push(line);
        if self.max_lines > 0 && self.lines.len() > self.max_lines {
            self.lines.remove(0);
        }
    }

    /// Clear all output
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Render the terminal output in a scrollable area
    pub fn render(&mut self, ui: &mut egui::Ui) {
        // Control bar
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_space(8.0);
            ui.label(egui::RichText::new("Terminal Output").heading());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);
                if ui.button("Clear").clicked() {
                    self.clear();
                }
                ui.add_space(8.0);
                ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
            });
        });
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);
        
        // Scrollable text area with monospace font
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.style_mut().override_text_style = Some(egui::TextStyle::Monospace);
                ui.spacing_mut().item_spacing = egui::vec2(4.0, 2.0);
                
                for line in &self.lines {
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        ui.label(
                            egui::RichText::new(line)
                                .size(12.0)
                                .family(egui::FontFamily::Monospace)
                        );
                    });
                }
                
                // Auto-scroll to bottom if enabled
                if self.auto_scroll && !self.lines.is_empty() {
                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                }
            });
    }
}

impl Default for TerminalOutput {
    fn default() -> Self {
        Self::new(1000) // Default to 1000 lines
    }
}

