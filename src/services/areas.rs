use crate::{
    models::{area::Area, store::Store},
    storage::{Storage, StorageError},
};
use slug::slugify;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CreateAreaError {
    #[error("Area with name '{}' already exists", .0)]
    AreaAlreadyExists(String),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

pub struct CreateAreaParameters {
    pub name: String,
}

pub fn create_area(
    store: &mut Store,
    storage: &impl Storage,
    parameters: CreateAreaParameters,
) -> Result<Area, CreateAreaError> {
    let area_slug = slugify(&parameters.name);

    let area = Area {
        name: parameters.name,
        slug: area_slug,
        ..Area::default()
    };

    let area_id = area.id;

    store.add_area(area);

    storage.save(store)?;

    Ok(store.get_area(area_id).unwrap().clone())
}

#[derive(Debug, Error)]
pub enum DeleteAreaError {
    #[error("Area with name '{}' not found", .0)]
    AreaNotFound(String),

    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}

pub struct DeleteAreaParameters {
    pub name: String,
}

pub fn delete_area(
    store: &mut Store,
    storage: &impl Storage,
    parameters: DeleteAreaParameters,
) -> Result<(), DeleteAreaError> {
    todo!()
}
