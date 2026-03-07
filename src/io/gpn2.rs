use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use crate::model::{PetriNetModel, GPN2_FORMAT_VERSION, GPN2_MAGIC};

pub fn save_gpn2(path: &Path, model: &PetriNetModel) -> Result<()> {
    let mut model = model.clone();
    model.format_version = GPN2_FORMAT_VERSION;
    model.validate()?;

    let json = serde_json::to_string(&model).context("Не удалось сериализовать GPN2")?;
    let mut bytes = Vec::with_capacity(GPN2_MAGIC.len() + json.len());
    bytes.extend_from_slice(GPN2_MAGIC.as_bytes());
    bytes.extend_from_slice(json.as_bytes());

    fs::write(path, bytes)
        .with_context(|| format!("Не удалось записать файл {}", path.display()))?;
    Ok(())
}

pub fn load_gpn2(path: &Path) -> Result<PetriNetModel> {
    let bytes =
        fs::read(path).with_context(|| format!("Не удалось прочитать файл {}", path.display()))?;
    load_gpn2_from_bytes(&bytes)
}

pub fn load_gpn2_from_bytes(bytes: &[u8]) -> Result<PetriNetModel> {
    if !bytes.starts_with(GPN2_MAGIC.as_bytes()) {
        return Err(anyhow!("Файл не содержит заголовок GPN2"));
    }

    let json_bytes = &bytes[GPN2_MAGIC.len()..];
    let value: serde_json::Value =
        serde_json::from_slice(json_bytes).context("Некорректный JSON в GPN2")?;

    let migrated = migrate_to_latest(value)?;
    let model: PetriNetModel =
        serde_json::from_value(migrated).context("JSON не соответствует схеме GPN2")?;
    model.validate()?;
    Ok(model)
}

fn migrate_to_latest(mut value: serde_json::Value) -> Result<serde_json::Value> {
    let Some(version) = value.get("format_version").and_then(|v| v.as_u64()) else {
        return Err(anyhow!("Отсутствует поле format_version"));
    };

    match version as u32 {
        GPN2_FORMAT_VERSION => Ok(value),
        1 => {
            // TODO: Миграция legacy JSON версии 1 -> GPN2 при необходимости.
            value["format_version"] = serde_json::Value::from(GPN2_FORMAT_VERSION);
            Ok(value)
        }
        other => Err(anyhow!("Неподдерживаемая версия формата: {}", other)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{NodeRef, PetriNetModel};

    #[test]
    fn roundtrip_save_load() {
        let tmp = tempfile::NamedTempFile::new().unwrap();
        let mut model = PetriNetModel::new();
        model.add_place([1.0, 2.0]);
        model.add_transition([3.0, 4.0]);
        let p_id = model.places[0].id;
        let t_id = model.transitions[0].id;
        model.add_arc(NodeRef::Place(p_id), NodeRef::Transition(t_id), 2);

        save_gpn2(tmp.path(), &model).unwrap();
        let loaded = load_gpn2(tmp.path()).unwrap();

        assert_eq!(model, loaded);
    }

    #[test]
    fn header_detection() {
        let bytes = b"NOPE{}";
        assert!(load_gpn2_from_bytes(bytes).is_err());
    }

    #[test]
    fn validation_failure_for_duplicate_ids() {
        let mut model = PetriNetModel::new();
        model.add_place([0.0, 0.0]);
        model.add_place([1.0, 1.0]);
        model.places[1].id = model.places[0].id;

        let json = serde_json::to_string_pretty(&model).unwrap();
        let mut bytes = Vec::new();
        bytes.extend_from_slice(GPN2_MAGIC.as_bytes());
        bytes.extend_from_slice(json.as_bytes());

        assert!(load_gpn2_from_bytes(&bytes).is_err());
    }
}
