use thiserror::Error;
use unicode_normalization::UnicodeNormalization as _;

use super::model::{
    InputSchema, JudgeConfig, Operation, StringNormalization, UnicodeNormalization,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationJudgement {
    pub normalized_operations: Vec<Operation>,
    pub correct: bool,
    pub remaining_pattern_count: usize,
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum JudgeInputError {
    #[error("judge config type does not match the submitted input")]
    JudgeConfigTypeMismatch,

    #[error("input schema contains an invalid limit")]
    InvalidInputSchema,

    #[error("operation sequence must not be empty")]
    EmptyOperationSequence,

    #[error("operation at index {index} must have a positive count")]
    InvalidOperationCount { index: usize },

    #[error("operation at index {index} uses a control that is not allowed")]
    UnknownOperationControl { index: usize },

    #[error("operation count total exceeds max_operations")]
    TooManyOperations,

    #[error("answer exceeds max_length")]
    AnswerTooLong,
}

pub fn judge_operation_sequence(
    input_schema: &InputSchema,
    judge_config: &JudgeConfig,
    operations: &[Operation],
) -> Result<OperationJudgement, JudgeInputError> {
    let JudgeConfig::OperationSequence {
        correct_operations,
        candidates,
    } = judge_config
    else {
        return Err(JudgeInputError::JudgeConfigTypeMismatch);
    };

    if input_schema.query.max_operations <= 0 {
        return Err(JudgeInputError::InvalidInputSchema);
    }

    if operations.is_empty() {
        return Err(JudgeInputError::EmptyOperationSequence);
    }

    let mut total = 0_i64;

    for (index, operation) in operations.iter().enumerate() {
        if operation.count <= 0 {
            return Err(JudgeInputError::InvalidOperationCount { index });
        }

        if !input_schema
            .query
            .allowed_controls
            .iter()
            .any(|allowed| allowed == &operation.control)
        {
            return Err(JudgeInputError::UnknownOperationControl { index });
        }

        total += i64::from(operation.count);

        if total > i64::from(input_schema.query.max_operations) {
            return Err(JudgeInputError::TooManyOperations);
        }
    }

    let normalized_operations = normalize_operations(operations);

    let remaining_pattern_count = candidates
        .iter()
        .filter(|candidate| is_operation_prefix(&normalized_operations, &candidate.operations))
        .count();

    Ok(OperationJudgement {
        correct: normalized_operations == *correct_operations,
        normalized_operations,
        remaining_pattern_count,
    })
}

pub fn judge_string_answer(
    input_schema: &InputSchema,
    judge_config: &JudgeConfig,
    answer: &str,
) -> Result<bool, JudgeInputError> {
    let JudgeConfig::String {
        accepted_answers,
        normalization,
    } = judge_config
    else {
        return Err(JudgeInputError::JudgeConfigTypeMismatch);
    };

    if input_schema.answer.max_length <= 0 {
        return Err(JudgeInputError::InvalidInputSchema);
    }

    if answer.chars().count() > input_schema.answer.max_length as usize {
        return Err(JudgeInputError::AnswerTooLong);
    }

    let normalized_answer = normalize_answer(answer, normalization);

    Ok(accepted_answers.contains(&normalized_answer))
}

pub(super) fn normalize_operations(operations: &[Operation]) -> Vec<Operation> {
    let mut normalized: Vec<Operation> = Vec::new();

    for operation in operations {
        if let Some(previous) = normalized.last_mut()
            && previous.control == operation.control
        {
            previous.count += operation.count;
        } else {
            normalized.push(operation.clone());
        }
    }

    normalized
}

pub(super) fn normalize_answer(value: &str, normalization: &StringNormalization) -> String {
    let mut normalized = match normalization.unicode {
        UnicodeNormalization::Nfkc => value.nfkc().collect::<String>(),
    };

    if normalization.trim_outer_whitespace {
        normalized = normalized.trim().to_owned();
    }

    if normalization.collapse_internal_whitespace {
        normalized = collapse_whitespace(&normalized);
    }

    if !normalization.case_sensitive {
        normalized = normalized.chars().flat_map(char::to_lowercase).collect();
    }

    normalized
}

fn collapse_whitespace(value: &str) -> String {
    let mut result = String::new();
    let mut previous_was_whitespace = false;

    for character in value.chars() {
        if character.is_whitespace() {
            if !previous_was_whitespace {
                result.push(' ');
            }

            previous_was_whitespace = true;
        } else {
            result.push(character);
            previous_was_whitespace = false;
        }
    }

    result
}

fn is_operation_prefix(submitted: &[Operation], candidate: &[Operation]) -> bool {
    let mut candidate_controls = expanded_controls(candidate);

    expanded_controls(submitted).all(|control| candidate_controls.next() == Some(control))
}

fn expanded_controls(operations: &[Operation]) -> impl Iterator<Item = &str> {
    operations.iter().flat_map(|operation| {
        std::iter::repeat_n(operation.control.as_str(), operation.count as usize)
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{JudgeInputError, judge_operation_sequence, judge_string_answer};
    use crate::problem::{Operation, Problem, load_problem_data};

    fn problem(title: &str) -> Problem {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../mock-problem-data");

        load_problem_data(root)
            .expect("problem data should be valid")
            .rooms
            .into_iter()
            .flat_map(|room| room.problems)
            .find(|problem| problem.title == title)
            .expect("requested problem should exist")
    }

    fn operation(control: &str, count: i32) -> Operation {
        Operation {
            control: control.to_owned(),
            count,
        }
    }

    #[test]
    fn correct_operation_sequence_is_accepted() {
        let problem = problem("生年月日");

        let result = judge_operation_sequence(
            &problem.input_schema,
            &problem.judge_config,
            &[operation("down", 1), operation("right", 2)],
        )
        .expect("operation sequence should be valid");

        assert!(result.correct);
        assert_eq!(result.remaining_pattern_count, 1);
    }

    #[test]
    fn adjacent_operations_are_normalized_before_judging() {
        let problem = problem("生年月日");

        let result = judge_operation_sequence(
            &problem.input_schema,
            &problem.judge_config,
            &[
                operation("down", 1),
                operation("right", 1),
                operation("right", 1),
            ],
        )
        .expect("operation sequence should be valid");

        assert!(result.correct);
        assert_eq!(
            result.normalized_operations,
            vec![operation("down", 1), operation("right", 2),]
        );
    }

    #[test]
    fn operation_prefix_keeps_matching_candidates() {
        let problem = problem("生年月日");

        let result = judge_operation_sequence(
            &problem.input_schema,
            &problem.judge_config,
            &[operation("down", 1)],
        )
        .expect("operation sequence should be valid");

        assert!(!result.correct);
        assert_eq!(result.remaining_pattern_count, 2);
    }

    #[test]
    fn non_matching_prefix_removes_all_candidates() {
        let problem = problem("生年月日");

        let result = judge_operation_sequence(
            &problem.input_schema,
            &problem.judge_config,
            &[operation("right", 1)],
        )
        .expect("operation sequence should be valid");

        assert!(!result.correct);
        assert_eq!(result.remaining_pattern_count, 0);
    }

    #[test]
    fn invalid_operation_input_is_rejected() {
        let problem = problem("生年月日");

        let error = judge_operation_sequence(
            &problem.input_schema,
            &problem.judge_config,
            &[operation("down", 0)],
        )
        .expect_err("zero count should be rejected");

        assert_eq!(error, JudgeInputError::InvalidOperationCount { index: 0 });
    }

    #[test]
    fn operation_limit_is_enforced_when_judging() {
        let problem = problem("生年月日");

        let error = judge_operation_sequence(
            &problem.input_schema,
            &problem.judge_config,
            &[operation("down", 101)],
        )
        .expect_err("operation limit should be enforced");

        assert_eq!(error, JudgeInputError::TooManyOperations);
    }

    #[test]
    fn normalized_string_answer_is_accepted() {
        let problem = problem("大なぞ");

        let correct =
            judge_string_answer(&problem.input_schema, &problem.judge_config, "　ﾜﾝﾏﾝｿﾝ　")
                .expect("answer should be valid");

        assert!(correct);
    }

    #[test]
    fn alternate_string_answer_is_accepted() {
        let problem = problem("合言葉");

        let correct =
            judge_string_answer(&problem.input_schema, &problem.judge_config, "顔文字くん")
                .expect("answer should be valid");

        assert!(correct);
    }

    #[test]
    fn incorrect_string_answer_is_rejected() {
        let problem = problem("合言葉");

        let correct = judge_string_answer(&problem.input_schema, &problem.judge_config, "違う答え")
            .expect("answer should be valid");

        assert!(!correct);
    }

    #[test]
    fn string_length_is_checked_before_normalization() {
        let problem = problem("合言葉");
        let answer = "あ".repeat(51);

        let error = judge_string_answer(&problem.input_schema, &problem.judge_config, &answer)
            .expect_err("long answer should be rejected");

        assert_eq!(error, JudgeInputError::AnswerTooLong);
    }
}
