use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde_yaml::Value;

const OPENAPI_VERSION: &str = "3.1.0";
const REQUIRED_OPERATIONS: [(&str, &str, &str); 11] = [
    ("/openapi.yaml", "get", "getOpenApi"),
    ("/api/me", "get", "getMe"),
    ("/api/auth/guest", "post", "loginGuest"),
    ("/api/auth/logout", "post", "logoutDemo"),
    ("/api/rooms/{room_id}/runs", "post", "startOrResumeRun"),
    ("/api/rooms/{room_id}/runs/current", "get", "getCurrentRun"),
    ("/api/rooms/{room_id}/problems", "post", "createProblem"),
    (
        "/api/rooms/{room_id}/problems/{problem_id}",
        "get",
        "getProblem",
    ),
    (
        "/api/rooms/{room_id}/problems/{problem_id}/assets",
        "get",
        "getProblemAssets",
    ),
    (
        "/api/rooms/{room_id}/problems/{problem_id}/queries",
        "post",
        "submitQuery",
    ),
    (
        "/api/rooms/{room_id}/problems/{problem_id}/answers",
        "post",
        "submitAnswer",
    ),
];

fn main() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let source = manifest_dir.join("../openapi/openapi-v1.yaml");
    println!("cargo::rerun-if-changed={}", source.display());

    let contents = fs::read_to_string(&source).map_err(|error| {
        format!(
            "failed to read shared OpenAPI document {}: {error}",
            source.display()
        )
    })?;
    validate_openapi(&contents, &source)?;

    let output = PathBuf::from(env::var("OUT_DIR")?).join("openapi-v1.yaml");
    fs::write(&output, contents).map_err(|error| {
        format!(
            "failed to copy OpenAPI document to {}: {error}",
            output.display()
        )
    })?;

    Ok(())
}

fn validate_openapi(contents: &str, source: &Path) -> Result<(), Box<dyn Error>> {
    let document: Value = serde_yaml::from_str(contents)
        .map_err(|error| format!("{} is not valid YAML: {error}", source.display()))?;

    let version = document
        .get("openapi")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{} has no string `openapi` version", source.display()))?;
    if version != OPENAPI_VERSION {
        return Err(format!(
            "{} uses OpenAPI {version}; expected {OPENAPI_VERSION}",
            source.display()
        )
        .into());
    }

    let paths = document
        .get("paths")
        .ok_or_else(|| format!("{} has no `paths` object", source.display()))?;

    for (path, method, expected_operation_id) in REQUIRED_OPERATIONS {
        let operation = paths
            .get(path)
            .and_then(|path_item| path_item.get(method))
            .ok_or_else(|| {
                format!(
                    "{} is missing required operation {method} {path}",
                    source.display()
                )
            })?;
        let operation_id = operation
            .get("operationId")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                format!(
                    "{} operation {method} {path} has no string `operationId`",
                    source.display()
                )
            })?;
        if operation_id != expected_operation_id {
            return Err(format!(
                "{} operation {method} {path} uses operationId `{operation_id}`; expected `{expected_operation_id}`",
                source.display()
            )
            .into());
        }
    }

    Ok(())
}
