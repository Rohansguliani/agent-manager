// Main application layout
// Handles window layout, panels, menu bar, and overall UI structure

use eframe::egui;
use crate::state::{AppState, Agent};
use crate::ui::components::*;

/// Render the main application layout
/// Includes menu bar, sidebar, main content area, and terminal output
pub fn render_app_layout(ctx: &egui::Context, state: &mut AppState, terminal: &mut TerminalOutput) {
    // Menu bar at the top
    render_menu_bar(ctx);

    // Main layout with sidebar and content
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.horizontal(|ui| {
            // Left sidebar for agent list
            if state.ui_state.sidebar_visible {
                render_sidebar(ui, state);
            }

            // Main content area
            ui.vertical(|ui| {
                ui.add_space(8.0);
                render_main_content(ui, state);
                
                // Terminal output at the bottom (if visible)
                if state.ui_state.terminal_visible {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                    terminal.render(ui);
                    ui.add_space(8.0);
                }
            });
        });
    });
}

/// Render the top menu bar
fn render_menu_bar(ctx: &egui::Context) {
    egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            // File menu
            ui.menu_button("File", |ui| {
                if ui.button("New Agent").clicked() {
                    // TODO: Phase 3 - Open new agent dialog
                    ui.close_menu();
                }
                if ui.button("Open...").clicked() {
                    // TODO: Phase 3 - Open agent configuration
                    ui.close_menu();
                }
                if ui.button("Save").clicked() {
                    // TODO: Phase 3 - Save agent configuration
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("Quit").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            // Edit menu
            ui.menu_button("Edit", |ui| {
                if ui.button("Preferences").clicked() {
                    // TODO: Phase 9 - Open preferences
                    ui.close_menu();
                }
            });

            // View menu
            ui.menu_button("View", |ui| {
                let mut dark_mode = ctx.style().visuals.dark_mode;
                if ui.checkbox(&mut dark_mode, "Dark Mode").changed() {
                    ctx.style_mut(|style| {
                        style.visuals.dark_mode = dark_mode;
                    });
                }
                ui.separator();
                // UI state checkboxes will be handled by AppState in future
            });

            // Help menu
            ui.menu_button("Help", |ui| {
                if ui.button("About").clicked() {
                    // TODO: Phase 9 - Show about dialog
                    ui.close_menu();
                }
            });
        });
    });
}

/// Render the left sidebar with agent list
fn render_sidebar(ui: &mut egui::Ui, state: &mut AppState) {
    egui::SidePanel::left("agent_sidebar")
        .resizable(true)
        .default_width(250.0)
        .min_width(150.0)
        .show_inside(ui, |ui| {
            // Use vertical layout to structure header and scrollable content
            ui.vertical(|ui| {
                // Header section - fixed height
                ui.add_space(8.0);
                ui.heading("Agents");
                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
                
                // Agent list - ScrollArea fills ALL remaining vertical space in sidebar
                // Use id to ensure consistent sizing and fill remaining height
                egui::ScrollArea::vertical()
                    .id_source("agent_list_scroll")
                    .auto_shrink([false; 2])
                    .show(ui, |ui| {
                        // Collect agents and their IDs first to avoid borrowing issues
                        let agents: Vec<_> = state.agents_list().iter().map(|a| (a.id.clone(), a.name.clone(), a.status)).collect();
                        let selected_id = state.selected_agent_id.clone();
                        
                        if agents.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(40.0);
                                ui.label(egui::RichText::new("No agents").italics().weak().size(14.0));
                                ui.add_space(8.0);
                                ui.label(egui::RichText::new("Create one from File > New Agent").weak().size(12.0));
                            });
                        } else {
                            for (id, name, status) in agents {
                                let is_selected = selected_id.as_ref() == Some(&id);
                                
                                let row_id = ui.id().with(format!("agent_row_{}", id));
                                
                                // Use Frame for background (draws behind content)
                                // Set fill based on selection (known) - hover checked after render
                                let mut frame = egui::Frame::none();
                                frame.rounding = egui::Rounding::same(4.0);
                                if is_selected {
                                    frame.fill = ui.visuals().selection.bg_fill;
                                }
                                
                                // Render row with Frame
                                let row_response = frame.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(8.0);
                                        ui.label(&name);
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            ui.add_space(8.0);
                                            status_badge(ui, status);
                                        });
                                    })
                                });
                                
                                let row_rect = row_response.response.rect;
                                
                                // Check hover and click on entire row
                                let interact = ui.interact(row_rect, row_id, egui::Sense::click());
                                let hovered = interact.hovered();
                                let clicked = interact.clicked();
                                
                                // Draw hover outline (transparent, doesn't cover text)
                                if hovered && !is_selected {
                                    // Use a subtle outline stroke instead of filled rectangle
                                    // Create a semi-transparent color for the outline
                                    let stroke_color = ui.visuals().widgets.hovered.bg_fill;
                                    let stroke_color_alpha = egui::Color32::from_rgba_unmultiplied(
                                        stroke_color.r(),
                                        stroke_color.g(),
                                        stroke_color.b(),
                                        100, // Semi-transparent alpha
                                    );
                                    ui.painter().rect_stroke(
                                        row_rect,
                                        egui::Rounding::same(4.0),
                                        egui::Stroke::new(2.0, stroke_color_alpha),
                                    );
                                }
                                
                                // Handle click - toggle selection
                                if clicked {
                                    if is_selected {
                                        state.deselect_agent();
                                    } else {
                                        state.select_agent(&id);
                                    }
                                }
                                
                                ui.add_space(4.0);
                            }
                        }
                    });
            });
        });
}

