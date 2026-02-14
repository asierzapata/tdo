use std::{
    fs::{self, OpenOptions, rename, write},
    path::{Path, PathBuf},
};

use fs2::FileExt;
use serde_json::to_string_pretty;
use uuid::Uuid;

use crate::{
    models::store::Store,
    storage::{Storage, StorageError},
};

pub struct JsonFileStorage {
    path: PathBuf,
}

impl JsonFileStorage {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn create_backup_dir(&self) -> Result<(), StorageError> {
        let backups_dir = self.get_backup_dir();
        fs::create_dir(&backups_dir).map_err(|e| StorageError::BackupFailed {
            path: backups_dir,
            source: e,
        })?;
        Ok(())
    }

    fn create_backup(&self) -> Result<u64, StorageError> {
        let file_exists = fs::exists(&self.path).map_err(|e| StorageError::BackupFailed {
            path: self.path.clone(),
            source: e,
        })?;
        if !file_exists {
            return Ok(0);
        }

        let backup_path = self.get_backup_path();
        let copy_result = fs::copy(&self.path, &backup_path);
        match copy_result {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                self.create_backup_dir()?;
                self.create_backup()
            }
            Err(e) => Err(StorageError::BackupFailed {
                path: backup_path,
                source: e,
            }),
            Ok(bytes) => Ok(bytes),
        }
    }

    fn cleanup_old_backups(&self) -> Result<(), StorageError> {
        let backup_dir = self.get_backup_dir();
        let backup_dir_exists =
            fs::exists(&backup_dir).map_err(|e| StorageError::CleanupFailed {
                dir: backup_dir.clone(),
                source: e,
            })?;
        if !backup_dir_exists {
            return Ok(());
        }

        let mut file_entries = fs::read_dir(&backup_dir)
            .map_err(|e| StorageError::CleanupFailed {
                dir: backup_dir.clone(),
                source: e,
            })?
            .flatten()
            .filter(|entry| entry.metadata().map(|m| m.is_file()).unwrap_or(false))
            .map(|entry| entry.path())
            .collect::<Vec<_>>();

        file_entries.sort();

        let number_of_files_to_delete = match file_entries.len() {
            x if x > 5 => x - 5,
            _ => 0,
        };

        if number_of_files_to_delete == 0 {
            return Ok(());
        }

        for file_path in &file_entries[0..number_of_files_to_delete] {
            fs::remove_file(file_path).map_err(|e| StorageError::CleanupFailed {
                dir: backup_dir.clone(),
                source: e,
            })?;
        }

        Ok(())
    }

    fn get_backup_dir(&self) -> PathBuf {
        let parent_store_path = self.path.parent().unwrap_or(Path::new("."));
        parent_store_path.join("backups")
    }

    fn get_backup_path(&self) -> PathBuf {
        let backups_dir = self.get_backup_dir();

        let timestamp = jiff::Timestamp::now().to_string();
        let filename = format!("{:?}-{}", self.path.file_name(), timestamp);

        backups_dir.join(filename)
    }
}

impl Storage for JsonFileStorage {
    fn load(&self) -> Result<Store, StorageError> {
        use crate::models::store::CURRENT_VERSION;
        use crate::storage::migrations::{apply_migrations, detect_version};

        match std::fs::read_to_string(&self.path) {
            Ok(content) => {
                let file_version = detect_version(&content)?;

                if file_version > CURRENT_VERSION {
                    return Err(StorageError::FutureVersion(file_version));
                }

                let mut data: serde_json::Value =
                    serde_json::from_str(&content).map_err(|e| StorageError::ParseFailed {
                        path: self.path.clone(),
                        source: e,
                    })?;

                if file_version < CURRENT_VERSION {
                    data = apply_migrations(data, file_version, CURRENT_VERSION)?;
                }

                if let Some(obj) = data.as_object_mut() {
                    obj.insert("version".to_string(), serde_json::json!(CURRENT_VERSION));
                }

                let store: Store =
                    serde_json::from_value(data).map_err(|e| StorageError::ParseFailed {
                        path: self.path.clone(),
                        source: e,
                    })?;
                Ok(store)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Store::default()),
            Err(e) => Err(StorageError::LoadFailed {
                path: self.path.clone(),
                source: e,
            }),
        }
    }

