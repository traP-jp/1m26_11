use openapi_generated::models::{
    AnswerInputSchema as PublicAnswerInputSchema, Asset as PublicAsset,
    ProblemInputSchema as PublicProblemInputSchema, ProblemResponse,
    ProblemStatus as PublicProblemStatus, ProblemType as PublicProblemType,
    QueryInputSchema as PublicQueryInputSchema, SubmissionType as PublicSubmissionType,
};
use thiserror::Error;

use super::{AssetUrlResolveError, AssetUrlResolver};
use crate::repository::ProblemDetailRecord;

#[derive(Debug, Error)]
pub enum ProblemProjectionError {
    #[error("stored problem field is invalid: {field}")]
    InvalidStoredField { field: &'static str },

    #[error(transparent)]
    AssetUrl(#[from] AssetUrlResolveError),
}

pub fn build_problem_response(
    record: ProblemDetailRecord,
    asset_url_resolver: &dyn AssetUrlResolver,
) -> Result<ProblemResponse, ProblemProjectionError> {
    let problem_type = match record.problem_type.as_str() {
        "small" => PublicProblemType::Small,
        "final" => PublicProblemType::Final,
        _ => {
            return Err(ProblemProjectionError::InvalidStoredField {
                field: "problem_type",
            });
        }
    };

    let submission_type = match record.submission_type.as_str() {
        "operation_sequence" => PublicSubmissionType::OperationSequence,
        "string" => PublicSubmissionType::String,
        _ => {
            return Err(ProblemProjectionError::InvalidStoredField {
                field: "submission_type",
            });
        }
    };

    let status = match record.status.as_str() {
        "locked" => PublicProblemStatus::Locked,
        "available" => PublicProblemStatus::Available,
        "cleared" => PublicProblemStatus::Cleared,
        _ => {
            return Err(ProblemProjectionError::InvalidStoredField { field: "status" });
        }
    };

    let assets = record
        .assets
        .0
        .into_iter()
        .map(|asset| {
            let url = asset_url_resolver.resolve(&asset.object_key)?;

            Ok(PublicAsset::new(asset.asset_type, url, asset.alt))
        })
        .collect::<Result<Vec<_>, ProblemProjectionError>>()?;

    let input_schema = record.input_schema.0;

    let query = PublicQueryInputSchema::new(
        "operation_sequence".to_owned(),
        input_schema.query.allowed_controls,
        input_schema.query.max_operations,
    );

    let answer = PublicAnswerInputSchema::new("string".to_owned(), input_schema.answer.max_length);

    let input_schema = PublicProblemInputSchema::new(query, answer);

    let hint_count = i32::try_from(record.hint_count).map_err(|_| {
        ProblemProjectionError::InvalidStoredField {
            field: "hint_count",
        }
    })?;

    if hint_count < 0 {
        return Err(ProblemProjectionError::InvalidStoredField {
            field: "hint_count",
        });
    }

    Ok(ProblemResponse::new(
        record.id,
        record.number,
        problem_type,
        record.title,
        record.body_markdown,
        submission_type,
        assets,
        status,
        input_schema,
        hint_count,
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use sqlx::types::Json;
    use uuid::Uuid;

    use super::{ProblemProjectionError, build_problem_response};
    use crate::{
        problem::{
            AssetUrlResolveError, AssetUrlResolver,
            model::{
                AnswerInputSchema, AnswerInputType, Asset, InputSchema, QueryInputSchema,
                QueryInputType,
            },
        },
        repository::ProblemDetailRecord,
    };

    #[derive(Default)]
    struct RecordingAssetUrlResolver {
        object_keys: Mutex<Vec<String>>,
    }

    impl AssetUrlResolver for RecordingAssetUrlResolver {
        fn resolve(&self, object_key: &str) -> Result<String, AssetUrlResolveError> {
            self.object_keys
                .lock()
                .expect("resolver call log should not be poisoned")
                .push(object_key.to_owned());

            Ok("/assets/problems/birthday.png".to_owned())
        }
    }

    struct FailingAssetUrlResolver;

    impl AssetUrlResolver for FailingAssetUrlResolver {
        fn resolve(&self, _object_key: &str) -> Result<String, AssetUrlResolveError> {
            Err(AssetUrlResolveError)
        }
    }

    fn valid_record() -> ProblemDetailRecord {
        ProblemDetailRecord {
            id: Uuid::parse_str("22222222-2222-4222-8222-222222222221")
                .expect("fixture problem ID should be valid"),
            number: 1,
            problem_type: "small".to_owned(),
            title: "生年月日".to_owned(),
            body_markdown: "問題文です".to_owned(),
            submission_type: "operation_sequence".to_owned(),
            assets: Json(vec![Asset {
                asset_type: "image".to_owned(),
                object_key: "private/problem-assets/birthday.png".to_owned(),
                alt: "問題資料".to_owned(),
            }]),
            input_schema: Json(InputSchema {
                query: QueryInputSchema {
                    input_type: QueryInputType::OperationSequence,
                    allowed_controls: vec!["down".to_owned(), "right".to_owned(), "up".to_owned()],
                    max_operations: 100,
                },
                answer: AnswerInputSchema {
                    input_type: AnswerInputType::String,
                    max_length: 50,
                },
            }),
            judge_config: Json(serde_json::json!({
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
            })),
            status: "available".to_owned(),
            hint_count: 2,
        }
    }

    #[test]
    fn public_problem_matches_openapi_fixture() {
        let resolver = RecordingAssetUrlResolver::default();

        let response = build_problem_response(valid_record(), &resolver)
            .expect("valid problem should be projected");

        let actual = serde_json::to_value(response).expect("response should serialize");

        let expected: serde_json::Value = serde_json::from_str(include_str!(
            "../../../openapi/examples/problems/available-response.json"
        ))
        .expect("OpenAPI fixture should be valid JSON");

        assert_eq!(actual, expected);

        assert_eq!(
            *resolver
                .object_keys
                .lock()
                .expect("resolver call log should not be poisoned"),
            vec!["private/problem-assets/birthday.png".to_owned()],
        );
    }

    #[test]
    fn invalid_stored_type_does_not_expose_its_value() {
        let resolver = RecordingAssetUrlResolver::default();
        let forbidden_value = "private-invalid-problem-type";
        let mut record = valid_record();
        record.problem_type = forbidden_value.to_owned();

        let error = build_problem_response(record, &resolver)
            .expect_err("invalid problem type should be rejected");

        assert!(matches!(
            error,
            ProblemProjectionError::InvalidStoredField {
                field: "problem_type"
            }
        ));
        assert!(!error.to_string().contains(forbidden_value));
        assert!(!format!("{error:?}").contains(forbidden_value));
    }

    #[test]
    fn invalid_hint_counts_are_rejected() {
        let resolver = RecordingAssetUrlResolver::default();

        for invalid_hint_count in [-1, i64::from(i32::MAX) + 1] {
            let mut record = valid_record();
            record.hint_count = invalid_hint_count;

            let error = build_problem_response(record, &resolver)
                .expect_err("invalid hint count should be rejected");

            assert!(matches!(
                error,
                ProblemProjectionError::InvalidStoredField {
                    field: "hint_count"
                }
            ));
        }
    }

    #[test]
    fn resolver_error_does_not_expose_object_key() {
        let forbidden_object_key = "private/problem-assets/do-not-log.png";

        let mut record = valid_record();
        record.assets.0[0].object_key = forbidden_object_key.to_owned();

        let error = build_problem_response(record, &FailingAssetUrlResolver)
            .expect_err("resolver failure should be returned");

        assert!(matches!(error, ProblemProjectionError::AssetUrl(_)));
        assert!(!error.to_string().contains(forbidden_object_key));
        assert!(!format!("{error:?}").contains(forbidden_object_key));
    }
}
