use std::collections::HashSet;

use openapi_generated::{
    models::{
        CreateProblemJudgeConfig, CreateProblemRequest, Operation as ApiOperation,
        ProblemType as ApiProblemType, SubmissionType as ApiSubmissionType,
    },
    types::Nullable,
};
use thiserror::Error;
use uuid::Uuid;

use super::{
    ProblemDataError,
    model::{
        AnswerInputSchema, AnswerInputType, Candidate, Hint, InputSchema, Operation, ProblemDraft,
        ProblemType, QueryInputSchema, QueryInputType, StringNormalization, SubmissionType,
        UnicodeNormalization,
    },
    validation::{validate_operation_judge_config, validate_string_judge_config},
};

const PROBLEM_TITLE_MAX_CHARS: usize = 255;

#[derive(Debug, Error, Eq, PartialEq)]
#[error("invalid problem at {field}: {message}")]
pub enum ProblemAuthoringError {
    Validation {
        field: String,
        message: &'static str,
    },
}

pub fn validate_problem_draft(
    room_id: Uuid,
    request: CreateProblemRequest,
) -> Result<ProblemDraft, ProblemAuthoringError> {
    let CreateProblemRequest {
        number,
        problem_type,
        title,
        body_markdown,
        submission_type,
        input_schema,
        hints,
        judge_config,
        depends_on_problem_id,
        is_required,
    } = request;

    let number = i32::try_from(number)
        .map_err(|_| validation_error("number", "must fit in a signed 32-bit integer"))?;

    require_non_empty(&title, "title")?;
    require_max_chars(
        &title,
        PROBLEM_TITLE_MAX_CHARS,
        "title",
        "must be at most 255 characters",
    )?;
    require_non_empty(&body_markdown, "body_markdown")?;

    if input_schema.query.r_type != "operation_sequence" {
        return Err(validation_error(
            "input_schema.query.type",
            "must be operation_sequence",
        ));
    }

    if input_schema.answer.r_type != "string" {
        return Err(validation_error(
            "input_schema.answer.type",
            "must be string",
        ));
    }

    if input_schema.query.max_operations <= 0 {
        return Err(validation_error(
            "input_schema.query.max_operations",
            "must be greater than zero",
        ));
    }

    if input_schema.answer.max_length <= 0 {
        return Err(validation_error(
            "input_schema.answer.max_length",
            "must be greater than zero",
        ));
    }

    if input_schema.query.allowed_controls.is_empty() {
        return Err(validation_error(
            "input_schema.query.allowed_controls",
            "at least one control is required",
        ));
    }

    let mut controls = HashSet::new();

    for (index, control) in input_schema.query.allowed_controls.iter().enumerate() {
        require_non_empty(
            control,
            format!("input_schema.query.allowed_controls[{index}]"),
        )?;

        if !controls.insert(control.clone()) {
            return Err(validation_error(
                format!("input_schema.query.allowed_controls[{index}]"),
                "controls must not be duplicated",
            ));
        }
    }

    let input_schema = InputSchema {
        query: QueryInputSchema {
            input_type: QueryInputType::OperationSequence,
            allowed_controls: input_schema.query.allowed_controls,
            max_operations: input_schema.query.max_operations,
        },
        answer: AnswerInputSchema {
            input_type: AnswerInputType::String,
            max_length: input_schema.answer.max_length,
        },
    };

    let hints = hints
        .into_iter()
        .enumerate()
        .map(|(index, hint)| {
            require_non_empty(&hint.body_markdown, format!("hints[{index}].body_markdown"))?;

            Ok(Hint {
                body_markdown: hint.body_markdown,
            })
        })
        .collect::<Result<Vec<_>, ProblemAuthoringError>>()?;

    let problem_type = match problem_type {
        ApiProblemType::Small => ProblemType::Small,
        ApiProblemType::Final => ProblemType::Final,
    };

    let submission_type = match submission_type {
        ApiSubmissionType::OperationSequence => SubmissionType::OperationSequence,
        ApiSubmissionType::String => SubmissionType::String,
    };

    let depends_on_problem_id = match depends_on_problem_id {
        Nullable::Null => None,
        Nullable::Present(problem_id) => Some(problem_id),
    };

    if problem_type == ProblemType::Final && depends_on_problem_id.is_some() {
        return Err(validation_error(
            "depends_on_problem_id",
            "final problem must not have a dependency",
        ));
    }

    let judge_config = match (submission_type, judge_config) {
        (
            SubmissionType::OperationSequence,
            CreateProblemJudgeConfig::CreateOperationSequenceJudgeConfig(config),
        ) => {
            if config.r_type != "operation_sequence" {
                return Err(validation_error(
                    "judge_config.type",
                    "must be operation_sequence",
                ));
            }

            let correct_operations = convert_operations(config.correct_operations);
            let candidates = config
                .candidates
                .into_iter()
                .map(|candidate| Candidate {
                    candidate_id: candidate.candidate_id,
                    operations: convert_operations(candidate.operations),
                })
                .collect();

            validate_operation_judge_config(
                correct_operations,
                candidates,
                &input_schema.query.allowed_controls,
                input_schema.query.max_operations,
                "judge_config",
            )
            .map_err(map_problem_data_error)?
        }
        (SubmissionType::String, CreateProblemJudgeConfig::CreateStringJudgeConfig(config)) => {
            if config.r_type != "string" {
                return Err(validation_error("judge_config.type", "must be string"));
            }

            if config.normalization.unicode != "nfkc" {
                return Err(validation_error(
                    "judge_config.normalization.unicode",
                    "must be nfkc",
                ));
            }

            let normalization = StringNormalization {
                unicode: UnicodeNormalization::Nfkc,
                trim_outer_whitespace: config.normalization.trim_outer_whitespace,
                collapse_internal_whitespace: config.normalization.collapse_internal_whitespace,
                case_sensitive: config.normalization.case_sensitive,
            };

            validate_string_judge_config(
                vec![config.accepted_answer],
                normalization,
                input_schema.answer.max_length,
                "judge_config",
            )
            .map_err(map_problem_data_error)?
        }
        _ => {
            return Err(validation_error(
                "judge_config",
                "type must match submission_type",
            ));
        }
    };

    Ok(ProblemDraft {
        room_id,
        number,
        problem_type,
        title,
        body_markdown,
        submission_type,
        input_schema,
        hints,
        judge_config,
        depends_on_problem_id,
        is_required,
    })
}

