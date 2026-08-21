use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use uuid::Uuid;

use super::{
    ProblemDataError,
    judge::{normalize_answer, normalize_operations},
    model::{
        Asset, Candidate, JudgeConfig, JudgeConfigInput, Operation, Problem, ProblemCatalog,
        ProblemInput, ProblemType, Room, RoomFileInput, StringNormalization, SubmissionType,
    },
};

pub(super) fn validate_room_file(
    input: RoomFileInput,
    room_directory: &Path,
) -> Result<Room, ProblemDataError> {
    let room_id = parse_uuid(&input.room.room_id, "room.room_id")?;

    let directory_name = room_directory
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| validation_error("room", "room directory name must be valid UTF-8"))?;

    if directory_name != room_id.to_string() {
        return Err(validation_error(
            "room.room_id",
            "room ID must match its directory name",
        ));
    }

    if input.room.number <= 0 {
        return Err(validation_error(
            "room.number",
            "room number must be greater than zero",
        ));
    }

    require_non_empty(&input.room.name, "room.name")?;
    require_non_empty(&input.room.genre, "room.genre")?;
    require_non_empty(&input.room.description, "room.description")?;

    let problems = input
        .problems
        .into_iter()
        .enumerate()
        .map(|(index, problem)| validate_problem(problem, room_id, index, room_directory))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Room {
        room_id,
        number: input.room.number,
        name: input.room.name,
        genre: input.room.genre,
        description: input.room.description,
        problems,
    })
}

pub(super) fn validate_catalog(catalog: &ProblemCatalog) -> Result<(), ProblemDataError> {
    if catalog.rooms.is_empty() {
        return Err(validation_error("rooms", "at least one room is required"));
    }

    let mut room_ids = HashSet::new();
    let mut room_numbers = HashSet::new();
    let mut problem_ids = HashSet::new();
    let mut problem_rooms = HashMap::new();

    for (room_index, room) in catalog.rooms.iter().enumerate() {
        if !room_ids.insert(room.room_id) {
            return Err(validation_error(
                format!("rooms[{room_index}].room_id"),
                "room ID must not be duplicated",
            ));
        }

        if !room_numbers.insert(room.number) {
            return Err(validation_error(
                format!("rooms[{room_index}].number"),
                "room number must not be duplicated",
            ));
        }

        let small_count = room
            .problems
            .iter()
            .filter(|problem| problem.problem_type == ProblemType::Small)
            .count();

        let final_count = room
            .problems
            .iter()
            .filter(|problem| problem.problem_type == ProblemType::Final)
            .count();

        if small_count != 3 || final_count != 1 {
            return Err(validation_error(
                format!("rooms[{room_index}].problems"),
                "each room must contain three small problems and one final problem",
            ));
        }

        let mut problem_numbers = HashSet::new();

        for (problem_index, problem) in room.problems.iter().enumerate() {
            if !problem_ids.insert(problem.problem_id) {
                return Err(validation_error(
                    format!("rooms[{room_index}].problems[{problem_index}].problem_id"),
                    "problem ID must not be duplicated",
                ));
            }

            if !problem_numbers.insert(problem.number) {
                return Err(validation_error(
                    format!("rooms[{room_index}].problems[{problem_index}].number"),
                    "problem number must not be duplicated within a room",
                ));
            }

            if !problem.is_required {
                return Err(validation_error(
                    format!("rooms[{room_index}].problems[{problem_index}].is_required"),
                    "the required four-problem set must use is_required true",
                ));
            }

            problem_rooms.insert(problem.problem_id, room.room_id);
        }
    }

    validate_dependency_references(catalog, &problem_rooms)?;

    for (room_index, room) in catalog.rooms.iter().enumerate() {
        validate_dependency_cycles(room, room_index)?;
        validate_unlock_order(room, room_index)?;
    }

    Ok(())
}