    fn save(&self, store: &Store) -> Result<(), StorageError> {
        let json =
            to_string_pretty(store).map_err(|e| StorageError::SerializeFailed { source: e })?;

        let unique_temp = format!("{}.tmp.{}", self.path.display(), Uuid::new_v4());
        let temp_path = PathBuf::from(&unique_temp);
        write(&temp_path, json).map_err(|e| StorageError::SaveFailed {
            path: temp_path.clone(),
            source: e,
        })?;

        let lock_file_path = self.path.with_extension("lock");
        let lock_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open(&lock_file_path)
            .map_err(|e| StorageError::SaveFailed {
                path: lock_file_path.clone(),
                source: e,
            })?;
        lock_file
            .lock_exclusive()
            .map_err(|e| StorageError::SaveFailed {
                path: lock_file_path,
                source: e,
            })?;

        self.create_backup()?;
        self.cleanup_old_backups()?;

        rename(&temp_path, &self.path).map_err(|e| StorageError::SaveFailed {
            path: self.path.clone(),
            source: e,
        })?;

        lock_file.unlock().map_err(|e| StorageError::SaveFailed {
            path: self.path.clone(),
            source: e,
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    use crate::{
        models::{area::Area, project::Project, store::Store, task::Task},
        storage::json::JsonFileStorage,
    };

    #[test]
    fn test_save_and_load() {
        let area = Area {
            name: String::from("Some Area"),
            ..Area::default()
        };
        let project = Project {
            area_id: Some(area.id),
            name: String::from("Some Project"),
            ..Project::default()
        };
        let task = Task {
            title: String::from("Some Task"),
            project_id: Some(project.id),
            ..Task::default()
        };
        let store = Store {
            version: 1,
            areas: Vec::from([area]),
            projects: Vec::from([project]),
            tasks: Vec::from([task]),
        };
        let json_file_storage = JsonFileStorage {
            path: PathBuf::from("/tmp/test_store.json"),
        };
        if let Err(_) = json_file_storage.save(&store) {
            panic!("Should correctly save the store");
        }
        match json_file_storage.load() {
            Ok(loaded_store) => {
                assert_eq!(loaded_store.areas[0].id, store.areas[0].id);
                assert_eq!(loaded_store.projects[0].id, store.projects[0].id);
                assert_eq!(loaded_store.tasks[0].id, store.tasks[0].id);
            }
            Err(_) => panic!("Should correctly load the saved store"),
        }
    }

    #[test]
    fn test_load_invalid_json() {
        let path = PathBuf::from("/tmp/invalid_store.json");

        std::fs::write(&path, "{ this is not valid json }").unwrap();

        let storage = JsonFileStorage::new(path);
        let result = storage.load();

        match result {
            Err(StorageError::ParseFailed { .. }) => {}
            _ => panic!("Expected ParseFailed error, got something else"),
        }
    }

    #[test]
    fn test_load_v1_without_version_field() {
        let path = PathBuf::from("/tmp/v1_store.json");
        let old_json = r#"{
            "tasks": [],
            "projects": [],
            "areas": []
        }"#;

        std::fs::write(&path, old_json).unwrap();

        let storage = JsonFileStorage::new(path);
        let result = storage.load();

        match result {
            Ok(store) => {
                assert_eq!(store.version, crate::models::store::CURRENT_VERSION);
            }
            Err(e) => panic!("Expected successful load, got error: {:?}", e),
        }
    }

    #[test]
    fn test_load_future_version() {
        let path = PathBuf::from("/tmp/future_store.json");
        let future_json = r#"{
            "version": 999,
            "tasks": [],
            "projects": [],
            "areas": []
        }"#;

        std::fs::write(&path, future_json).unwrap();

        let storage = JsonFileStorage::new(path);
        let result = storage.load();

        match result {
            Err(StorageError::FutureVersion(999)) => {
                // Expected: should fail with FutureVersion error
            }
            _ => panic!("Expected FutureVersion(999) error"),
        }
    }

    #[test]
    fn test_backup_creation_and_cleanup() {
        let test_dir = PathBuf::from("/tmp/tdo_backup_test");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let store_path = test_dir.join("store.json");
        let storage = JsonFileStorage::new(store_path.clone());

        for i in 1..=7 {
            let mut store = Store::default();
            store.version = i;

            storage.save(&store).unwrap();

            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        let backups_dir = test_dir.join("backups");
        let backup_count = fs::read_dir(&backups_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.metadata().map(|m| m.is_file()).unwrap_or(false))
            .count();

        assert_eq!(backup_count, 5, "Should keep exactly 5 backups");

        fs::remove_dir_all(&test_dir).unwrap();
    }

    #[test]
    fn test_backup_directory_created_on_second_save() {
        let test_dir = PathBuf::from("/tmp/tdo_backup_dir_test");
        let _ = fs::remove_dir_all(&test_dir);
        fs::create_dir_all(&test_dir).unwrap();

        let store_path = test_dir.join("store.json");
        let storage = JsonFileStorage::new(store_path.clone());

        let backups_dir = test_dir.join("backups");
        assert!(!backups_dir.exists(), "Backups dir should not exist yet");

        let store = Store::default();
        storage.save(&store).unwrap();

        assert!(
            !backups_dir.exists(),
            "Backups dir should not exist after first save"
        );

        let mut store2 = Store::default();
        store2.version = 2;
        storage.save(&store2).unwrap();

        assert!(
            backups_dir.exists(),
            "Backups dir should be created on second save"
        );
        assert!(backups_dir.is_dir(), "Backups path should be a directory");

        fs::remove_dir_all(&test_dir).unwrap();
    }
}
