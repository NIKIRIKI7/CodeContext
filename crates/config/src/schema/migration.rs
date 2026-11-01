use core::preset::models::resolution::ConfigWarning;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use super::dtos::CURRENT_SCHEMA_VERSION;

type MigrationStepFn = fn(&mut Map<String, Value>);

const MIGRATION_PIPELINE: &[(u32, MigrationStepFn)] = &[(0, migrate_v0_to_v1)];

pub(crate) fn parse_and_migrate_json<T>(
    raw_json: &str,
    warnings: &mut Vec<ConfigWarning>,
) -> Result<T, String>
where
    T: DeserializeOwned,
{
    if peek_schema_version_fast(raw_json) == Some(CURRENT_SCHEMA_VERSION)
        && let Ok(direct) = serde_json::from_str::<T>(raw_json)
    {
        return Ok(direct);
    }

    let mut value: Value = serde_json::from_str(raw_json).map_err(|err| err.to_string())?;
    migrate_json_value(&mut value, warnings)?;
    serde_json::from_value::<T>(value).map_err(|err| err.to_string())
}

pub(crate) fn migrate_json_value(
    value: &mut Value,
    warnings: &mut Vec<ConfigWarning>,
) -> Result<u32, String> {
    let Some(obj) = value.as_object_mut() else {
        return Err("Корневой элемент конфигурации должен быть JSON-объектом `{...}`".to_owned());
    };

    let mut file_version = match obj.get("schema_version") {
        None => 0,
        Some(v) => v
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .ok_or_else(|| "Поле `schema_version` должно быть целым числом >= 0".to_owned())?,
    };

    if file_version > CURRENT_SCHEMA_VERSION {
        warnings.push(ConfigWarning::NewerSchemaVersion {
            file_version,
            supported_version: CURRENT_SCHEMA_VERSION,
        });
        return Ok(file_version);
    }

    for &(from_version, step_fn) in MIGRATION_PIPELINE {
        if file_version == from_version {
            step_fn(obj);
            file_version = from_version + 1;
        }
    }

    Ok(CURRENT_SCHEMA_VERSION)
}

fn migrate_v0_to_v1(obj: &mut Map<String, Value>) {
    obj.insert("schema_version".to_owned(), Value::from(CURRENT_SCHEMA_VERSION));
    rename_key_if_target_missing(obj, "active_preset", "active_preset_name");
    upgrade_v0_preset_fields(obj);
    if let Some(Value::Array(presets)) = obj.get_mut("presets") {
        for item in presets {
            if let Some(preset_obj) = item.as_object_mut() {
                upgrade_v0_preset_fields(preset_obj);
            }
        }
    }
}

fn upgrade_v0_preset_fields(preset_obj: &mut Map<String, Value>) {
    rename_key_if_target_missing(preset_obj, "use_gitignore", "respect_gitignore");
    rename_key_if_target_missing(preset_obj, "minify", "minify_code");
    rename_key_if_target_missing(preset_obj, "max_file_size_bytes", "large_file_threshold_bytes");
}

fn rename_key_if_target_missing(obj: &mut Map<String, Value>, old_key: &str, new_key: &str) {
    if let Some(val) = obj.remove(old_key)
        && !obj.contains_key(new_key)
    {
        obj.insert(new_key.to_owned(), val);
    }
}

#[inline]
fn peek_schema_version_fast(raw_json: &str) -> Option<u32> {
    let prefix = raw_json.get(..512).unwrap_or(raw_json);
    let trimmed = prefix.trim_start();
    if !trimmed.starts_with('{') {
        return None;
    }
    let marker = "\"schema_version\"";
    let pos = trimmed.find(marker)?;
    let after_key = trimmed.get(pos + marker.len()..)?.trim_start();
    let after_colon = after_key.strip_prefix(':')?.trim_start();
    let digits_end = after_colon.find(|c: char| !c.is_ascii_digit()).unwrap_or(after_colon.len());
    let digits = after_colon.get(..digits_end)?;
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u32>().ok()
}

#[cfg(test)]
#[allow(clippy::assert_is_empty, clippy::indexing_slicing)]
mod tests {
    use serde_json::json;

    use super::{migrate_json_value, parse_and_migrate_json, peek_schema_version_fast};
    use crate::schema::dtos::{CURRENT_SCHEMA_VERSION, GlobalConfigDto};

    #[test]
    fn fast_schema_version_peek_works_without_allocations() {
        assert_eq!(
            peek_schema_version_fast("{\n  \"schema_version\": 1,\n  \"name\": \"Default\"\n}"),
            Some(1)
        );
        assert_eq!(peek_schema_version_fast("{\"name\": \"Legacy\"}"), None);
    }

    #[test]
    fn migrates_v0_to_v1_and_is_strictly_idempotent() {
        let mut legacy = json!({
            "active_preset": "Minimal",
            "presets": [
                {
                    "name": "Minimal",
                    "use_gitignore": false,
                    "minify": true,
                    "max_file_size_bytes": 524_288
                }
            ]
        });
        let mut warnings = Vec::new();
        let v1 = migrate_json_value(&mut legacy, &mut warnings).unwrap();
        assert_eq!(v1, CURRENT_SCHEMA_VERSION);
        assert!(warnings.is_empty());

        let snapshot_after_first = legacy.clone();
        let v2 = migrate_json_value(&mut legacy, &mut warnings).unwrap();
        assert_eq!(v2, CURRENT_SCHEMA_VERSION);
        assert_eq!(legacy, snapshot_after_first);

        let dto: GlobalConfigDto =
            parse_and_migrate_json(&legacy.to_string(), &mut warnings).unwrap();
        assert_eq!(dto.active_preset_name, "Minimal");
        assert!(!dto.presets[0].respect_gitignore);
        assert!(dto.presets[0].minify_code);
        assert_eq!(dto.presets[0].large_file_threshold_bytes, 524_288);
    }
}
