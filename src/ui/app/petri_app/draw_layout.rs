use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn draw_layout(&mut self, ctx: &egui::Context) {
        if self.show_table_view && self.table_fullscreen {
            egui::CentralPanel::default().show(ctx, |ui| {
                self.draw_table_workspace(ui);
            });
            return;
        }

        if self.layout_mode == LayoutMode::Minimized {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.heading("Все окна свернуты");
            });
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| match self.layout_mode {
            LayoutMode::Cascade => {
                if self.show_graph_view {
                    self.draw_graph_view(ui);
                }
                if self.show_table_view {
                    self.draw_table_workspace(ui);
                }
            }
            LayoutMode::TileHorizontal => {
                if !self.show_table_view {
                    if self.show_graph_view {
                        self.draw_graph_view(ui);
                    }
                    return;
                }
                ui.vertical(|ui| {
                    if self.show_graph_view {
                        ui.allocate_ui_with_layout(
                            Vec2::new(ui.available_width(), ui.available_height() * 0.55),
                            egui::Layout::top_down(egui::Align::LEFT),
                            |ui| self.draw_graph_view(ui),
                        );
                    }
                    ui.separator();
                    self.draw_table_workspace(ui);
                });
            }
            LayoutMode::TileVertical => {
                if !self.show_table_view {
                    if self.show_graph_view {
                        self.draw_graph_view(ui);
                    }
                    return;
                }
                ui.columns(2, |columns| {
                    if self.show_graph_view {
                        self.draw_graph_view(&mut columns[0]);
                    }
                    self.draw_table_workspace(&mut columns[1]);
                });
            }
            LayoutMode::Minimized => {}
        });
    }
}
