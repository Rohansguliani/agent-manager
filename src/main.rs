// Agent Manager GUI - Main Entry Point
// Native Rust GUI application for managing CLI agents

mod state;
mod ui;

use eframe::egui;
use state::{AppState, Agent, AgentStatus};
use ui::{render_app_layout, TerminalOutput};

fn main() -> eframe::Result<()> {
    // Configure window options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Agent Manager GUI")
            .with_inner_size([1000.0, 700.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    // Run the application
    eframe::run_native(
        "Agent Manager GUI",
        options,
        Box::new(|_cc| {
            // Initialize the application with some test data for Phase 2
            let mut app = AgentManagerApp::new();
            app.initialize_test_data();
            Box::new(app)
        }),
    )
}

/// Main application struct
/// Manages application state and UI rendering
struct AgentManagerApp {
    /// Application state (agents, selection, UI preferences)
    state: AppState,
    /// Terminal output display
    terminal: TerminalOutput,
}

impl AgentManagerApp {
    /// Create a new application instance
    fn new() -> Self {
        Self {
            state: AppState::new(),
            terminal: TerminalOutput::new(500), // Keep last 500 lines
        }
    }

    /// Initialize test data for Phase 2 GUI testing
    /// This will be removed/replaced with actual agent loading in Phase 3
    fn initialize_test_data(&mut self) {
        // Add some test agents to demonstrate the UI
        self.state.add_agent(Agent::new(
            "agent-1".to_string(),
            "Gemini CLI Agent".to_string(),
        ));
        self.state.add_agent(Agent::new(
            "agent-2".to_string(),
            "Claude Code Agent".to_string(),
        ));
        self.state.add_agent(Agent::new(
            "agent-3".to_string(),
            "Development Helper".to_string(),
        ));

        // Set different statuses for testing
        self.state.update_agent_status(&"agent-1".to_string(), AgentStatus::Idle);
        self.state.update_agent_status(&"agent-2".to_string(), AgentStatus::Running);
        self.state.update_agent_status(&"agent-3".to_string(), AgentStatus::Stopped);

        // Add some test terminal output
        self.terminal.add_line("Agent Manager GUI initialized".to_string());
        self.terminal.add_line("Phase 2: GUI Foundation".to_string());
        self.terminal.add_line("Ready for agent management".to_string());
    }
}

impl eframe::App for AgentManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Render the main application layout
        render_app_layout(ctx, &mut self.state, &mut self.terminal);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_creation() {
        let app = AgentManagerApp::new();
        assert_eq!(app.state.agent_count(), 0);
    }

    #[test]
    fn test_app_initialization() {
        let mut app = AgentManagerApp::new();
        app.initialize_test_data();
        assert_eq!(app.state.agent_count(), 3);
    }
}