/// Render the main content area
fn render_main_content(ui: &mut egui::Ui, state: &mut AppState) {
    ui.vertical(|ui| {
        ui.add_space(12.0);
        
        // Header
        ui.heading("Agent Details");
        ui.add_space(8.0);
        ui.separator();
        ui.add_space(12.0);
        
        // Show selected agent details or welcome message
        // Clone agent data to avoid borrowing issues
        if let Some(agent_id) = &state.selected_agent_id {
            if let Some(agent) = state.agents.get(agent_id).cloned() {
                render_agent_details(ui, state, &agent);
            } else {
                render_welcome_view(ui);
            }
        } else {
            render_welcome_view(ui);
        }
    });
}

/// Render welcome view when no agent is selected
fn render_welcome_view(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(80.0);
        ui.heading(egui::RichText::new("Welcome to Agent Manager").size(24.0));
        ui.add_space(24.0);
        ui.label(
            egui::RichText::new("Select an agent from the sidebar to view details")
                .size(14.0)
        );
        ui.add_space(8.0);
        ui.label(
            egui::RichText::new("or create a new agent from File > New Agent")
                .size(14.0)
        );
        ui.add_space(40.0);
        
        // Placeholder for future: quick actions, recent agents, etc.
        ui.label(
            egui::RichText::new("Phase 2: GUI Foundation")
                .weak()
                .small()
                .italics()
        );
    });
}

/// Render agent details view
fn render_agent_details(ui: &mut egui::Ui, _state: &mut AppState, agent: &Agent) {
    // Card-style container with better styling
    ui.group(|ui| {
        ui.vertical(|ui| {
            ui.add_space(16.0);
            
            // Agent name header with status badge
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.heading(egui::RichText::new(&agent.name).size(20.0));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(16.0);
                    status_badge(ui, agent.status);
                });
            });
            
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(16.0);
            
            // Agent information section
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    // Agent ID
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("ID:").strong());
                        ui.add_space(12.0);
                        ui.label(
                            egui::RichText::new(&agent.id)
                                .monospace()
                                .weak()
                                .size(13.0)
                        );
                    });
                    
                    ui.add_space(12.0);
                    
                    // Status
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Status:").strong());
                        ui.add_space(12.0);
                        status_badge(ui, agent.status);
                    });
                });
                ui.add_space(16.0);
            });
            
            ui.add_space(24.0);
            ui.separator();
            ui.add_space(16.0);
            
            // Action buttons section
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.spacing_mut().button_padding = egui::vec2(12.0, 8.0);
                if start_button(ui).clicked() {
                    // TODO: Phase 4 - Start agent process
                }
                ui.add_space(8.0);
                if stop_button(ui).clicked() {
                    // TODO: Phase 4 - Stop agent process
                }
            });
            
            ui.add_space(24.0);
            ui.separator();
            ui.add_space(16.0);
            
            // Configuration section
            ui.horizontal(|ui| {
                ui.add_space(16.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Configuration").heading().size(16.0));
                    ui.add_space(8.0);
                    ui.label(
                        egui::RichText::new("Agent configuration will be available in Phase 3")
                            .weak()
                            .size(13.0)
                    );
                });
                ui.add_space(16.0);
            });
            
            ui.add_space(16.0);
        });
    });
}