fn validate_dependency_references(
    catalog: &ProblemCatalog,
    problem_rooms: &HashMap<Uuid, Uuid>,
) -> Result<(), ProblemDataError> {
    for (room_index, room) in catalog.rooms.iter().enumerate() {
        for (problem_index, problem) in room.problems.iter().enumerate() {
            let Some(dependency_id) = problem.depends_on_problem_id else {
                continue;
            };

            let field =
                format!("rooms[{room_index}].problems[{problem_index}].depends_on_problem_id");

            if dependency_id == problem.problem_id {
                return Err(validation_error(field, "problem must not depend on itself"));
            }

            let Some(dependency_room_id) = problem_rooms.get(&dependency_id) else {
                return Err(validation_error(
                    field,
                    "dependency must reference an existing problem",
                ));
            };

            if *dependency_room_id != room.room_id {
                return Err(validation_error(
                    field,
                    "dependency must reference a problem in the same room",
                ));
            }
        }
    }

    Ok(())
}

fn validate_dependency_cycles(room: &Room, room_index: usize) -> Result<(), ProblemDataError> {
    let dependencies = room
        .problems
        .iter()
        .map(|problem| (problem.problem_id, problem.depends_on_problem_id))
        .collect::<HashMap<_, _>>();

    for (problem_index, problem) in room.problems.iter().enumerate() {
        let mut visited = HashSet::new();
        let mut current_problem_id = Some(problem.problem_id);

        while let Some(problem_id) = current_problem_id {
            if !visited.insert(problem_id) {
                return Err(validation_error(
                    format!("rooms[{room_index}].problems[{problem_index}].depends_on_problem_id"),
                    "problem dependencies must not contain a cycle",
                ));
            }

            current_problem_id = dependencies.get(&problem_id).copied().flatten();
        }
    }

    Ok(())
}

fn validate_unlock_order(room: &Room, room_index: usize) -> Result<(), ProblemDataError> {
    let mut small_problems = room
        .problems
        .iter()
        .enumerate()
        .filter(|(_, problem)| problem.problem_type == ProblemType::Small)
        .collect::<Vec<_>>();

    small_problems.sort_by_key(|(_, problem)| problem.number);

    for (small_index, (problem_index, problem)) in small_problems.iter().enumerate() {
        let expected_dependency = small_index
            .checked_sub(1)
            .map(|previous_index| small_problems[previous_index].1.problem_id);

        if problem.depends_on_problem_id != expected_dependency {
            return Err(validation_error(
                format!("rooms[{room_index}].problems[{problem_index}].depends_on_problem_id"),
                "small problems must unlock in ascending number order",
            ));
        }
    }

    for (problem_index, problem) in room.problems.iter().enumerate() {
        if problem.problem_type == ProblemType::Final && problem.depends_on_problem_id.is_some() {
            return Err(validation_error(
                format!("rooms[{room_index}].problems[{problem_index}].depends_on_problem_id"),
                "final problem must not have a dependency",
            ));
        }
    }

    Ok(())
}

fn validation_error(field: impl Into<String>, message: &'static str) -> ProblemDataError {
    ProblemDataError::Validation {
        field: field.into(),
        message,
    }
}

fn parse_uuid(value: &str, field: impl Into<String>) -> Result<Uuid, ProblemDataError> {
    Uuid::parse_str(value).map_err(|_| validation_error(field, "must be a valid UUID"))
}

fn require_non_empty(value: &str, field: impl Into<String>) -> Result<(), ProblemDataError> {
    if value.trim().is_empty() {
        return Err(validation_error(field, "must not be empty"));
    }

    Ok(())
}

fn validate_operations(
    operations: Vec<Operation>,
    allowed_controls: &[String],
    max_operations: i32,
    field: &str,
) -> Result<Vec<Operation>, ProblemDataError> {
    if operations.is_empty() {
        return Err(validation_error(
            field,
            "operation sequence must not be empty",
        ));
    }

    let mut total = 0_i64;

    for (index, operation) in operations.iter().enumerate() {
        if operation.count <= 0 {
            return Err(validation_error(
                format!("{field}[{index}].count"),
                "operation count must be greater than zero",
            ));
        }

        if !allowed_controls
            .iter()
            .any(|allowed| allowed == &operation.control)
        {
            return Err(validation_error(
                format!("{field}[{index}].control"),
                "operation control must be allowed by input schema",
            ));
        }

        total += i64::from(operation.count);

        if total > i64::from(max_operations) {
            return Err(validation_error(
                field,
                "operation count total must not exceed max operations",
            ));
        }
    }

    Ok(normalize_operations(&operations))
}

