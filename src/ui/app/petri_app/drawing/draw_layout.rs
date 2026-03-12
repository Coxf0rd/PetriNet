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

                let available_width = ui.available_width();
                let min_table_width = 280.0;
                let max_table_width = (available_width - 240.0).max(min_table_width);

                egui::SidePanel::right("table_view_side_panel")
                    .resizable(true)
                    .default_width(self.table_panel_width)
                    .min_width(min_table_width)
                    .max_width(max_table_width)
                    .show_inside(ui, |ui| {
                        self.table_panel_width = ui.max_rect().width();
                        self.draw_table_workspace(ui);
                    });

                if self.show_graph_view {
                    self.draw_graph_view(ui);
                }
            }
            LayoutMode::Minimized => {}
        });
    }
}
