use std::{fs, path::Path};

use super::{
    ProblemDataError,
    model::{ProblemCatalog, RoomFileInput},
    validation::{validate_catalog, validate_room_file},
};

pub fn load_problem_data(root: impl AsRef<Path>) -> Result<ProblemCatalog, ProblemDataError> {
    let rooms_dir = root.as_ref().join("rooms");

    let entries = fs::read_dir(&rooms_dir).map_err(|source| ProblemDataError::Io {
        path: rooms_dir.clone(),
        source,
    })?;

    let mut room_directories = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|source| ProblemDataError::Io {
            path: rooms_dir.clone(),
            source,
        })?;

        let path = entry.path();

        if path.is_dir() {
            room_directories.push(path);
        }
    }

    room_directories.sort();

    let mut rooms = Vec::new();

    for room_directory in room_directories {
        let file_path = room_directory.join("room.json");

        let json = fs::read_to_string(&file_path).map_err(|source| ProblemDataError::Io {
            path: file_path.clone(),
            source,
        })?;

        let input: RoomFileInput =
            serde_json::from_str(&json).map_err(|source| ProblemDataError::Json {
                path: file_path.clone(),
                source,
            })?;

        rooms.push(validate_room_file(input, &room_directory)?);
    }

    let catalog = ProblemCatalog { rooms };
    validate_catalog(&catalog)?;

    Ok(catalog)
}