fn validate_operation_judge_config(
    correct_operations: Vec<Operation>,
    candidates: Vec<Candidate>,
    allowed_controls: &[String],
    max_operations: i32,
    field: &str,
) -> Result<JudgeConfig, ProblemDataError> {
    let correct_operations = validate_operations(
        correct_operations,
        allowed_controls,
        max_operations,
        &format!("{field}.correct_operations"),
    )?;

    if candidates.is_empty() {
        return Err(validation_error(
            format!("{field}.candidates"),
            "at least one candidate is required",
        ));
    }

    let mut candidate_ids = HashSet::new();
    let mut candidate_operations = Vec::new();
    let mut normalized_candidates = Vec::new();

    for (index, candidate) in candidates.into_iter().enumerate() {
        let candidate_id = candidate.candidate_id.trim().to_owned();

        if candidate_id.is_empty() {
            return Err(validation_error(
                format!("{field}.candidates[{index}].candidate_id"),
                "candidate ID must not be empty",
            ));
        }

        if !candidate_ids.insert(candidate_id.clone()) {
            return Err(validation_error(
                format!("{field}.candidates[{index}].candidate_id"),
                "candidate ID must not be duplicated",
            ));
        }

        let operations = validate_operations(
            candidate.operations,
            allowed_controls,
            max_operations,
            &format!("{field}.candidates[{index}].operations"),
        )?;

        if candidate_operations.contains(&operations) {
            return Err(validation_error(
                format!("{field}.candidates[{index}].operations"),
                "normalized candidate operations must not be duplicated",
            ));
        }

        candidate_operations.push(operations.clone());
        normalized_candidates.push(Candidate {
            candidate_id,
            operations,
        });
    }

    if !candidate_operations.contains(&correct_operations) {
        return Err(validation_error(
            format!("{field}.correct_operations"),
            "correct operations must match one candidate",
        ));
    }

    Ok(JudgeConfig::OperationSequence {
        correct_operations,
        candidates: normalized_candidates,
    })
}

fn validate_string_judge_config(
    accepted_answers: Vec<String>,
    normalization: StringNormalization,
    max_length: i32,
    field: &str,
) -> Result<JudgeConfig, ProblemDataError> {
    if accepted_answers.is_empty() {
        return Err(validation_error(
            format!("{field}.accepted_answers"),
            "at least one accepted answer is required",
        ));
    }

    let mut normalized_answers = Vec::new();
    let mut unique_answers = HashSet::new();

    for (index, answer) in accepted_answers.into_iter().enumerate() {
        if answer.chars().count() > max_length as usize {
            return Err(validation_error(
                format!("{field}.accepted_answers[{index}]"),
                "accepted answer must not exceed max length",
            ));
        }

        let normalized = normalize_answer(&answer, &normalization);

        if normalized.is_empty() {
            return Err(validation_error(
                format!("{field}.accepted_answers[{index}]"),
                "normalized answer must not be empty",
            ));
        }

        if !unique_answers.insert(normalized.clone()) {
            return Err(validation_error(
                format!("{field}.accepted_answers[{index}]"),
                "normalized accepted answers must not be duplicated",
            ));
        }

        normalized_answers.push(normalized);
    }

    Ok(JudgeConfig::String {
        accepted_answers: normalized_answers,
        normalization,
    })
}

const MAX_ASSET_SIZE: u64 = 5 * 1024 * 1024;

