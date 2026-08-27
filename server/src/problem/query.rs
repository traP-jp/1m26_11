use thiserror::Error;

use super::{
    InputSchema, JudgeConfig, Operation,
    model::JudgeConfigInput,
    validation::{
        normalize_operations, validate_operation_judge_config, validate_string_judge_config,
    },
};

const ALLOWED_SOURCES: [&str; 3] = ["keyboard", "mouse", "serial"];

pub(crate) struct QueryJudgement {
    pub(crate) normalized_operations: Vec<Operation>,
    pub(crate) correct: bool,
    pub(crate) remaining_pattern_count: i32,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub(crate) enum QueryJudgeError {
    #[error("query source is not allowed")]
    InvalidSource,
    #[error("operation sequence is empty")]
    EmptyOperations,
    #[error("operation count is not positive")]
    NonPositiveCount,
    #[error("operation control is not allowed")]
    UnknownControl,
    #[error("operation count total exceeds the limit")]
    OperationLimitExceeded,
    #[error("problem does not accept operation sequences")]
    WrongSubmissionType,
    #[error("stored judge configuration is invalid")]
    InvalidStoredJudgeConfig,
}

pub(crate) fn decode_stored_judge_config(
    submission_type: &str,
    value: &serde_json::Value,
    input_schema: &InputSchema,
) -> Result<JudgeConfig, QueryJudgeError> {
    let input: JudgeConfigInput = serde_json::from_value(value.clone())
        .map_err(|_| QueryJudgeError::InvalidStoredJudgeConfig)?;

    let result = match (submission_type, input) {
        (
            "operation_sequence",
            JudgeConfigInput::OperationSequence {
                correct_operations,
                candidates,
            },
        ) => validate_operation_judge_config(
            correct_operations,
            candidates,
            &input_schema.query.allowed_controls,
            input_schema.query.max_operations,
            "judge_config",
        ),

        (
            "string",
            JudgeConfigInput::String {
                accepted_answers,
                normalization,
            },
        ) => validate_string_judge_config(
            accepted_answers,
            normalization,
            input_schema.answer.max_length,
            "judge_config",
        ),

        _ => {
            return Err(QueryJudgeError::InvalidStoredJudgeConfig);
        }
    };

    result.map_err(|_| QueryJudgeError::InvalidStoredJudgeConfig)
}

pub(crate) fn judge_query(
    source: &str,
    operations: &[Operation],
    input_schema: &InputSchema,
    judge_config: &JudgeConfig,
) -> Result<QueryJudgement, QueryJudgeError> {
    if !ALLOWED_SOURCES.contains(&source) {
        return Err(QueryJudgeError::InvalidSource);
    }

    let JudgeConfig::OperationSequence {
        correct_operations,
        candidates,
    } = judge_config
    else {
        return Err(QueryJudgeError::WrongSubmissionType);
    };

    if operations.is_empty() {
        return Err(QueryJudgeError::EmptyOperations);
    }

    let mut operation_total = 0_i64;

    for operation in operations {
        if operation.count <= 0 {
            return Err(QueryJudgeError::NonPositiveCount);
        }

        if !input_schema
            .query
            .allowed_controls
            .iter()
            .any(|control| control == &operation.control)
        {
            return Err(QueryJudgeError::UnknownControl);
        }

        operation_total += i64::from(operation.count);

        if operation_total > i64::from(input_schema.query.max_operations) {
            return Err(QueryJudgeError::OperationLimitExceeded);
        }
    }

    let normalized_operations = normalize_operations(operations);

    let correct = normalized_operations.as_slice() == correct_operations.as_slice();

    let remaining_pattern_count = candidates
        .iter()
        .filter(|candidate| operations_are_prefix(&normalized_operations, &candidate.operations))
        .count();

    let remaining_pattern_count = i32::try_from(remaining_pattern_count)
        .map_err(|_| QueryJudgeError::InvalidStoredJudgeConfig)?;

    Ok(QueryJudgement {
        normalized_operations,
        correct,
        remaining_pattern_count,
    })
}

fn operations_are_prefix(submitted: &[Operation], candidate: &[Operation]) -> bool {
    submitted
        .iter()
        .enumerate()
        .all(|(index, submitted_operation)| {
            let Some(candidate_operation) = candidate.get(index) else {
                return false;
            };

            if submitted_operation.control != candidate_operation.control {
                return false;
            }

            if index + 1 == submitted.len() {
                submitted_operation.count <= candidate_operation.count
            } else {
                submitted_operation.count == candidate_operation.count
            }
        })
}

#[cfg(test)]
mod tests {
    use super::super::{
        InputSchema, JudgeConfig, Operation,
        model::{
            AnswerInputSchema, AnswerInputType, Candidate, QueryInputSchema, QueryInputType,
            StringNormalization, UnicodeNormalization,
        },
    };
    use super::{QueryJudgeError, QueryJudgement, decode_stored_judge_config, judge_query};
    use serde_json::json;

