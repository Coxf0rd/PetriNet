use super::*;

impl PetriApp {
    pub(super) fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let mut do_new = false;
        let mut do_open = false;
        let mut do_save = false;
        let mut do_exit = false;
        let mut do_delete = false;
        let mut do_copy = false;
        let mut do_paste = false;
        let mut do_undo = false;
        let mut do_select_all = false;
        let mut do_clear_selection = false;

        ctx.input(|i| {
            do_new = i.modifiers.command && i.key_pressed(egui::Key::N);
            do_open = i.modifiers.command && i.key_pressed(egui::Key::O);
            do_save = i.modifiers.command && i.key_pressed(egui::Key::S);
            do_exit = i.modifiers.command && i.key_pressed(egui::Key::Q);
            do_delete = i.key_pressed(egui::Key::Delete);
            // Strict shortcuts: only Ctrl+key where Ctrl is already held.
            do_copy = i.modifiers.ctrl && i.key_pressed(egui::Key::C);
            do_paste = i.modifiers.ctrl && i.key_pressed(egui::Key::V);
            do_undo = i.modifiers.ctrl && i.key_pressed(egui::Key::Z);
            do_select_all = i.modifiers.ctrl && i.key_pressed(egui::Key::A);
            do_clear_selection = i.key_pressed(egui::Key::Escape);

            // Layout fallback (RU keyboard), still requiring Ctrl held.
            for e in &i.events {
                match e {
                    egui::Event::Copy => do_copy = true,
                    egui::Event::Paste(_) => do_paste = true,
                    _ => {}
                }
                if let egui::Event::Key {
                    key,
                    physical_key,
                    pressed: true,
                    modifiers,
                    ..
                } = e
                {
                    if modifiers.ctrl
                        && (*key == egui::Key::C || *physical_key == Some(egui::Key::C))
                    {
                        do_copy = true;
                    }
                    if modifiers.ctrl
                        && (*key == egui::Key::V || *physical_key == Some(egui::Key::V))
                    {
                        do_paste = true;
                    }
                    if modifiers.ctrl
                        && (*key == egui::Key::Z || *physical_key == Some(egui::Key::Z))
                    {
                        do_undo = true;
                    }
                    if modifiers.ctrl
                        && (*key == egui::Key::A || *physical_key == Some(egui::Key::A))
                    {
                        do_select_all = true;
                    }
                    if *key == egui::Key::Escape || *physical_key == Some(egui::Key::Escape) {
                        do_clear_selection = true;
                    }
                }
                if let egui::Event::Text(text) = e {
                    if i.modifiers.ctrl {
                        if text.eq_ignore_ascii_case("c")
                            || matches!(text.as_str(), "СЃ" | "РЎ" | "Р РЋ" | "Р РЋР С“")
                        {
                            do_copy = true;
                        }
                        if text.eq_ignore_ascii_case("v")
                            || matches!(text.as_str(), "Рј" | "Рњ" | "Р С" | "Р Сљ")
                        {
                            do_paste = true;
                        }
                        if text.eq_ignore_ascii_case("z")
                            || matches!(text.as_str(), "СЏ" | "РЇ" | "Р Р‡")
                        {
                            do_undo = true;
                        }
                    }
                }
            }
            #[cfg(target_os = "windows")]
            {
                do_exit = do_exit || (i.modifiers.command && i.key_pressed(egui::Key::X));
            }
        });

        // Additional low-level key consumption to survive integrations where key_pressed/modifiers are flaky.
        ctx.input_mut(|i| {
            do_copy = do_copy || i.consume_key(egui::Modifiers::CTRL, egui::Key::C);
            do_paste = do_paste || i.consume_key(egui::Modifiers::CTRL, egui::Key::V);
            do_undo = do_undo || i.consume_key(egui::Modifiers::CTRL, egui::Key::Z);
            do_select_all = do_select_all || i.consume_key(egui::Modifiers::CTRL, egui::Key::A);
            do_clear_selection =
                do_clear_selection || i.consume_key(egui::Modifiers::NONE, egui::Key::Escape);
        });

        if do_new {
            self.new_file();
        }
        if do_open {
            self.open_file();
        }
        if do_save {
            self.save_file();
        }
        if do_exit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if do_delete {
            self.delete_selected();
        }
        if do_copy {
            self.copy_selected_objects();
        }
        if do_paste {
            self.paste_copied_objects();
        }
        if do_undo {
            self.undo_last_action();
        }
        if do_select_all {
            self.select_all_objects();
        }
        if do_clear_selection {
            self.clear_selection();
            self.canvas.arc_start = None;
        }
    }
}