fn validate_asset(
    asset: &Asset,
    room_id: Uuid,
    room_directory: &Path,
    field: &str,
) -> Result<(), ProblemDataError> {
    let url_prefix = format!("/assets/problems/{room_id}/");

    let file_name = asset.url.strip_prefix(&url_prefix).ok_or_else(|| {
        validation_error(
            format!("{field}.url"),
            "asset URL must contain its parent room ID",
        )
    })?;

    let mut characters = file_name.chars();

    let valid_first_character = characters
        .next()
        .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit());

    let valid_remaining_characters = characters.all(|character| {
        character.is_ascii_lowercase()
            || character.is_ascii_digit()
            || matches!(character, '.' | '_' | '-')
    });

    if !valid_first_character
        || !valid_remaining_characters
        || file_name.contains("..")
        || file_name.contains('/')
        || file_name.contains('\\')
    {
        return Err(validation_error(
            format!("{field}.url"),
            "asset file name has an invalid format",
        ));
    }

    let expected_format = match Path::new(file_name)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("png") => "png",
        Some("jpg" | "jpeg") => "jpeg",
        Some("webp") => "webp",
        _ => {
            return Err(validation_error(
                format!("{field}.url"),
                "asset extension must be png, jpg, jpeg, or webp",
            ));
        }
    };

    let asset_path = room_directory.join("assets").join(file_name);

    if !asset_path.exists() {
        return Err(validation_error(
            format!("{field}.url"),
            "asset file must exist",
        ));
    }

    let metadata = fs::metadata(&asset_path).map_err(|source| ProblemDataError::Io {
        path: asset_path.clone(),
        source,
    })?;

    if !metadata.is_file() {
        return Err(validation_error(
            format!("{field}.url"),
            "asset path must point to a regular file",
        ));
    }

    if metadata.len() > MAX_ASSET_SIZE {
        return Err(validation_error(
            format!("{field}.url"),
            "asset file must not exceed 5 MiB",
        ));
    }

    let data = fs::read(&asset_path).map_err(|source| ProblemDataError::Io {
        path: asset_path,
        source,
    })?;

    let actual_format = detect_image_format(&data).ok_or_else(|| {
        validation_error(
            format!("{field}.url"),
            "asset file must be PNG, JPEG, or WebP",
        )
    })?;

    if actual_format != expected_format {
        return Err(validation_error(
            format!("{field}.url"),
            "asset extension must match its file content",
        ));
    }

    Ok(())
}

fn detect_image_format(data: &[u8]) -> Option<&'static str> {
    if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else if data.starts_with(&[0xff, 0xd8, 0xff]) {
        Some("jpeg")
    } else if data.len() >= 12 && &data[..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        Some("webp")
    } else {
        None
    }
}