fn convert_operations(operations: Vec<ApiOperation>) -> Vec<Operation> {
    operations
        .into_iter()
        .map(|operation| Operation {
            control: operation.control,
            count: operation.count,
        })
        .collect()
}

fn require_non_empty(value: &str, field: impl Into<String>) -> Result<(), ProblemAuthoringError> {
    if value.trim().is_empty() {
        return Err(validation_error(field, "must not be empty"));
    }

    Ok(())
}

fn require_max_chars(
    value: &str,
    max_chars: usize,
    field: impl Into<String>,
    message: &'static str,
) -> Result<(), ProblemAuthoringError> {
    if value.chars().count() > max_chars {
        return Err(validation_error(field, message));
    }

    Ok(())
}

fn validation_error(field: impl Into<String>, message: &'static str) -> ProblemAuthoringError {
    ProblemAuthoringError::Validation {
        field: field.into(),
        message,
    }
}

fn map_problem_data_error(error: ProblemDataError) -> ProblemAuthoringError {
    match error {
        ProblemDataError::Validation { field, message } => {
            ProblemAuthoringError::Validation { field, message }
        }
        ProblemDataError::Io { .. } | ProblemDataError::Json { .. } => {
            validation_error("judge_config", "judge configuration is invalid")
        }
    }
}

