use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::model::{PetriNetModel, GPN2_MAGIC};

pub mod gpn2;
pub mod legacy_gpn;

pub use legacy_gpn::{LegacyDebugInfo, LegacyImportError, LegacyImportResult};

#[derive(Debug, Clone)]
pub struct LoadGpnResult {
    pub model: PetriNetModel,
    pub warnings: Vec<String>,
    pub legacy_debug: Option<LegacyDebugInfo>,
}

pub fn load_gpn(path: &Path) -> Result<LoadGpnResult> {
    let bytes =
        fs::read(path).with_context(|| format!("Не удалось прочитать файл {}", path.display()))?;

    if bytes.starts_with(GPN2_MAGIC.as_bytes()) {
        let model = gpn2::load_gpn2_from_bytes(&bytes)?;
        Ok(LoadGpnResult {
            model,
            warnings: Vec::new(),
            legacy_debug: None,
        })
    } else {
        if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if value.is_object() {
                if let Ok(model) = serde_json::from_value::<PetriNetModel>(value.clone()) {
                    model.validate()?;
                    return Ok(LoadGpnResult {
                        model,
                        warnings: vec!["Файл JSON открыт без заголовка GPN2".to_string()],
                        legacy_debug: None,
                    });
                }
            }
        }

        let legacy = legacy_gpn::import_legacy_gpn(path)?;
        Ok(LoadGpnResult {
            model: legacy.model,
            warnings: legacy.warnings,
            legacy_debug: Some(legacy.debug),
        })
    }
}

pub fn save_gpn(path: &Path, model: &PetriNetModel) -> Result<()> {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("gpn2"))
        .unwrap_or(false)
    {
        gpn2::save_gpn2(path, model)
    } else {
        legacy_gpn::export_legacy_gpn(path, model)
            .with_context(|| format!("Не удалось сохранить legacy GPN в {}", path.display()))
    }
}
