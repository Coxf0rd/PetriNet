use super::*;

impl PetriApp {
    pub(in crate::ui::app) fn export_netstar_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Файлы NetStar (gpn)", &["gpn"])
            .set_file_name("экспорт_netstar.gpn")
            .save_file()
        {
            self.start_netstar_export_validation(path);
        }
    }
}
