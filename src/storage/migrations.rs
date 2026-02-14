use std::path::PathBuf;

use serde_json::Value;

use crate::storage::StorageError;

type MigrationFn = fn(Value) -> Result<Value, StorageError>;

fn get_migrations() -> Vec<MigrationFn> {
    vec![
        // Future migrations will be added here
    ]
}

/// Returns 1 if version field is missing (assumes v1, our first versioned schema)
pub fn detect_version(content: &str) -> Result<u32, StorageError> {
    let value: Value = serde_json::from_str(content).map_err(|e| StorageError::ParseFailed {
        path: PathBuf::from("<unknown>"),
        source: e,
    })?;

    match value.get("version") {
        Some(v) => v.as_u64().map(|n| n as u32).ok_or_else(|| {
            // Create a dummy parse error since serde_json::Error doesn't have a simple constructor
            let dummy_err = serde_json::from_str::<Value>("invalid").unwrap_err();
            StorageError::ParseFailed {
                path: PathBuf::from("<unknown>"),
                source: dummy_err,
            }
        }),
        None => Ok(1), // No version field = v1
    }
}

/// Migrations are applied sequentially: v1→v2→v3→...→target
pub fn apply_migrations(
    mut data: Value,
    from_version: u32,
    to_version: u32,
) -> Result<Value, StorageError> {
    if from_version == to_version {
        return Ok(data);
    }

    if from_version > to_version {
        return Err(StorageError::FutureVersion(from_version));
    }

    let migrations = get_migrations();

    // Apply each migration in sequence
    for version in from_version..to_version {
        let migration_idx = (version - 1) as usize; // v1→v2 is at index 0

        if migration_idx >= migrations.len() {
            return Err(StorageError::UnsupportedVersion(version));
        }

        data = migrations[migration_idx](data)?;
    }

    Ok(data)
}

// ============================================================================
// EXAMPLE: How to write a migration when you need to create v2
// ============================================================================
//
// Step 1: Update CURRENT_VERSION in src/models/store.rs to 2
// Step 2: Add migrate_v1_to_v2 to get_migrations() vec above
// Step 3: Implement the migration function below
//
// #[allow(dead_code)]
// fn migrate_v1_to_v2(mut value: Value) -> Result<Value, StorageError> {
//     if let Some(obj) = value.as_object_mut() {
//         // Update version number
//         obj.insert("version".to_string(), Value::from(2));
//
//         // Apply your migration logic here
//         // Example: Add a new "priority" field to all tasks with default value 0
//         if let Some(tasks) = obj.get_mut("tasks").and_then(|t| t.as_array_mut()) {
//             for task in tasks {
//                 if let Some(task_obj) = task.as_object_mut() {
//                     task_obj.insert("priority".to_string(), Value::from(0));
//                 }
//             }
//         }
//     }
//
//     Ok(value)
// }
//
// Common migration patterns:
// - Adding field: obj.insert("new_field".to_string(), Value::from(default));
// - Renaming field: if let Some(v) = obj.remove("old") { obj.insert("new".to_string(), v); }
// - Type change: Match on old type, convert, insert new value
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_version_with_version_field() {
        let json = r#"{"version": 2, "tasks": [], "projects": [], "areas": []}"#;
        assert_eq!(detect_version(json).unwrap(), 2);
    }

    #[test]
    fn test_detect_version_without_version_field() {
        let json = r#"{"tasks": [], "projects": [], "areas": []}"#;
        assert_eq!(detect_version(json).unwrap(), 1);
    }

    #[test]
    fn test_apply_migrations_same_version() {
        let data = serde_json::json!({"version": 1});
        let result = apply_migrations(data.clone(), 1, 1).unwrap();
        assert_eq!(result, data);
    }

    #[test]
    fn test_apply_migrations_future_version() {
        let data = serde_json::json!({"version": 5});
        let result = apply_migrations(data, 5, 1);
        assert!(matches!(result, Err(StorageError::FutureVersion(5))));
    }
}