#[cfg(test)]
mod tests {
    use openapi_generated::models::{
        CreateProblemJudgeConfig, CreateProblemRequest, ProblemType as ApiProblemType,
        SubmissionType as ApiSubmissionType,
    };
    use uuid::Uuid;

    use super::{ProblemAuthoringError, validate_problem_draft};
    use crate::problem::{JudgeConfig, ProblemType, SubmissionType};

    const OPERATION_REQUEST: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../openapi/examples/problems/create-operation-sequence-request.json"
    ));

    const STRING_REQUEST: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../openapi/examples/problems/create-string-request.json"
    ));

    fn room_id() -> Uuid {
        Uuid::parse_str("11111111-1111-4111-8111-111111111111").expect("room UUID should be valid")
    }

    fn parse_request(fixture: &str) -> CreateProblemRequest {
        serde_json::from_str(fixture).expect("fixture should match the generated request model")
    }

    #[test]
    fn operation_sequence_request_is_converted() {
        let draft = validate_problem_draft(room_id(), parse_request(OPERATION_REQUEST))
            .expect("operation sequence request should be valid");

        assert_eq!(draft.room_id, room_id());
        assert_eq!(draft.number, 3);
        assert_eq!(draft.problem_type, ProblemType::Small);
        assert_eq!(draft.submission_type, SubmissionType::OperationSequence);
        assert_eq!(draft.input_schema.query.max_operations, 100);
        assert_eq!(draft.hints.len(), 1);
        assert!(draft.depends_on_problem_id.is_some());

        let JudgeConfig::OperationSequence {
            correct_operations,
            candidates,
        } = draft.judge_config
        else {
            panic!("operation request should produce operation judge config");
        };

        assert_eq!(correct_operations.len(), 2);
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].candidate_id, "pattern-a");
    }

    #[test]
    fn string_request_is_converted_to_one_accepted_answer() {
        let draft = validate_problem_draft(room_id(), parse_request(STRING_REQUEST))
            .expect("string request should be valid");

        assert_eq!(draft.problem_type, ProblemType::Final);
        assert_eq!(draft.submission_type, SubmissionType::String);
        assert_eq!(draft.depends_on_problem_id, None);

        let JudgeConfig::String {
            accepted_answers,
            normalization,
        } = draft.judge_config
        else {
            panic!("string request should produce string judge config");
        };

        assert_eq!(accepted_answers, vec!["ワンマンソン"]);
        assert!(normalization.trim_outer_whitespace);
        assert!(!normalization.case_sensitive);
    }

    #[test]
    fn submission_type_and_judge_config_must_match() {
        let mut request = parse_request(OPERATION_REQUEST);
        request.submission_type = ApiSubmissionType::String;

        let error = validate_problem_draft(room_id(), request)
            .expect_err("mismatched judge config should be rejected");

        assert_eq!(
            error,
            ProblemAuthoringError::Validation {
                field: "judge_config".to_owned(),
                message: "type must match submission_type",
            }
        );
    }

    #[test]
    fn final_problem_dependency_is_rejected() {
        let mut request = parse_request(OPERATION_REQUEST);
        request.problem_type = ApiProblemType::Final;

        let error = validate_problem_draft(room_id(), request)
            .expect_err("final problem dependency should be rejected");

        assert_eq!(
            error,
            ProblemAuthoringError::Validation {
                field: "depends_on_problem_id".to_owned(),
                message: "final problem must not have a dependency",
            }
        );
    }

    #[test]
    fn generated_judge_config_variants_are_distinguishable() {
        let operation_request = parse_request(OPERATION_REQUEST);
        assert!(matches!(
            operation_request.judge_config,
            CreateProblemJudgeConfig::CreateOperationSequenceJudgeConfig(_)
        ));

        let string_request = parse_request(STRING_REQUEST);
        assert!(matches!(
            string_request.judge_config,
            CreateProblemJudgeConfig::CreateStringJudgeConfig(_)
        ));
    }
}