fn validate_problem(
    input: ProblemInput,
    expected_room_id: Uuid,
    index: usize,
    room_directory: &Path,
) -> Result<Problem, ProblemDataError> {
    let field_prefix = format!("problems[{index}]");

    let problem_id = parse_uuid(&input.problem_id, format!("{field_prefix}.problem_id"))?;

    let room_id = parse_uuid(&input.room_id, format!("{field_prefix}.room_id"))?;

    if room_id != expected_room_id {
        return Err(validation_error(
            format!("{field_prefix}.room_id"),
            "problem room ID must match its parent room",
        ));
    }

    if input.number <= 0 {
        return Err(validation_error(
            format!("{field_prefix}.number"),
            "problem number must be greater than zero",
        ));
    }

    require_non_empty(&input.title, format!("{field_prefix}.title"))?;

    require_non_empty(
        &input.body_markdown,
        format!("{field_prefix}.body_markdown"),
    )?;

    if input.input_schema.query.max_operations <= 0 {
        return Err(validation_error(
            format!("{field_prefix}.input_schema.query.max_operations"),
            "max operations must be greater than zero",
        ));
    }

    if input.input_schema.answer.max_length <= 0 {
        return Err(validation_error(
            format!("{field_prefix}.input_schema.answer.max_length"),
            "max length must be greater than zero",
        ));
    }

    if input.input_schema.query.allowed_controls.is_empty() {
        return Err(validation_error(
            format!("{field_prefix}.input_schema.query.allowed_controls"),
            "at least one control is required",
        ));
    }

    let mut controls = HashSet::new();

    for (control_index, control) in input.input_schema.query.allowed_controls.iter().enumerate() {
        require_non_empty(
            control,
            format!("{field_prefix}.input_schema.query.allowed_controls[{control_index}]"),
        )?;

        if !controls.insert(control) {
            return Err(validation_error(
                format!("{field_prefix}.input_schema.query.allowed_controls[{control_index}]"),
                "controls must not be duplicated",
            ));
        }
    }

    for (asset_index, asset) in input.assets.iter().enumerate() {
        if asset.asset_type != "image" {
            return Err(validation_error(
                format!("{field_prefix}.assets[{asset_index}].type"),
                "asset type must be image",
            ));
        }

        require_non_empty(
            &asset.url,
            format!("{field_prefix}.assets[{asset_index}].url"),
        )?;

        require_non_empty(
            &asset.alt,
            format!("{field_prefix}.assets[{asset_index}].alt"),
        )?;

        validate_asset(
            asset,
            expected_room_id,
            room_directory,
            &format!("{field_prefix}.assets[{asset_index}]"),
        )?;
    }

    for (hint_index, hint) in input.hints.iter().enumerate() {
        require_non_empty(
            &hint.body_markdown,
            format!("{field_prefix}.hints[{hint_index}].body_markdown"),
        )?;
    }

    let depends_on_problem_id = input
        .depends_on_problem_id
        .as_deref()
        .map(|value| parse_uuid(value, format!("{field_prefix}.depends_on_problem_id")))
        .transpose()?;

    let judge_config = match (input.submission_type, input.judge_config) {
        (
            SubmissionType::OperationSequence,
            JudgeConfigInput::OperationSequence {
                correct_operations,
                candidates,
            },
        ) => validate_operation_judge_config(
            correct_operations,
            candidates,
            &input.input_schema.query.allowed_controls,
            input.input_schema.query.max_operations,
            &format!("{field_prefix}.judge_config"),
        )?,

        (
            SubmissionType::String,
            JudgeConfigInput::String {
                accepted_answers,
                normalization,
            },
        ) => validate_string_judge_config(
            accepted_answers,
            normalization,
            input.input_schema.answer.max_length,
            &format!("{field_prefix}.judge_config"),
        )?,

        _ => {
            return Err(validation_error(
                format!("{field_prefix}.judge_config"),
                "judge config type must match submission type",
            ));
        }
    };

    Ok(Problem {
        problem_id,
        room_id,
        number: input.number,
        problem_type: input.problem_type,
        title: input.title,
        body_markdown: input.body_markdown,
        submission_type: input.submission_type,
        assets: input.assets,
        input_schema: input.input_schema,
        hints: input.hints,
        judge_config,
        depends_on_problem_id,
        is_required: input.is_required,
    })
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use uuid::Uuid;

    use super::{
        Asset, Candidate, JudgeConfig, MAX_ASSET_SIZE, Operation, ProblemDataError,
        StringNormalization, validate_asset, validate_catalog, validate_operation_judge_config,
        validate_operations, validate_string_judge_config,
    };

    use super::super::{judge::normalize_answer, model::UnicodeNormalization};

    use crate::problem::{ProblemCatalog, load_problem_data};

    fn valid_catalog() -> ProblemCatalog {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../problem-data");

        load_problem_data(root).expect("test problem data should be valid")
    }

    fn operation(control: &str, count: i32) -> Operation {
        Operation {
            control: control.to_owned(),
            count,
        }
    }

    fn candidate(candidate_id: &str, operations: Vec<Operation>) -> Candidate {
        Candidate {
            candidate_id: candidate_id.to_owned(),
            operations,
        }
    }

    fn string_normalization() -> StringNormalization {
        StringNormalization {
            unicode: UnicodeNormalization::Nfkc,
            trim_outer_whitespace: true,
            collapse_internal_whitespace: true,
            case_sensitive: false,
        }
    }

    fn assert_problem_data_error(
        error: ProblemDataError,
        expected_field: &str,
        expected_message: &str,
    ) {
        match error {
            ProblemDataError::Validation { field, message } => {
                assert_eq!(field, expected_field);
                assert_eq!(message, expected_message);
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    fn assert_validation_error(
        catalog: &ProblemCatalog,
        expected_field: &str,
        expected_message: &str,
    ) {
        let error = validate_catalog(catalog).expect_err("validation should fail");

        assert_problem_data_error(error, expected_field, expected_message);
    }

    struct TestRoomDirectory {
        path: PathBuf,
    }

    impl TestRoomDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("problem-validation-{}", Uuid::new_v4()));

            fs::create_dir_all(path.join("assets"))
                .expect("test asset directory should be created");

            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestRoomDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn image_asset(room_id: Uuid, file_name: &str) -> Asset {
        Asset {
            asset_type: "image".to_owned(),
            url: format!("/assets/problems/{room_id}/{file_name}"),
            alt: "問題資料".to_owned(),
        }
    }

    #[test]
    fn duplicate_problem_number_is_rejected() {
        let mut catalog = valid_catalog();

        catalog.rooms[0].problems[1].number = catalog.rooms[0].problems[0].number;

        assert_validation_error(
            &catalog,
            "rooms[0].problems[1].number",
            "problem number must not be duplicated within a room",
        );
    }

    #[test]
    fn missing_dependency_is_rejected() {
        let mut catalog = valid_catalog();

        catalog.rooms[0].problems[1].depends_on_problem_id = Some(Uuid::new_v4());

        assert_validation_error(
            &catalog,
            "rooms[0].problems[1].depends_on_problem_id",
            "dependency must reference an existing problem",
        );
    }

    #[test]
    fn self_dependency_is_rejected() {
        let mut catalog = valid_catalog();
        let problem_id = catalog.rooms[0].problems[1].problem_id;

        catalog.rooms[0].problems[1].depends_on_problem_id = Some(problem_id);

        assert_validation_error(
            &catalog,
            "rooms[0].problems[1].depends_on_problem_id",
            "problem must not depend on itself",
        );
    }

    #[test]
    fn cross_room_dependency_is_rejected() {
        let mut catalog = valid_catalog();
        let mut second_room = catalog.rooms[0].clone();

        second_room.room_id = Uuid::new_v4();
        second_room.number = 2;

        let second_room_problem_ids = (0..second_room.problems.len())
            .map(|_| Uuid::new_v4())
            .collect::<Vec<_>>();

        for (index, problem) in second_room.problems.iter_mut().enumerate() {
            problem.problem_id = second_room_problem_ids[index];
            problem.room_id = second_room.room_id;
        }

        second_room.problems[0].depends_on_problem_id = None;
        second_room.problems[1].depends_on_problem_id = Some(second_room_problem_ids[0]);
        second_room.problems[2].depends_on_problem_id = Some(second_room_problem_ids[1]);
        second_room.problems[3].depends_on_problem_id = None;

        let dependency_id = second_room.problems[0].problem_id;
        catalog.rooms.push(second_room);

        catalog.rooms[0].problems[1].depends_on_problem_id = Some(dependency_id);

        assert_validation_error(
            &catalog,
            "rooms[0].problems[1].depends_on_problem_id",
            "dependency must reference a problem in the same room",
        );
    }

    #[test]
    fn dependency_cycle_is_rejected() {
        let mut catalog = valid_catalog();
        let third_problem_id = catalog.rooms[0].problems[2].problem_id;

        catalog.rooms[0].problems[0].depends_on_problem_id = Some(third_problem_id);

        assert_validation_error(
            &catalog,
            "rooms[0].problems[0].depends_on_problem_id",
            "problem dependencies must not contain a cycle",
        );
    }

    #[test]
    fn invalid_small_problem_unlock_order_is_rejected() {
        let mut catalog = valid_catalog();
        let first_problem_id = catalog.rooms[0].problems[0].problem_id;

        catalog.rooms[0].problems[2].depends_on_problem_id = Some(first_problem_id);

        assert_validation_error(
            &catalog,
            "rooms[0].problems[2].depends_on_problem_id",
            "small problems must unlock in ascending number order",
        );
    }

    #[test]
    fn final_problem_dependency_is_rejected() {
        let mut catalog = valid_catalog();
        let third_problem_id = catalog.rooms[0].problems[2].problem_id;

        catalog.rooms[0].problems[3].depends_on_problem_id = Some(third_problem_id);

        assert_validation_error(
            &catalog,
            "rooms[0].problems[3].depends_on_problem_id",
            "final problem must not have a dependency",
        );
    }

    #[test]
    fn adjacent_operations_are_normalized() {
        let operations = vec![
            operation("down", 1),
            operation("down", 2),
            operation("right", 1),
        ];

        let allowed_controls = vec!["down".to_owned(), "right".to_owned()];

        let normalized = validate_operations(operations, &allowed_controls, 4, "operations")
            .expect("operations should be valid");

        assert_eq!(
            normalized,
            vec![operation("down", 3), operation("right", 1)]
        );
    }

    #[test]
    fn empty_operations_are_rejected() {
        let error = validate_operations(Vec::new(), &["down".to_owned()], 100, "operations")
            .expect_err("empty operations should be rejected");

        assert_problem_data_error(error, "operations", "operation sequence must not be empty");
    }

    #[test]
    fn non_positive_operation_count_is_rejected() {
        let error = validate_operations(
            vec![operation("down", 0)],
            &["down".to_owned()],
            100,
            "operations",
        )
        .expect_err("zero count should be rejected");

        assert_problem_data_error(
            error,
            "operations[0].count",
            "operation count must be greater than zero",
        );
    }

    #[test]
    fn unknown_operation_control_is_rejected() {
        let error = validate_operations(
            vec![operation("left", 1)],
            &["down".to_owned()],
            100,
            "operations",
        )
        .expect_err("unknown control should be rejected");

        assert_problem_data_error(
            error,
            "operations[0].control",
            "operation control must be allowed by input schema",
        );
    }

    #[test]
    fn operation_total_over_limit_is_rejected() {
        let error = validate_operations(
            vec![operation("down", 3), operation("right", 2)],
            &["down".to_owned(), "right".to_owned()],
            4,
            "operations",
        )
        .expect_err("operation total over limit should be rejected");

        assert_problem_data_error(
            error,
            "operations",
            "operation count total must not exceed max operations",
        );
    }

    #[test]
    fn empty_candidates_are_rejected() {
        let error = validate_operation_judge_config(
            vec![operation("down", 1)],
            Vec::new(),
            &["down".to_owned()],
            100,
            "judge_config",
        )
        .expect_err("empty candidates should be rejected");

        assert_problem_data_error(
            error,
            "judge_config.candidates",
            "at least one candidate is required",
        );
    }

    #[test]
    fn duplicate_candidate_id_is_rejected_after_trimming() {
        let error = validate_operation_judge_config(
            vec![operation("down", 1)],
            vec![
                candidate("pattern-a", vec![operation("down", 1)]),
                candidate(" pattern-a ", vec![operation("down", 2)]),
            ],
            &["down".to_owned()],
            100,
            "judge_config",
        )
        .expect_err("duplicate candidate IDs should be rejected");

        assert_problem_data_error(
            error,
            "judge_config.candidates[1].candidate_id",
            "candidate ID must not be duplicated",
        );
    }

    #[test]
    fn duplicate_normalized_candidate_operations_are_rejected() {
        let error = validate_operation_judge_config(
            vec![operation("down", 2)],
            vec![
                candidate(
                    "pattern-a",
                    vec![operation("down", 1), operation("down", 1)],
                ),
                candidate("pattern-b", vec![operation("down", 2)]),
            ],
            &["down".to_owned()],
            100,
            "judge_config",
        )
        .expect_err("duplicate normalized candidates should be rejected");

        assert_problem_data_error(
            error,
            "judge_config.candidates[1].operations",
            "normalized candidate operations must not be duplicated",
        );
    }

    #[test]
    fn correct_operations_must_match_a_candidate() {
        let error = validate_operation_judge_config(
            vec![operation("down", 1)],
            vec![candidate("pattern-a", vec![operation("up", 1)])],
            &["down".to_owned(), "up".to_owned()],
            100,
            "judge_config",
        )
        .expect_err("unknown correct operations should be rejected");

        assert_problem_data_error(
            error,
            "judge_config.correct_operations",
            "correct operations must match one candidate",
        );
    }

    #[test]
    fn string_answer_is_normalized_in_documented_order() {
        let normalization = string_normalization();

        let normalized = normalize_answer("  Ａ　\tB  ", &normalization);

        assert_eq!(normalized, "a b");
    }

    #[test]
    fn empty_accepted_answers_are_rejected() {
        let error =
            validate_string_judge_config(Vec::new(), string_normalization(), 50, "judge_config")
                .expect_err("empty accepted answers should be rejected");

        assert_problem_data_error(
            error,
            "judge_config.accepted_answers",
            "at least one accepted answer is required",
        );
    }

    #[test]
    fn normalized_empty_answer_is_rejected() {
        let error = validate_string_judge_config(
            vec!["　 \t".to_owned()],
            string_normalization(),
            50,
            "judge_config",
        )
        .expect_err("whitespace-only answer should be rejected");

        assert_problem_data_error(
            error,
            "judge_config.accepted_answers[0]",
            "normalized answer must not be empty",
        );
    }

    #[test]
    fn answer_length_is_checked_before_normalization() {
        let error = validate_string_judge_config(
            vec!["ＡＢＣＤ".to_owned()],
            string_normalization(),
            3,
            "judge_config",
        )
        .expect_err("answer over max length should be rejected");

        assert_problem_data_error(
            error,
            "judge_config.accepted_answers[0]",
            "accepted answer must not exceed max length",
        );
    }

    #[test]
    fn answer_length_uses_unicode_scalar_count() {
        let judge_config = validate_string_judge_config(
            vec!["日本語".to_owned()],
            string_normalization(),
            3,
            "judge_config",
        )
        .expect("three Japanese characters should fit max length three");

        match judge_config {
            JudgeConfig::String {
                accepted_answers, ..
            } => {
                assert_eq!(accepted_answers, vec!["日本語".to_owned()]);
            }
            _ => panic!("string judge config should be returned"),
        }
    }

    #[test]
    fn answers_duplicated_after_normalization_are_rejected() {
        let error = validate_string_judge_config(
            vec!["Ａ".to_owned(), "a".to_owned()],
            string_normalization(),
            50,
            "judge_config",
        )
        .expect_err("normalized duplicate answers should be rejected");

        assert_problem_data_error(
            error,
            "judge_config.accepted_answers[1]",
            "normalized accepted answers must not be duplicated",
        );
    }

    #[test]
    fn valid_png_asset_is_accepted() {
        let room_id = Uuid::new_v4();
        let directory = TestRoomDirectory::new();
        let asset = image_asset(room_id, "image.png");

        fs::write(
            directory.path().join("assets/image.png"),
            b"\x89PNG\r\n\x1a\n",
        )
        .expect("test PNG should be written");

        validate_asset(&asset, room_id, directory.path(), "problems[0].assets[0]")
            .expect("valid PNG should be accepted");
    }

    #[test]
    fn asset_url_with_wrong_room_id_is_rejected() {
        let room_id = Uuid::new_v4();
        let asset = image_asset(Uuid::new_v4(), "image.png");
        let directory = TestRoomDirectory::new();

        let error = validate_asset(&asset, room_id, directory.path(), "problems[0].assets[0]")
            .expect_err("wrong room ID should be rejected");

        assert_problem_data_error(
            error,
            "problems[0].assets[0].url",
            "asset URL must contain its parent room ID",
        );
    }

    #[test]
    fn asset_path_traversal_is_rejected() {
        let room_id = Uuid::new_v4();
        let asset = image_asset(room_id, "../image.png");
        let directory = TestRoomDirectory::new();

        let error = validate_asset(&asset, room_id, directory.path(), "problems[0].assets[0]")
            .expect_err("path traversal should be rejected");

        assert_problem_data_error(
            error,
            "problems[0].assets[0].url",
            "asset file name has an invalid format",
        );
    }

    #[test]
    fn missing_asset_file_is_rejected() {
        let room_id = Uuid::new_v4();
        let asset = image_asset(room_id, "missing.png");
        let directory = TestRoomDirectory::new();

        let error = validate_asset(&asset, room_id, directory.path(), "problems[0].assets[0]")
            .expect_err("missing file should be rejected");

        assert_problem_data_error(error, "problems[0].assets[0].url", "asset file must exist");
    }

    #[test]
    fn asset_extension_must_match_file_signature() {
        let room_id = Uuid::new_v4();
        let directory = TestRoomDirectory::new();
        let asset = image_asset(room_id, "image.png");

        fs::write(
            directory.path().join("assets/image.png"),
            [0xff, 0xd8, 0xff],
        )
        .expect("test JPEG should be written");

        let error = validate_asset(&asset, room_id, directory.path(), "problems[0].assets[0]")
            .expect_err("signature mismatch should be rejected");

        assert_problem_data_error(
            error,
            "problems[0].assets[0].url",
            "asset extension must match its file content",
        );
    }

    #[test]
    fn asset_size_limit_is_enforced() {
        let room_id = Uuid::new_v4();
        let directory = TestRoomDirectory::new();
        let asset = image_asset(room_id, "image.png");
        let asset_path = directory.path().join("assets/image.png");

        let mut maximum_size_data = vec![0; MAX_ASSET_SIZE as usize];
        maximum_size_data[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");

        fs::write(&asset_path, &maximum_size_data).expect("maximum-size PNG should be written");

        validate_asset(&asset, room_id, directory.path(), "problems[0].assets[0]")
            .expect("asset at exactly 5 MiB should be accepted");

        maximum_size_data.push(0);

        fs::write(&asset_path, maximum_size_data).expect("oversized PNG should be written");

        let error = validate_asset(&asset, room_id, directory.path(), "problems[0].assets[0]")
            .expect_err("asset over 5 MiB should be rejected");

        assert_problem_data_error(
            error,
            "problems[0].assets[0].url",
            "asset file must not exceed 5 MiB",
        );
    }
}