    fn operation(control: &str, count: i32) -> Operation {
        Operation {
            control: control.to_owned(),
            count,
        }
    }

    fn input_schema(max_operations: i32) -> InputSchema {
        InputSchema {
            query: QueryInputSchema {
                input_type: QueryInputType::OperationSequence,
                allowed_controls: vec!["down".to_owned(), "right".to_owned(), "up".to_owned()],
                max_operations,
            },
            answer: AnswerInputSchema {
                input_type: AnswerInputType::String,
                max_length: 50,
            },
        }
    }

    fn candidate(candidate_id: &str, operations: Vec<Operation>) -> Candidate {
        Candidate {
            candidate_id: candidate_id.to_owned(),
            operations,
        }
    }

    fn operation_judge_config() -> JudgeConfig {
        JudgeConfig::OperationSequence {
            correct_operations: vec![operation("down", 2), operation("right", 1)],
            candidates: vec![
                candidate("correct", vec![operation("down", 2), operation("right", 1)]),
                candidate("another", vec![operation("down", 3), operation("right", 1)]),
                candidate("different", vec![operation("up", 1)]),
            ],
        }
    }

    fn expect_error(result: Result<QueryJudgement, QueryJudgeError>) -> QueryJudgeError {
        match result {
            Err(error) => error,
            Ok(_) => panic!("query judgement should fail"),
        }
    }

    fn expect_decode_error(result: Result<JudgeConfig, QueryJudgeError>) -> QueryJudgeError {
        match result {
            Err(error) => error,
            Ok(_) => {
                panic!("stored judge config should be rejected")
            }
        }
    }

    #[test]
    fn allowed_sources_are_accepted() {
        let schema = input_schema(10);
        let judge_config = operation_judge_config();
        let operations = vec![operation("down", 1)];

        for source in ["keyboard", "mouse", "serial"] {
            assert!(judge_query(source, &operations, &schema, &judge_config,).is_ok());
        }
    }

    #[test]
    fn unknown_source_is_rejected() {
        let error = expect_error(judge_query(
            "vr",
            &[operation("down", 1)],
            &input_schema(10),
            &operation_judge_config(),
        ));

        assert_eq!(error, QueryJudgeError::InvalidSource);
    }
    #[test]
    fn adjacent_operations_are_normalized_and_judged() {
        let result = judge_query(
            "serial",
            &[
                operation("down", 1),
                operation("down", 1),
                operation("right", 1),
            ],
            &input_schema(10),
            &operation_judge_config(),
        )
        .expect("valid query should be judged");

        assert!(result.normalized_operations == vec![operation("down", 2), operation("right", 1),]);
        assert!(result.correct);
        assert_eq!(result.remaining_pattern_count, 1);
    }

    #[test]
    fn partial_sequence_keeps_matching_candidates() {
        let result = judge_query(
            "mouse",
            &[operation("down", 1)],
            &input_schema(10),
            &operation_judge_config(),
        )
        .expect("valid query should be judged");

        assert!(!result.correct);
        assert_eq!(result.remaining_pattern_count, 2);
    }

