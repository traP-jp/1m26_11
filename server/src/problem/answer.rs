use thiserror::Error;

use super::{InputSchema, JudgeConfig, validation::normalize_answer};

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct AnswerJudgement {
    pub(crate) correct: bool,
}

#[derive(Debug, Error, Eq, PartialEq)]
pub(crate) enum AnswerJudgeError {
    #[error("answer is empty after normalization")]
    EmptyAnswer,

    #[error("answer exceeds the configured length limit")]
    AnswerLengthExceeded,

    #[error("problem does not accept string answers")]
    WrongSubmissionType,

    #[error("stored answer configuration is invalid")]
    InvalidStoredConfig,
}

pub(crate) fn judge_answer(
    answer: &str,
    input_schema: &InputSchema,
    judge_config: &JudgeConfig,
) -> Result<AnswerJudgement, AnswerJudgeError> {
    let JudgeConfig::String {
        accepted_answers,
        normalization,
    } = judge_config
    else {
        return Err(AnswerJudgeError::WrongSubmissionType);
    };

    let max_length = usize::try_from(input_schema.answer.max_length)
        .ok()
        .filter(|max_length| *max_length > 0)
        .ok_or(AnswerJudgeError::InvalidStoredConfig)?;

    if accepted_answers.is_empty() {
        return Err(AnswerJudgeError::InvalidStoredConfig);
    }

    if answer.chars().count() > max_length {
        return Err(AnswerJudgeError::AnswerLengthExceeded);
    }

    let normalized_answer = normalize_answer(answer, normalization);

    if normalized_answer.is_empty() {
        return Err(AnswerJudgeError::EmptyAnswer);
    }

    let correct = accepted_answers
        .iter()
        .any(|accepted_answer| accepted_answer == &normalized_answer);

    Ok(AnswerJudgement { correct })
}

#[cfg(test)]
mod tests {
    use super::{AnswerJudgeError, judge_answer};
    use crate::problem::{
        InputSchema, JudgeConfig,
        model::{
            AnswerInputSchema, AnswerInputType, QueryInputSchema, QueryInputType,
            StringNormalization, UnicodeNormalization,
        },
    };

    fn input_schema(max_length: i32) -> InputSchema {
        InputSchema {
            query: QueryInputSchema {
                input_type: QueryInputType::OperationSequence,
                allowed_controls: vec!["down".to_owned(), "right".to_owned(), "up".to_owned()],
                max_operations: 100,
            },
            answer: AnswerInputSchema {
                input_type: AnswerInputType::String,
                max_length,
            },
        }
    }

    fn normalization() -> StringNormalization {
        StringNormalization {
            unicode: UnicodeNormalization::Nfkc,
            trim_outer_whitespace: true,
            collapse_internal_whitespace: true,
            case_sensitive: false,
        }
    }

    fn string_judge_config(accepted_answers: &[&str]) -> JudgeConfig {
        JudgeConfig::String {
            accepted_answers: accepted_answers
                .iter()
                .map(|answer| (*answer).to_owned())
                .collect(),
            normalization: normalization(),
        }
    }

    #[test]
    fn exact_answer_is_correct() {
        let result = judge_answer(
            "answer",
            &input_schema(50),
            &string_judge_config(&["answer"]),
        )
        .expect("valid answer should be judged");

        assert!(result.correct);
    }

    #[test]
    fn normalized_answer_is_correct() {
        let result = judge_answer(
            "  Ａ　\tB  ",
            &input_schema(50),
            &string_judge_config(&["a b"]),
        )
        .expect("valid answer should be judged");

        assert!(result.correct);
    }

    #[test]
    fn incorrect_answer_is_not_an_error() {
        let result = judge_answer(
            "wrong",
            &input_schema(50),
            &string_judge_config(&["answer"]),
        )
        .expect("incorrect answer should still be a valid judgement");

        assert!(!result.correct);
    }

    #[test]
    fn answer_over_max_length_is_rejected() {
        let result = judge_answer("four", &input_schema(3), &string_judge_config(&["one"]));

        assert_eq!(result, Err(AnswerJudgeError::AnswerLengthExceeded));
    }

    #[test]
    fn answer_length_uses_unicode_scalar_count() {
        let result = judge_answer(
            "日本語",
            &input_schema(3),
            &string_judge_config(&["日本語"]),
        )
        .expect("three Unicode scalar values should fit max_length 3");

        assert!(result.correct);
    }

    #[test]
    fn normalized_empty_answer_is_rejected() {
        let result = judge_answer(
            "  \t　",
            &input_schema(50),
            &string_judge_config(&["answer"]),
        );

        assert_eq!(result, Err(AnswerJudgeError::EmptyAnswer));
    }

    #[test]
    fn operation_sequence_problem_is_rejected() {
        let judge_config = JudgeConfig::OperationSequence {
            correct_operations: Vec::new(),
            candidates: Vec::new(),
        };

        let result = judge_answer("answer", &input_schema(50), &judge_config);

        assert_eq!(result, Err(AnswerJudgeError::WrongSubmissionType));
    }

    #[test]
    fn invalid_stored_configuration_is_rejected() {
        let result = judge_answer(
            "answer",
            &input_schema(0),
            &string_judge_config(&["answer"]),
        );

        assert_eq!(result, Err(AnswerJudgeError::InvalidStoredConfig));
    }
}