    #[test]
    fn sequence_longer_than_candidates_keeps_no_candidates() {
        let result = judge_query(
            "keyboard",
            &[operation("down", 4)],
            &input_schema(10),
            &operation_judge_config(),
        )
        .expect("valid query should be judged");

        assert!(!result.correct);
        assert_eq!(result.remaining_pattern_count, 0);
    }
    #[test]
    fn empty_operations_are_rejected() {
        let error = expect_error(judge_query(
            "serial",
            &[],
            &input_schema(10),
            &operation_judge_config(),
        ));

        assert_eq!(error, QueryJudgeError::EmptyOperations);
    }

    #[test]
    fn non_positive_count_is_rejected() {
        for count in [0, -1] {
            let error = expect_error(judge_query(
                "serial",
                &[operation("down", count)],
                &input_schema(10),
                &operation_judge_config(),
            ));

            assert_eq!(error, QueryJudgeError::NonPositiveCount);
        }
    }

    #[test]
    fn unknown_control_is_rejected() {
        let error = expect_error(judge_query(
            "serial",
            &[operation("left", 1)],
            &input_schema(10),
            &operation_judge_config(),
        ));

        assert_eq!(error, QueryJudgeError::UnknownControl);
    }

    #[test]
    fn operation_total_over_limit_is_rejected() {
        let error = expect_error(judge_query(
            "serial",
            &[operation("down", 3), operation("right", 2)],
            &input_schema(4),
            &operation_judge_config(),
        ));

        assert_eq!(error, QueryJudgeError::OperationLimitExceeded);
    }
    #[test]
    fn string_problem_is_rejected() {
        let judge_config = JudgeConfig::String {
            accepted_answers: vec!["answer".to_owned()],
            normalization: StringNormalization {
                unicode: UnicodeNormalization::Nfkc,
                trim_outer_whitespace: true,
                collapse_internal_whitespace: true,
                case_sensitive: false,
            },
        };

        let error = expect_error(judge_query(
            "serial",
            &[operation("down", 1)],
            &input_schema(10),
            &judge_config,
        ));

        assert_eq!(error, QueryJudgeError::WrongSubmissionType);
    }
    #[test]
    fn stored_operation_judge_config_is_decoded() {
        let schema = input_schema(10);

        let stored = json!({
            "type": "operation_sequence",
            "correct_operations": [
                {
                    "control": "down",
                    "count": 2
                },
                {
                    "control": "right",
                    "count": 1
                }
            ],
            "candidates": [
                {
                    "candidate_id": "correct",
                    "operations": [
                        {
                            "control": "down",
                            "count": 2
                        },
                        {
                            "control": "right",
                            "count": 1
                        }
                    ]
                }
            ]
        });

        let judge_config = decode_stored_judge_config("operation_sequence", &stored, &schema)
            .expect("stored judge config should be valid");

        let result = judge_query(
            "serial",
            &[operation("down", 2), operation("right", 1)],
            &schema,
            &judge_config,
        )
        .expect("query should be judged");

        assert!(result.correct);
        assert_eq!(result.remaining_pattern_count, 1);
    }
    #[test]
    fn stored_judge_config_type_mismatch_is_rejected() {
        let stored = json!({
            "type": "string",
            "accepted_answers": ["answer"],
            "normalization": {
                "unicode": "nfkc",
                "trim_outer_whitespace": true,
                "collapse_internal_whitespace": true,
                "case_sensitive": false
            }
        });

        let error = expect_decode_error(decode_stored_judge_config(
            "operation_sequence",
            &stored,
            &input_schema(10),
        ));

        assert_eq!(error, QueryJudgeError::InvalidStoredJudgeConfig);
    }

    #[test]
    fn invalid_stored_operation_config_is_rejected() {
        let stored = json!({
            "type": "operation_sequence",
            "correct_operations": [
                {
                    "control": "down",
                    "count": 1
                }
            ],
            "candidates": []
        });

        let error = expect_decode_error(decode_stored_judge_config(
            "operation_sequence",
            &stored,
            &input_schema(10),
        ));

        assert_eq!(error, QueryJudgeError::InvalidStoredJudgeConfig);
    }
}
