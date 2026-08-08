#![allow(unused_qualifications)]

use http::HeaderValue;
use validator::Validate;

#[cfg(feature = "server")]
use crate::header;
use crate::{models, types::*};

#[allow(dead_code)]
pub type SSE = std::pin::Pin<std::boxed::Box<dyn futures_util::Stream<Item = std::result::Result<axum::response::sse::Event, std::convert::Infallible>> + std::marker::Send + std::marker::Sync>>;

#[allow(dead_code)]
fn from_validation_error(e: validator::ValidationError) -> validator::ValidationErrors {
  let mut errs = validator::ValidationErrors::new();
  errs.add("na", e);
  errs
}

#[allow(dead_code)]
pub fn check_xss_string(v: &str) -> std::result::Result<(), validator::ValidationError> {
    if ammonia::is_html(v) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_vec_string(v: &[String]) -> std::result::Result<(), validator::ValidationError> {
    if v.iter().any(|i| ammonia::is_html(i)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_map_string(
    v: &std::collections::HashMap<String, String>,
) -> std::result::Result<(), validator::ValidationError> {
    if v.keys().any(|k| ammonia::is_html(k)) || v.values().any(|v| ammonia::is_html(v)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_map_nested<T>(
    v: &std::collections::HashMap<String, T>,
) -> std::result::Result<(), validator::ValidationError>
where
    T: validator::Validate,
{
    if v.keys().any(|k| ammonia::is_html(k)) || v.values().any(|v| v.validate().is_err()) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}

#[allow(dead_code)]
pub fn check_xss_map<T>(v: &std::collections::HashMap<String, T>) -> std::result::Result<(), validator::ValidationError> {
    if v.keys().any(|k| ammonia::is_html(k)) {
        std::result::Result::Err(validator::ValidationError::new("xss detected"))
    } else {
        std::result::Result::Ok(())
    }
}


    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
    #[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
    pub struct SubmitAnswerPathParams {
            /// 部屋のUUID。下記は契約用例示値であり、開始導線の実room_idではありません。 
                pub room_id: uuid::Uuid,
            /// 問題のUUID。下記は契約用例示値であり、開始導線の実problem_idではありません。 
                pub problem_id: uuid::Uuid,
    }






    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
    #[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
    pub struct GetProblemPathParams {
            /// 部屋のUUID。下記は契約用例示値であり、開始導線の実room_idではありません。 
                pub room_id: uuid::Uuid,
            /// 問題のUUID。下記は契約用例示値であり、開始導線の実problem_idではありません。 
                pub problem_id: uuid::Uuid,
    }



    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
    #[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
    pub struct SubmitQueryPathParams {
            /// 部屋のUUID。下記は契約用例示値であり、開始導線の実room_idではありません。 
                pub room_id: uuid::Uuid,
            /// 問題のUUID。下記は契約用例示値であり、開始導線の実problem_idではありません。 
                pub problem_id: uuid::Uuid,
    }



    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
    #[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
    pub struct GetCurrentRunPathParams {
            /// 部屋のUUID。下記は契約用例示値であり、開始導線の実room_idではありません。 
                pub room_id: uuid::Uuid,
    }



    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
    #[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
    pub struct StartOrResumeRunPathParams {
            /// 部屋のUUID。下記は契約用例示値であり、開始導線の実room_idではありません。 
                pub room_id: uuid::Uuid,
    }





#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ActiveRunResponse {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "status")]
          #[validate(custom(function = "check_xss_string"))]
    pub status: String,

    #[serde(rename = "started_at")]
    pub started_at: chrono::DateTime::<chrono::Utc>,

    #[serde(rename = "elapsed_ms")]
    pub elapsed_ms: i32,

    #[serde(rename = "cleared_problem_ids")]
    pub cleared_problem_ids: Vec<uuid::Uuid>,

    #[serde(rename = "query_count")]
    pub query_count: i32,

}



impl ActiveRunResponse {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(status: String, started_at: chrono::DateTime::<chrono::Utc>, elapsed_ms: i32, cleared_problem_ids: Vec<uuid::Uuid>, query_count: i32, ) -> ActiveRunResponse {
        ActiveRunResponse {
 status,
 started_at,
 elapsed_ms,
 cleared_problem_ids,
 query_count,
        }
    }
}

/// Converts the ActiveRunResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for ActiveRunResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("status".to_string()),
            Some(self.status.to_string()),

            // Skipping started_at in query parameter serialization


            Some("elapsed_ms".to_string()),
            Some(self.elapsed_ms.to_string()),

            // Skipping cleared_problem_ids in query parameter serialization


            Some("query_count".to_string()),
            Some(self.query_count.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ActiveRunResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ActiveRunResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub status: Vec<String>,
            pub started_at: Vec<chrono::DateTime::<chrono::Utc>>,
            pub elapsed_ms: Vec<i32>,
            pub cleared_problem_ids: Vec<Vec<uuid::Uuid>>,
            pub query_count: Vec<i32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ActiveRunResponse".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep.status.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "started_at" => intermediate_rep.started_at.push(<chrono::DateTime::<chrono::Utc> as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "elapsed_ms" => intermediate_rep.elapsed_ms.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "cleared_problem_ids" => return std::result::Result::Err("Parsing a container in this style is not supported in ActiveRunResponse".to_string()),
                    #[allow(clippy::redundant_clone)]
                    "query_count" => intermediate_rep.query_count.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ActiveRunResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ActiveRunResponse {
            status: intermediate_rep.status.into_iter().next().ok_or_else(|| "status missing in ActiveRunResponse".to_string())?,
            started_at: intermediate_rep.started_at.into_iter().next().ok_or_else(|| "started_at missing in ActiveRunResponse".to_string())?,
            elapsed_ms: intermediate_rep.elapsed_ms.into_iter().next().ok_or_else(|| "elapsed_ms missing in ActiveRunResponse".to_string())?,
            cleared_problem_ids: intermediate_rep.cleared_problem_ids.into_iter().next().ok_or_else(|| "cleared_problem_ids missing in ActiveRunResponse".to_string())?,
            query_count: intermediate_rep.query_count.into_iter().next().ok_or_else(|| "query_count missing in ActiveRunResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ActiveRunResponse> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<ActiveRunResponse>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ActiveRunResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for ActiveRunResponse - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<ActiveRunResponse> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ActiveRunResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into ActiveRunResponse - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AnswerInputSchema {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
          #[validate(custom(function = "check_xss_string"))]
    pub r_type: String,

    #[serde(rename = "max_length")]
    pub max_length: i32,

}



impl AnswerInputSchema {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(r_type: String, max_length: i32, ) -> AnswerInputSchema {
        AnswerInputSchema {
 r_type,
 max_length,
        }
    }
}

/// Converts the AnswerInputSchema value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AnswerInputSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("type".to_string()),
            Some(self.r_type.to_string()),


            Some("max_length".to_string()),
            Some(self.max_length.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AnswerInputSchema value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AnswerInputSchema {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub r_type: Vec<String>,
            pub max_length: Vec<i32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing AnswerInputSchema".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "type" => intermediate_rep.r_type.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "max_length" => intermediate_rep.max_length.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing AnswerInputSchema".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AnswerInputSchema {
            r_type: intermediate_rep.r_type.into_iter().next().ok_or_else(|| "type missing in AnswerInputSchema".to_string())?,
            max_length: intermediate_rep.max_length.into_iter().next().ok_or_else(|| "max_length missing in AnswerInputSchema".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AnswerInputSchema> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AnswerInputSchema>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<AnswerInputSchema>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for AnswerInputSchema - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AnswerInputSchema> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <AnswerInputSchema as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into AnswerInputSchema - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct AnswerRequest {
    #[serde(rename = "answer")]
          #[validate(custom(function = "check_xss_string"))]
    pub answer: String,

}



impl AnswerRequest {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(answer: String, ) -> AnswerRequest {
        AnswerRequest {
 answer,
        }
    }
}

/// Converts the AnswerRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for AnswerRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("answer".to_string()),
            Some(self.answer.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AnswerRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AnswerRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub answer: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing AnswerRequest".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "answer" => intermediate_rep.answer.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing AnswerRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(AnswerRequest {
            answer: intermediate_rep.answer.into_iter().next().ok_or_else(|| "answer missing in AnswerRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<AnswerRequest> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<AnswerRequest>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<AnswerRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for AnswerRequest - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<AnswerRequest> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <AnswerRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into AnswerRequest - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[allow(non_camel_case_types, clippy::large_enum_variant)]
pub enum AnswerResponse {
    IncorrectAnswerResponse(models::IncorrectAnswerResponse),
    CorrectAnswerResponse(models::CorrectAnswerResponse),
}

impl validator::Validate for AnswerResponse
{
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        match self {
            Self::IncorrectAnswerResponse(v) => v.validate(),
            Self::CorrectAnswerResponse(v) => v.validate(),
        }
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a AnswerResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for AnswerResponse {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}


impl From<models::IncorrectAnswerResponse> for AnswerResponse {
    fn from(value: models::IncorrectAnswerResponse) -> Self {
        Self::IncorrectAnswerResponse(value)
    }
}
impl From<models::CorrectAnswerResponse> for AnswerResponse {
    fn from(value: models::CorrectAnswerResponse) -> Self {
        Self::CorrectAnswerResponse(value)
    }
}





#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Asset {
    #[serde(rename = "type")]
          #[validate(custom(function = "check_xss_string"))]
    pub r_type: String,

    #[serde(rename = "url")]
          #[validate(custom(function = "check_xss_string"))]
    pub url: String,

    #[serde(rename = "alt")]
          #[validate(custom(function = "check_xss_string"))]
    pub alt: String,

}



impl Asset {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(r_type: String, url: String, alt: String, ) -> Asset {
        Asset {
 r_type,
 url,
 alt,
        }
    }
}

/// Converts the Asset value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Asset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("type".to_string()),
            Some(self.r_type.to_string()),


            Some("url".to_string()),
            Some(self.url.to_string()),


            Some("alt".to_string()),
            Some(self.alt.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Asset value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Asset {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub r_type: Vec<String>,
            pub url: Vec<String>,
            pub alt: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Asset".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "type" => intermediate_rep.r_type.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "url" => intermediate_rep.url.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "alt" => intermediate_rep.alt.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Asset".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Asset {
            r_type: intermediate_rep.r_type.into_iter().next().ok_or_else(|| "type missing in Asset".to_string())?,
            url: intermediate_rep.url.into_iter().next().ok_or_else(|| "url missing in Asset".to_string())?,
            alt: intermediate_rep.alt.into_iter().next().ok_or_else(|| "alt missing in Asset".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Asset> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Asset>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Asset>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Asset - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Asset> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Asset as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Asset - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CorrectAnswerResponse {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "correct")]
    pub correct: bool,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "problem_status")]
          #[validate(custom(function = "check_xss_string"))]
    pub problem_status: String,

    #[serde(rename = "unlocked_problem_ids")]
    pub unlocked_problem_ids: Vec<uuid::Uuid>,

    #[serde(rename = "run_status")]
          #[validate(nested)]
    pub run_status: models::RunStatus,

    #[serde(rename = "progress")]
          #[validate(nested)]
    pub progress: models::Progress,

    #[serde(rename = "elapsed_ms")]
    pub elapsed_ms: i32,

}



impl CorrectAnswerResponse {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(correct: bool, problem_status: String, unlocked_problem_ids: Vec<uuid::Uuid>, run_status: models::RunStatus, progress: models::Progress, elapsed_ms: i32, ) -> CorrectAnswerResponse {
        CorrectAnswerResponse {
 correct,
 problem_status,
 unlocked_problem_ids,
 run_status,
 progress,
 elapsed_ms,
        }
    }
}

/// Converts the CorrectAnswerResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for CorrectAnswerResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("correct".to_string()),
            Some(self.correct.to_string()),


            Some("problem_status".to_string()),
            Some(self.problem_status.to_string()),

            // Skipping unlocked_problem_ids in query parameter serialization

            // Skipping run_status in query parameter serialization

            // Skipping progress in query parameter serialization


            Some("elapsed_ms".to_string()),
            Some(self.elapsed_ms.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CorrectAnswerResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CorrectAnswerResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub correct: Vec<bool>,
            pub problem_status: Vec<String>,
            pub unlocked_problem_ids: Vec<Vec<uuid::Uuid>>,
            pub run_status: Vec<models::RunStatus>,
            pub progress: Vec<models::Progress>,
            pub elapsed_ms: Vec<i32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing CorrectAnswerResponse".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "correct" => intermediate_rep.correct.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "problem_status" => intermediate_rep.problem_status.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "unlocked_problem_ids" => return std::result::Result::Err("Parsing a container in this style is not supported in CorrectAnswerResponse".to_string()),
                    #[allow(clippy::redundant_clone)]
                    "run_status" => intermediate_rep.run_status.push(<models::RunStatus as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "progress" => intermediate_rep.progress.push(<models::Progress as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "elapsed_ms" => intermediate_rep.elapsed_ms.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing CorrectAnswerResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CorrectAnswerResponse {
            correct: intermediate_rep.correct.into_iter().next().ok_or_else(|| "correct missing in CorrectAnswerResponse".to_string())?,
            problem_status: intermediate_rep.problem_status.into_iter().next().ok_or_else(|| "problem_status missing in CorrectAnswerResponse".to_string())?,
            unlocked_problem_ids: intermediate_rep.unlocked_problem_ids.into_iter().next().ok_or_else(|| "unlocked_problem_ids missing in CorrectAnswerResponse".to_string())?,
            run_status: intermediate_rep.run_status.into_iter().next().ok_or_else(|| "run_status missing in CorrectAnswerResponse".to_string())?,
            progress: intermediate_rep.progress.into_iter().next().ok_or_else(|| "progress missing in CorrectAnswerResponse".to_string())?,
            elapsed_ms: intermediate_rep.elapsed_ms.into_iter().next().ok_or_else(|| "elapsed_ms missing in CorrectAnswerResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CorrectAnswerResponse> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<CorrectAnswerResponse>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<CorrectAnswerResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for CorrectAnswerResponse - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<CorrectAnswerResponse> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <CorrectAnswerResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into CorrectAnswerResponse - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct CorrectQueryResponse {
    #[serde(rename = "query_id")]
    pub query_id: uuid::Uuid,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "correct")]
    pub correct: bool,

    #[serde(rename = "normalized_operations")]
          #[validate(nested)]
    pub normalized_operations: Vec<models::Operation>,

    #[serde(rename = "remaining_pattern_count")]
    pub remaining_pattern_count: i32,

    #[serde(rename = "query_count")]
    pub query_count: i32,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "problem_status")]
          #[validate(custom(function = "check_xss_string"))]
    pub problem_status: String,

}



impl CorrectQueryResponse {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(query_id: uuid::Uuid, correct: bool, normalized_operations: Vec<models::Operation>, remaining_pattern_count: i32, query_count: i32, problem_status: String, ) -> CorrectQueryResponse {
        CorrectQueryResponse {
 query_id,
 correct,
 normalized_operations,
 remaining_pattern_count,
 query_count,
 problem_status,
        }
    }
}

/// Converts the CorrectQueryResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for CorrectQueryResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping query_id in query parameter serialization


            Some("correct".to_string()),
            Some(self.correct.to_string()),

            // Skipping normalized_operations in query parameter serialization


            Some("remaining_pattern_count".to_string()),
            Some(self.remaining_pattern_count.to_string()),


            Some("query_count".to_string()),
            Some(self.query_count.to_string()),


            Some("problem_status".to_string()),
            Some(self.problem_status.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a CorrectQueryResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for CorrectQueryResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub query_id: Vec<uuid::Uuid>,
            pub correct: Vec<bool>,
            pub normalized_operations: Vec<Vec<models::Operation>>,
            pub remaining_pattern_count: Vec<i32>,
            pub query_count: Vec<i32>,
            pub problem_status: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing CorrectQueryResponse".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "query_id" => intermediate_rep.query_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "correct" => intermediate_rep.correct.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "normalized_operations" => return std::result::Result::Err("Parsing a container in this style is not supported in CorrectQueryResponse".to_string()),
                    #[allow(clippy::redundant_clone)]
                    "remaining_pattern_count" => intermediate_rep.remaining_pattern_count.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "query_count" => intermediate_rep.query_count.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "problem_status" => intermediate_rep.problem_status.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing CorrectQueryResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(CorrectQueryResponse {
            query_id: intermediate_rep.query_id.into_iter().next().ok_or_else(|| "query_id missing in CorrectQueryResponse".to_string())?,
            correct: intermediate_rep.correct.into_iter().next().ok_or_else(|| "correct missing in CorrectQueryResponse".to_string())?,
            normalized_operations: intermediate_rep.normalized_operations.into_iter().next().ok_or_else(|| "normalized_operations missing in CorrectQueryResponse".to_string())?,
            remaining_pattern_count: intermediate_rep.remaining_pattern_count.into_iter().next().ok_or_else(|| "remaining_pattern_count missing in CorrectQueryResponse".to_string())?,
            query_count: intermediate_rep.query_count.into_iter().next().ok_or_else(|| "query_count missing in CorrectQueryResponse".to_string())?,
            problem_status: intermediate_rep.problem_status.into_iter().next().ok_or_else(|| "problem_status missing in CorrectQueryResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<CorrectQueryResponse> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<CorrectQueryResponse>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<CorrectQueryResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for CorrectQueryResponse - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<CorrectQueryResponse> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <CorrectQueryResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into CorrectQueryResponse - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ErrorResponse {
    #[serde(rename = "error")]
          #[validate(nested)]
    pub error: models::ErrorResponseError,

}



impl ErrorResponse {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(error: models::ErrorResponseError, ) -> ErrorResponse {
        ErrorResponse {
 error,
        }
    }
}

/// Converts the ErrorResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping error in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ErrorResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ErrorResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub error: Vec<models::ErrorResponseError>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ErrorResponse".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "error" => intermediate_rep.error.push(<models::ErrorResponseError as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ErrorResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ErrorResponse {
            error: intermediate_rep.error.into_iter().next().ok_or_else(|| "error missing in ErrorResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ErrorResponse> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<ErrorResponse>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ErrorResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for ErrorResponse - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<ErrorResponse> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ErrorResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into ErrorResponse - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ErrorResponseError {
    #[serde(rename = "code")]
          #[validate(custom(function = "check_xss_string"))]
    pub code: String,

    #[serde(rename = "message")]
          #[validate(custom(function = "check_xss_string"))]
    pub message: String,

    #[serde(rename = "details")]
    pub details: crate::types::Object,

}



impl ErrorResponseError {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(code: String, message: String, details: crate::types::Object, ) -> ErrorResponseError {
        ErrorResponseError {
 code,
 message,
 details,
        }
    }
}

/// Converts the ErrorResponseError value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for ErrorResponseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("code".to_string()),
            Some(self.code.to_string()),


            Some("message".to_string()),
            Some(self.message.to_string()),

            // Skipping details in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ErrorResponseError value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ErrorResponseError {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub code: Vec<String>,
            pub message: Vec<String>,
            pub details: Vec<crate::types::Object>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ErrorResponseError".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "code" => intermediate_rep.code.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "message" => intermediate_rep.message.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "details" => intermediate_rep.details.push(<crate::types::Object as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ErrorResponseError".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ErrorResponseError {
            code: intermediate_rep.code.into_iter().next().ok_or_else(|| "code missing in ErrorResponseError".to_string())?,
            message: intermediate_rep.message.into_iter().next().ok_or_else(|| "message missing in ErrorResponseError".to_string())?,
            details: intermediate_rep.details.into_iter().next().ok_or_else(|| "details missing in ErrorResponseError".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ErrorResponseError> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<ErrorResponseError>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ErrorResponseError>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for ErrorResponseError - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<ErrorResponseError> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ErrorResponseError as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into ErrorResponseError - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct GuestLoginRequest {
    #[serde(rename = "display_name")]
          #[validate(custom(function = "check_xss_string"))]
    pub display_name: String,

}



impl GuestLoginRequest {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(display_name: String, ) -> GuestLoginRequest {
        GuestLoginRequest {
 display_name,
        }
    }
}

/// Converts the GuestLoginRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for GuestLoginRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("display_name".to_string()),
            Some(self.display_name.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a GuestLoginRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for GuestLoginRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub display_name: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing GuestLoginRequest".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "display_name" => intermediate_rep.display_name.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing GuestLoginRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(GuestLoginRequest {
            display_name: intermediate_rep.display_name.into_iter().next().ok_or_else(|| "display_name missing in GuestLoginRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<GuestLoginRequest> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<GuestLoginRequest>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<GuestLoginRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for GuestLoginRequest - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<GuestLoginRequest> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <GuestLoginRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into GuestLoginRequest - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct GuestLoginResponse {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "authenticated")]
    pub authenticated: bool,

    #[serde(rename = "user")]
          #[validate(nested)]
    pub user: models::User,

}



impl GuestLoginResponse {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(authenticated: bool, user: models::User, ) -> GuestLoginResponse {
        GuestLoginResponse {
 authenticated,
 user,
        }
    }
}

/// Converts the GuestLoginResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for GuestLoginResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("authenticated".to_string()),
            Some(self.authenticated.to_string()),

            // Skipping user in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a GuestLoginResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for GuestLoginResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub authenticated: Vec<bool>,
            pub user: Vec<models::User>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing GuestLoginResponse".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "authenticated" => intermediate_rep.authenticated.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "user" => intermediate_rep.user.push(<models::User as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing GuestLoginResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(GuestLoginResponse {
            authenticated: intermediate_rep.authenticated.into_iter().next().ok_or_else(|| "authenticated missing in GuestLoginResponse".to_string())?,
            user: intermediate_rep.user.into_iter().next().ok_or_else(|| "user missing in GuestLoginResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<GuestLoginResponse> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<GuestLoginResponse>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<GuestLoginResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for GuestLoginResponse - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<GuestLoginResponse> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <GuestLoginResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into GuestLoginResponse - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct IncorrectAnswerResponse {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "correct")]
    pub correct: bool,

    #[serde(rename = "answer_attempt_count")]
    pub answer_attempt_count: i32,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "problem_status")]
          #[validate(custom(function = "check_xss_string"))]
    pub problem_status: String,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "run_status")]
          #[validate(custom(function = "check_xss_string"))]
    pub run_status: String,

}



impl IncorrectAnswerResponse {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(correct: bool, answer_attempt_count: i32, problem_status: String, run_status: String, ) -> IncorrectAnswerResponse {
        IncorrectAnswerResponse {
 correct,
 answer_attempt_count,
 problem_status,
 run_status,
        }
    }
}

/// Converts the IncorrectAnswerResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for IncorrectAnswerResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("correct".to_string()),
            Some(self.correct.to_string()),


            Some("answer_attempt_count".to_string()),
            Some(self.answer_attempt_count.to_string()),


            Some("problem_status".to_string()),
            Some(self.problem_status.to_string()),


            Some("run_status".to_string()),
            Some(self.run_status.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a IncorrectAnswerResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for IncorrectAnswerResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub correct: Vec<bool>,
            pub answer_attempt_count: Vec<i32>,
            pub problem_status: Vec<String>,
            pub run_status: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing IncorrectAnswerResponse".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "correct" => intermediate_rep.correct.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "answer_attempt_count" => intermediate_rep.answer_attempt_count.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "problem_status" => intermediate_rep.problem_status.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "run_status" => intermediate_rep.run_status.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing IncorrectAnswerResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(IncorrectAnswerResponse {
            correct: intermediate_rep.correct.into_iter().next().ok_or_else(|| "correct missing in IncorrectAnswerResponse".to_string())?,
            answer_attempt_count: intermediate_rep.answer_attempt_count.into_iter().next().ok_or_else(|| "answer_attempt_count missing in IncorrectAnswerResponse".to_string())?,
            problem_status: intermediate_rep.problem_status.into_iter().next().ok_or_else(|| "problem_status missing in IncorrectAnswerResponse".to_string())?,
            run_status: intermediate_rep.run_status.into_iter().next().ok_or_else(|| "run_status missing in IncorrectAnswerResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<IncorrectAnswerResponse> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<IncorrectAnswerResponse>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<IncorrectAnswerResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for IncorrectAnswerResponse - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<IncorrectAnswerResponse> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <IncorrectAnswerResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into IncorrectAnswerResponse - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct IncorrectQueryResponse {
    #[serde(rename = "query_id")]
    pub query_id: uuid::Uuid,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "correct")]
    pub correct: bool,

    #[serde(rename = "normalized_operations")]
          #[validate(nested)]
    pub normalized_operations: Vec<models::Operation>,

    #[serde(rename = "remaining_pattern_count")]
    pub remaining_pattern_count: i32,

    #[serde(rename = "query_count")]
    pub query_count: i32,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "problem_status")]
          #[validate(custom(function = "check_xss_string"))]
    pub problem_status: String,

}



impl IncorrectQueryResponse {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(query_id: uuid::Uuid, correct: bool, normalized_operations: Vec<models::Operation>, remaining_pattern_count: i32, query_count: i32, problem_status: String, ) -> IncorrectQueryResponse {
        IncorrectQueryResponse {
 query_id,
 correct,
 normalized_operations,
 remaining_pattern_count,
 query_count,
 problem_status,
        }
    }
}

/// Converts the IncorrectQueryResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for IncorrectQueryResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping query_id in query parameter serialization


            Some("correct".to_string()),
            Some(self.correct.to_string()),

            // Skipping normalized_operations in query parameter serialization


            Some("remaining_pattern_count".to_string()),
            Some(self.remaining_pattern_count.to_string()),


            Some("query_count".to_string()),
            Some(self.query_count.to_string()),


            Some("problem_status".to_string()),
            Some(self.problem_status.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a IncorrectQueryResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for IncorrectQueryResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub query_id: Vec<uuid::Uuid>,
            pub correct: Vec<bool>,
            pub normalized_operations: Vec<Vec<models::Operation>>,
            pub remaining_pattern_count: Vec<i32>,
            pub query_count: Vec<i32>,
            pub problem_status: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing IncorrectQueryResponse".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "query_id" => intermediate_rep.query_id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "correct" => intermediate_rep.correct.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "normalized_operations" => return std::result::Result::Err("Parsing a container in this style is not supported in IncorrectQueryResponse".to_string()),
                    #[allow(clippy::redundant_clone)]
                    "remaining_pattern_count" => intermediate_rep.remaining_pattern_count.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "query_count" => intermediate_rep.query_count.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "problem_status" => intermediate_rep.problem_status.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing IncorrectQueryResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(IncorrectQueryResponse {
            query_id: intermediate_rep.query_id.into_iter().next().ok_or_else(|| "query_id missing in IncorrectQueryResponse".to_string())?,
            correct: intermediate_rep.correct.into_iter().next().ok_or_else(|| "correct missing in IncorrectQueryResponse".to_string())?,
            normalized_operations: intermediate_rep.normalized_operations.into_iter().next().ok_or_else(|| "normalized_operations missing in IncorrectQueryResponse".to_string())?,
            remaining_pattern_count: intermediate_rep.remaining_pattern_count.into_iter().next().ok_or_else(|| "remaining_pattern_count missing in IncorrectQueryResponse".to_string())?,
            query_count: intermediate_rep.query_count.into_iter().next().ok_or_else(|| "query_count missing in IncorrectQueryResponse".to_string())?,
            problem_status: intermediate_rep.problem_status.into_iter().next().ok_or_else(|| "problem_status missing in IncorrectQueryResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<IncorrectQueryResponse> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<IncorrectQueryResponse>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<IncorrectQueryResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for IncorrectQueryResponse - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<IncorrectQueryResponse> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <IncorrectQueryResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into IncorrectQueryResponse - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MeLocalAuthenticated {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "authenticated")]
    pub authenticated: bool,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "auth_mode")]
          #[validate(custom(function = "check_xss_string"))]
    pub auth_mode: String,

    #[serde(rename = "user")]
          #[validate(nested)]
    pub user: models::User,

    #[serde(rename = "login_url")]
          #[validate(nested)]
    pub login_url: models::Null,

    #[serde(rename = "logout_url")]
          #[validate(nested)]
    pub logout_url: models::Null,

}



impl MeLocalAuthenticated {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(authenticated: bool, auth_mode: String, user: models::User, login_url: models::Null, logout_url: models::Null, ) -> MeLocalAuthenticated {
        MeLocalAuthenticated {
 authenticated,
 auth_mode,
 user,
 login_url,
 logout_url,
        }
    }
}

/// Converts the MeLocalAuthenticated value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for MeLocalAuthenticated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("authenticated".to_string()),
            Some(self.authenticated.to_string()),


            Some("auth_mode".to_string()),
            Some(self.auth_mode.to_string()),

            // Skipping user in query parameter serialization

            // Skipping login_url in query parameter serialization

            // Skipping logout_url in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MeLocalAuthenticated value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MeLocalAuthenticated {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub authenticated: Vec<bool>,
            pub auth_mode: Vec<String>,
            pub user: Vec<models::User>,
            pub login_url: Vec<models::Null>,
            pub logout_url: Vec<models::Null>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing MeLocalAuthenticated".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "authenticated" => intermediate_rep.authenticated.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "auth_mode" => intermediate_rep.auth_mode.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "user" => intermediate_rep.user.push(<models::User as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "login_url" => intermediate_rep.login_url.push(<models::Null as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "logout_url" => intermediate_rep.logout_url.push(<models::Null as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing MeLocalAuthenticated".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MeLocalAuthenticated {
            authenticated: intermediate_rep.authenticated.into_iter().next().ok_or_else(|| "authenticated missing in MeLocalAuthenticated".to_string())?,
            auth_mode: intermediate_rep.auth_mode.into_iter().next().ok_or_else(|| "auth_mode missing in MeLocalAuthenticated".to_string())?,
            user: intermediate_rep.user.into_iter().next().ok_or_else(|| "user missing in MeLocalAuthenticated".to_string())?,
            login_url: intermediate_rep.login_url.into_iter().next().ok_or_else(|| "login_url missing in MeLocalAuthenticated".to_string())?,
            logout_url: intermediate_rep.logout_url.into_iter().next().ok_or_else(|| "logout_url missing in MeLocalAuthenticated".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MeLocalAuthenticated> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<MeLocalAuthenticated>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<MeLocalAuthenticated>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for MeLocalAuthenticated - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<MeLocalAuthenticated> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <MeLocalAuthenticated as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into MeLocalAuthenticated - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MeLocalUnauthenticated {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "authenticated")]
    pub authenticated: bool,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "auth_mode")]
          #[validate(custom(function = "check_xss_string"))]
    pub auth_mode: String,

    #[serde(rename = "user")]
          #[validate(nested)]
    pub user: models::Null,

    #[serde(rename = "login_url")]
          #[validate(nested)]
    pub login_url: models::Null,

    #[serde(rename = "logout_url")]
          #[validate(nested)]
    pub logout_url: models::Null,

}



impl MeLocalUnauthenticated {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(authenticated: bool, auth_mode: String, user: models::Null, login_url: models::Null, logout_url: models::Null, ) -> MeLocalUnauthenticated {
        MeLocalUnauthenticated {
 authenticated,
 auth_mode,
 user,
 login_url,
 logout_url,
        }
    }
}

/// Converts the MeLocalUnauthenticated value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for MeLocalUnauthenticated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("authenticated".to_string()),
            Some(self.authenticated.to_string()),


            Some("auth_mode".to_string()),
            Some(self.auth_mode.to_string()),

            // Skipping user in query parameter serialization

            // Skipping login_url in query parameter serialization

            // Skipping logout_url in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MeLocalUnauthenticated value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MeLocalUnauthenticated {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub authenticated: Vec<bool>,
            pub auth_mode: Vec<String>,
            pub user: Vec<models::Null>,
            pub login_url: Vec<models::Null>,
            pub logout_url: Vec<models::Null>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing MeLocalUnauthenticated".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "authenticated" => intermediate_rep.authenticated.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "auth_mode" => intermediate_rep.auth_mode.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "user" => intermediate_rep.user.push(<models::Null as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "login_url" => intermediate_rep.login_url.push(<models::Null as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "logout_url" => intermediate_rep.logout_url.push(<models::Null as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing MeLocalUnauthenticated".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MeLocalUnauthenticated {
            authenticated: intermediate_rep.authenticated.into_iter().next().ok_or_else(|| "authenticated missing in MeLocalUnauthenticated".to_string())?,
            auth_mode: intermediate_rep.auth_mode.into_iter().next().ok_or_else(|| "auth_mode missing in MeLocalUnauthenticated".to_string())?,
            user: intermediate_rep.user.into_iter().next().ok_or_else(|| "user missing in MeLocalUnauthenticated".to_string())?,
            login_url: intermediate_rep.login_url.into_iter().next().ok_or_else(|| "login_url missing in MeLocalUnauthenticated".to_string())?,
            logout_url: intermediate_rep.logout_url.into_iter().next().ok_or_else(|| "logout_url missing in MeLocalUnauthenticated".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MeLocalUnauthenticated> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<MeLocalUnauthenticated>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<MeLocalUnauthenticated>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for MeLocalUnauthenticated - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<MeLocalUnauthenticated> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <MeLocalUnauthenticated as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into MeLocalUnauthenticated - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MeNeoshowcaseAuthenticated {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "authenticated")]
    pub authenticated: bool,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "auth_mode")]
          #[validate(custom(function = "check_xss_string"))]
    pub auth_mode: String,

    #[serde(rename = "user")]
          #[validate(nested)]
    pub user: models::User,

    #[serde(rename = "login_url")]
          #[validate(nested)]
    pub login_url: models::Null,

    #[serde(rename = "logout_url")]
          #[validate(custom(function = "check_xss_string"))]
    pub logout_url: String,

}



impl MeNeoshowcaseAuthenticated {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(authenticated: bool, auth_mode: String, user: models::User, login_url: models::Null, logout_url: String, ) -> MeNeoshowcaseAuthenticated {
        MeNeoshowcaseAuthenticated {
 authenticated,
 auth_mode,
 user,
 login_url,
 logout_url,
        }
    }
}

/// Converts the MeNeoshowcaseAuthenticated value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for MeNeoshowcaseAuthenticated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("authenticated".to_string()),
            Some(self.authenticated.to_string()),


            Some("auth_mode".to_string()),
            Some(self.auth_mode.to_string()),

            // Skipping user in query parameter serialization

            // Skipping login_url in query parameter serialization


            Some("logout_url".to_string()),
            Some(self.logout_url.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MeNeoshowcaseAuthenticated value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MeNeoshowcaseAuthenticated {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub authenticated: Vec<bool>,
            pub auth_mode: Vec<String>,
            pub user: Vec<models::User>,
            pub login_url: Vec<models::Null>,
            pub logout_url: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing MeNeoshowcaseAuthenticated".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "authenticated" => intermediate_rep.authenticated.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "auth_mode" => intermediate_rep.auth_mode.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "user" => intermediate_rep.user.push(<models::User as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "login_url" => intermediate_rep.login_url.push(<models::Null as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "logout_url" => intermediate_rep.logout_url.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing MeNeoshowcaseAuthenticated".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MeNeoshowcaseAuthenticated {
            authenticated: intermediate_rep.authenticated.into_iter().next().ok_or_else(|| "authenticated missing in MeNeoshowcaseAuthenticated".to_string())?,
            auth_mode: intermediate_rep.auth_mode.into_iter().next().ok_or_else(|| "auth_mode missing in MeNeoshowcaseAuthenticated".to_string())?,
            user: intermediate_rep.user.into_iter().next().ok_or_else(|| "user missing in MeNeoshowcaseAuthenticated".to_string())?,
            login_url: intermediate_rep.login_url.into_iter().next().ok_or_else(|| "login_url missing in MeNeoshowcaseAuthenticated".to_string())?,
            logout_url: intermediate_rep.logout_url.into_iter().next().ok_or_else(|| "logout_url missing in MeNeoshowcaseAuthenticated".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MeNeoshowcaseAuthenticated> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<MeNeoshowcaseAuthenticated>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<MeNeoshowcaseAuthenticated>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for MeNeoshowcaseAuthenticated - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<MeNeoshowcaseAuthenticated> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <MeNeoshowcaseAuthenticated as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into MeNeoshowcaseAuthenticated - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct MeNeoshowcaseUnauthenticated {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "authenticated")]
    pub authenticated: bool,

    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "auth_mode")]
          #[validate(custom(function = "check_xss_string"))]
    pub auth_mode: String,

    #[serde(rename = "user")]
          #[validate(nested)]
    pub user: models::Null,

    #[serde(rename = "login_url")]
          #[validate(custom(function = "check_xss_string"))]
    pub login_url: String,

    #[serde(rename = "logout_url")]
          #[validate(nested)]
    pub logout_url: models::Null,

}



impl MeNeoshowcaseUnauthenticated {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(authenticated: bool, auth_mode: String, user: models::Null, login_url: String, logout_url: models::Null, ) -> MeNeoshowcaseUnauthenticated {
        MeNeoshowcaseUnauthenticated {
 authenticated,
 auth_mode,
 user,
 login_url,
 logout_url,
        }
    }
}

/// Converts the MeNeoshowcaseUnauthenticated value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for MeNeoshowcaseUnauthenticated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("authenticated".to_string()),
            Some(self.authenticated.to_string()),


            Some("auth_mode".to_string()),
            Some(self.auth_mode.to_string()),

            // Skipping user in query parameter serialization


            Some("login_url".to_string()),
            Some(self.login_url.to_string()),

            // Skipping logout_url in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MeNeoshowcaseUnauthenticated value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MeNeoshowcaseUnauthenticated {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub authenticated: Vec<bool>,
            pub auth_mode: Vec<String>,
            pub user: Vec<models::Null>,
            pub login_url: Vec<String>,
            pub logout_url: Vec<models::Null>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing MeNeoshowcaseUnauthenticated".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "authenticated" => intermediate_rep.authenticated.push(<bool as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "auth_mode" => intermediate_rep.auth_mode.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "user" => intermediate_rep.user.push(<models::Null as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "login_url" => intermediate_rep.login_url.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "logout_url" => intermediate_rep.logout_url.push(<models::Null as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing MeNeoshowcaseUnauthenticated".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(MeNeoshowcaseUnauthenticated {
            authenticated: intermediate_rep.authenticated.into_iter().next().ok_or_else(|| "authenticated missing in MeNeoshowcaseUnauthenticated".to_string())?,
            auth_mode: intermediate_rep.auth_mode.into_iter().next().ok_or_else(|| "auth_mode missing in MeNeoshowcaseUnauthenticated".to_string())?,
            user: intermediate_rep.user.into_iter().next().ok_or_else(|| "user missing in MeNeoshowcaseUnauthenticated".to_string())?,
            login_url: intermediate_rep.login_url.into_iter().next().ok_or_else(|| "login_url missing in MeNeoshowcaseUnauthenticated".to_string())?,
            logout_url: intermediate_rep.logout_url.into_iter().next().ok_or_else(|| "logout_url missing in MeNeoshowcaseUnauthenticated".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<MeNeoshowcaseUnauthenticated> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<MeNeoshowcaseUnauthenticated>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<MeNeoshowcaseUnauthenticated>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for MeNeoshowcaseUnauthenticated - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<MeNeoshowcaseUnauthenticated> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <MeNeoshowcaseUnauthenticated as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into MeNeoshowcaseUnauthenticated - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[allow(non_camel_case_types, clippy::large_enum_variant)]
pub enum MeResponse {
    MeNeoshowcaseAuthenticated(models::MeNeoshowcaseAuthenticated),
    MeNeoshowcaseUnauthenticated(models::MeNeoshowcaseUnauthenticated),
    MeLocalAuthenticated(models::MeLocalAuthenticated),
    MeLocalUnauthenticated(models::MeLocalUnauthenticated),
}

impl validator::Validate for MeResponse
{
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        match self {
            Self::MeNeoshowcaseAuthenticated(v) => v.validate(),
            Self::MeNeoshowcaseUnauthenticated(v) => v.validate(),
            Self::MeLocalAuthenticated(v) => v.validate(),
            Self::MeLocalUnauthenticated(v) => v.validate(),
        }
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a MeResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for MeResponse {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}


impl From<models::MeNeoshowcaseAuthenticated> for MeResponse {
    fn from(value: models::MeNeoshowcaseAuthenticated) -> Self {
        Self::MeNeoshowcaseAuthenticated(value)
    }
}
impl From<models::MeNeoshowcaseUnauthenticated> for MeResponse {
    fn from(value: models::MeNeoshowcaseUnauthenticated) -> Self {
        Self::MeNeoshowcaseUnauthenticated(value)
    }
}
impl From<models::MeLocalAuthenticated> for MeResponse {
    fn from(value: models::MeLocalAuthenticated) -> Self {
        Self::MeLocalAuthenticated(value)
    }
}
impl From<models::MeLocalUnauthenticated> for MeResponse {
    fn from(value: models::MeLocalUnauthenticated) -> Self {
        Self::MeLocalUnauthenticated(value)
    }
}





#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Operation {
    #[serde(rename = "control")]
          #[validate(custom(function = "check_xss_string"))]
    pub control: String,

    #[serde(rename = "count")]
    pub count: i32,

}



impl Operation {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(control: String, count: i32, ) -> Operation {
        Operation {
 control,
 count,
        }
    }
}

/// Converts the Operation value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("control".to_string()),
            Some(self.control.to_string()),


            Some("count".to_string()),
            Some(self.count.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Operation value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Operation {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub control: Vec<String>,
            pub count: Vec<i32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Operation".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "control" => intermediate_rep.control.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "count" => intermediate_rep.count.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Operation".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Operation {
            control: intermediate_rep.control.into_iter().next().ok_or_else(|| "control missing in Operation".to_string())?,
            count: intermediate_rep.count.into_iter().next().ok_or_else(|| "count missing in Operation".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Operation> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Operation>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Operation>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Operation - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Operation> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Operation as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Operation - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ProblemInputSchema {
    #[serde(rename = "query")]
          #[validate(nested)]
    pub query: models::QueryInputSchema,

    #[serde(rename = "answer")]
          #[validate(nested)]
    pub answer: models::AnswerInputSchema,

}



impl ProblemInputSchema {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(query: models::QueryInputSchema, answer: models::AnswerInputSchema, ) -> ProblemInputSchema {
        ProblemInputSchema {
 query,
 answer,
        }
    }
}

/// Converts the ProblemInputSchema value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for ProblemInputSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping query in query parameter serialization

            // Skipping answer in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ProblemInputSchema value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ProblemInputSchema {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub query: Vec<models::QueryInputSchema>,
            pub answer: Vec<models::AnswerInputSchema>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ProblemInputSchema".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "query" => intermediate_rep.query.push(<models::QueryInputSchema as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "answer" => intermediate_rep.answer.push(<models::AnswerInputSchema as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ProblemInputSchema".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ProblemInputSchema {
            query: intermediate_rep.query.into_iter().next().ok_or_else(|| "query missing in ProblemInputSchema".to_string())?,
            answer: intermediate_rep.answer.into_iter().next().ok_or_else(|| "answer missing in ProblemInputSchema".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ProblemInputSchema> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<ProblemInputSchema>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ProblemInputSchema>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for ProblemInputSchema - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<ProblemInputSchema> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ProblemInputSchema as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into ProblemInputSchema - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct ProblemResponse {
    #[serde(rename = "id")]
    pub id: uuid::Uuid,

    #[serde(rename = "number")]
    pub number: i32,

    #[serde(rename = "type")]
          #[validate(nested)]
    pub r_type: models::ProblemType,

    #[serde(rename = "title")]
          #[validate(custom(function = "check_xss_string"))]
    pub title: String,

    #[serde(rename = "body_markdown")]
          #[validate(custom(function = "check_xss_string"))]
    pub body_markdown: String,

    #[serde(rename = "submission_type")]
          #[validate(nested)]
    pub submission_type: models::SubmissionType,

    #[serde(rename = "assets")]
          #[validate(nested)]
    pub assets: Vec<models::Asset>,

    #[serde(rename = "status")]
          #[validate(nested)]
    pub status: models::ProblemStatus,

    #[serde(rename = "input_schema")]
          #[validate(nested)]
    pub input_schema: models::ProblemInputSchema,

    #[serde(rename = "hint_count")]
    pub hint_count: i32,

}



impl ProblemResponse {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(id: uuid::Uuid, number: i32, r_type: models::ProblemType, title: String, body_markdown: String, submission_type: models::SubmissionType, assets: Vec<models::Asset>, status: models::ProblemStatus, input_schema: models::ProblemInputSchema, hint_count: i32, ) -> ProblemResponse {
        ProblemResponse {
 id,
 number,
 r_type,
 title,
 body_markdown,
 submission_type,
 assets,
 status,
 input_schema,
 hint_count,
        }
    }
}

/// Converts the ProblemResponse value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for ProblemResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping id in query parameter serialization


            Some("number".to_string()),
            Some(self.number.to_string()),

            // Skipping type in query parameter serialization


            Some("title".to_string()),
            Some(self.title.to_string()),


            Some("body_markdown".to_string()),
            Some(self.body_markdown.to_string()),

            // Skipping submission_type in query parameter serialization

            // Skipping assets in query parameter serialization

            // Skipping status in query parameter serialization

            // Skipping input_schema in query parameter serialization


            Some("hint_count".to_string()),
            Some(self.hint_count.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a ProblemResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for ProblemResponse {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub id: Vec<uuid::Uuid>,
            pub number: Vec<i32>,
            pub r_type: Vec<models::ProblemType>,
            pub title: Vec<String>,
            pub body_markdown: Vec<String>,
            pub submission_type: Vec<models::SubmissionType>,
            pub assets: Vec<Vec<models::Asset>>,
            pub status: Vec<models::ProblemStatus>,
            pub input_schema: Vec<models::ProblemInputSchema>,
            pub hint_count: Vec<i32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing ProblemResponse".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "id" => intermediate_rep.id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "number" => intermediate_rep.number.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "type" => intermediate_rep.r_type.push(<models::ProblemType as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "title" => intermediate_rep.title.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "body_markdown" => intermediate_rep.body_markdown.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "submission_type" => intermediate_rep.submission_type.push(<models::SubmissionType as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "assets" => return std::result::Result::Err("Parsing a container in this style is not supported in ProblemResponse".to_string()),
                    #[allow(clippy::redundant_clone)]
                    "status" => intermediate_rep.status.push(<models::ProblemStatus as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "input_schema" => intermediate_rep.input_schema.push(<models::ProblemInputSchema as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "hint_count" => intermediate_rep.hint_count.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing ProblemResponse".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(ProblemResponse {
            id: intermediate_rep.id.into_iter().next().ok_or_else(|| "id missing in ProblemResponse".to_string())?,
            number: intermediate_rep.number.into_iter().next().ok_or_else(|| "number missing in ProblemResponse".to_string())?,
            r_type: intermediate_rep.r_type.into_iter().next().ok_or_else(|| "type missing in ProblemResponse".to_string())?,
            title: intermediate_rep.title.into_iter().next().ok_or_else(|| "title missing in ProblemResponse".to_string())?,
            body_markdown: intermediate_rep.body_markdown.into_iter().next().ok_or_else(|| "body_markdown missing in ProblemResponse".to_string())?,
            submission_type: intermediate_rep.submission_type.into_iter().next().ok_or_else(|| "submission_type missing in ProblemResponse".to_string())?,
            assets: intermediate_rep.assets.into_iter().next().ok_or_else(|| "assets missing in ProblemResponse".to_string())?,
            status: intermediate_rep.status.into_iter().next().ok_or_else(|| "status missing in ProblemResponse".to_string())?,
            input_schema: intermediate_rep.input_schema.into_iter().next().ok_or_else(|| "input_schema missing in ProblemResponse".to_string())?,
            hint_count: intermediate_rep.hint_count.into_iter().next().ok_or_else(|| "hint_count missing in ProblemResponse".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<ProblemResponse> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<ProblemResponse>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<ProblemResponse>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for ProblemResponse - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<ProblemResponse> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <ProblemResponse as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into ProblemResponse - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types, clippy::large_enum_variant)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum ProblemStatus {
    #[serde(rename = "locked")]
    Locked,
    #[serde(rename = "available")]
    Available,
    #[serde(rename = "cleared")]
    Cleared,
}

impl validator::Validate for ProblemStatus
{
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::fmt::Display for ProblemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ProblemStatus::Locked => write!(f, "locked"),
            ProblemStatus::Available => write!(f, "available"),
            ProblemStatus::Cleared => write!(f, "cleared"),
        }
    }
}

impl std::str::FromStr for ProblemStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "locked" => std::result::Result::Ok(ProblemStatus::Locked),
            "available" => std::result::Result::Ok(ProblemStatus::Available),
            "cleared" => std::result::Result::Ok(ProblemStatus::Cleared),
            _ => std::result::Result::Err(format!(r#"Value not valid: {s}"#)),
        }
    }
}


/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types, clippy::large_enum_variant)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum ProblemType {
    #[serde(rename = "small")]
    Small,
    #[serde(rename = "final")]
    Final,
}

impl validator::Validate for ProblemType
{
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::fmt::Display for ProblemType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ProblemType::Small => write!(f, "small"),
            ProblemType::Final => write!(f, "final"),
        }
    }
}

impl std::str::FromStr for ProblemType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "small" => std::result::Result::Ok(ProblemType::Small),
            "final" => std::result::Result::Ok(ProblemType::Final),
            _ => std::result::Result::Err(format!(r#"Value not valid: {s}"#)),
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct Progress {
    #[serde(rename = "cleared_problem_count")]
    pub cleared_problem_count: i32,

    #[serde(rename = "total_problem_count")]
    pub total_problem_count: i32,

}



impl Progress {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(cleared_problem_count: i32, total_problem_count: i32, ) -> Progress {
        Progress {
 cleared_problem_count,
 total_problem_count,
        }
    }
}

/// Converts the Progress value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for Progress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("cleared_problem_count".to_string()),
            Some(self.cleared_problem_count.to_string()),


            Some("total_problem_count".to_string()),
            Some(self.total_problem_count.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a Progress value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for Progress {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub cleared_problem_count: Vec<i32>,
            pub total_problem_count: Vec<i32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing Progress".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "cleared_problem_count" => intermediate_rep.cleared_problem_count.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "total_problem_count" => intermediate_rep.total_problem_count.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing Progress".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(Progress {
            cleared_problem_count: intermediate_rep.cleared_problem_count.into_iter().next().ok_or_else(|| "cleared_problem_count missing in Progress".to_string())?,
            total_problem_count: intermediate_rep.total_problem_count.into_iter().next().ok_or_else(|| "total_problem_count missing in Progress".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<Progress> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<Progress>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<Progress>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for Progress - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<Progress> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <Progress as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into Progress - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct QueryInputSchema {
    /// Note: inline enums are not fully supported by openapi-generator
    #[serde(rename = "type")]
          #[validate(custom(function = "check_xss_string"))]
    pub r_type: String,

    #[serde(rename = "allowed_controls")]
          #[validate(custom(function = "check_xss_vec_string"))]
    pub allowed_controls: Vec<String>,

    #[serde(rename = "max_operations")]
    pub max_operations: i32,

}



impl QueryInputSchema {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(r_type: String, allowed_controls: Vec<String>, max_operations: i32, ) -> QueryInputSchema {
        QueryInputSchema {
 r_type,
 allowed_controls,
 max_operations,
        }
    }
}

/// Converts the QueryInputSchema value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for QueryInputSchema {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("type".to_string()),
            Some(self.r_type.to_string()),


            Some("allowed_controls".to_string()),
            Some(self.allowed_controls.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")),


            Some("max_operations".to_string()),
            Some(self.max_operations.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a QueryInputSchema value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for QueryInputSchema {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub r_type: Vec<String>,
            pub allowed_controls: Vec<Vec<String>>,
            pub max_operations: Vec<i32>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing QueryInputSchema".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "type" => intermediate_rep.r_type.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "allowed_controls" => return std::result::Result::Err("Parsing a container in this style is not supported in QueryInputSchema".to_string()),
                    #[allow(clippy::redundant_clone)]
                    "max_operations" => intermediate_rep.max_operations.push(<i32 as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing QueryInputSchema".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(QueryInputSchema {
            r_type: intermediate_rep.r_type.into_iter().next().ok_or_else(|| "type missing in QueryInputSchema".to_string())?,
            allowed_controls: intermediate_rep.allowed_controls.into_iter().next().ok_or_else(|| "allowed_controls missing in QueryInputSchema".to_string())?,
            max_operations: intermediate_rep.max_operations.into_iter().next().ok_or_else(|| "max_operations missing in QueryInputSchema".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<QueryInputSchema> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<QueryInputSchema>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<QueryInputSchema>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for QueryInputSchema - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<QueryInputSchema> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <QueryInputSchema as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into QueryInputSchema - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct QueryRequest {
    #[serde(rename = "source")]
          #[validate(custom(function = "check_xss_string"))]
    pub source: String,

    #[serde(rename = "operations")]
          #[validate(nested)]
    pub operations: Vec<models::Operation>,

}



impl QueryRequest {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(source: String, operations: Vec<models::Operation>, ) -> QueryRequest {
        QueryRequest {
 source,
 operations,
        }
    }
}

/// Converts the QueryRequest value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for QueryRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![

            Some("source".to_string()),
            Some(self.source.to_string()),

            // Skipping operations in query parameter serialization

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a QueryRequest value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for QueryRequest {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub source: Vec<String>,
            pub operations: Vec<Vec<models::Operation>>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing QueryRequest".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "source" => intermediate_rep.source.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    "operations" => return std::result::Result::Err("Parsing a container in this style is not supported in QueryRequest".to_string()),
                    _ => return std::result::Result::Err("Unexpected key while parsing QueryRequest".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(QueryRequest {
            source: intermediate_rep.source.into_iter().next().ok_or_else(|| "source missing in QueryRequest".to_string())?,
            operations: intermediate_rep.operations.into_iter().next().ok_or_else(|| "operations missing in QueryRequest".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<QueryRequest> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<QueryRequest>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<QueryRequest>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for QueryRequest - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<QueryRequest> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <QueryRequest as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into QueryRequest - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}



#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
#[allow(non_camel_case_types, clippy::large_enum_variant)]
pub enum QueryResponse {
    IncorrectQueryResponse(models::IncorrectQueryResponse),
    CorrectQueryResponse(models::CorrectQueryResponse),
}

impl validator::Validate for QueryResponse
{
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        match self {
            Self::IncorrectQueryResponse(v) => v.validate(),
            Self::CorrectQueryResponse(v) => v.validate(),
        }
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a QueryResponse value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for QueryResponse {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}


impl From<models::IncorrectQueryResponse> for QueryResponse {
    fn from(value: models::IncorrectQueryResponse) -> Self {
        Self::IncorrectQueryResponse(value)
    }
}
impl From<models::CorrectQueryResponse> for QueryResponse {
    fn from(value: models::CorrectQueryResponse) -> Self {
        Self::CorrectQueryResponse(value)
    }
}





/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types, clippy::large_enum_variant)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum RunStatus {
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "cleared")]
    Cleared,
}

impl validator::Validate for RunStatus
{
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::fmt::Display for RunStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            RunStatus::Active => write!(f, "active"),
            RunStatus::Cleared => write!(f, "cleared"),
        }
    }
}

impl std::str::FromStr for RunStatus {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "active" => std::result::Result::Ok(RunStatus::Active),
            "cleared" => std::result::Result::Ok(RunStatus::Cleared),
            _ => std::result::Result::Err(format!(r#"Value not valid: {s}"#)),
        }
    }
}


/// Enumeration of values.
/// Since this enum's variants do not hold data, we can easily define them as `#[repr(C)]`
/// which helps with FFI.
#[allow(non_camel_case_types, clippy::large_enum_variant)]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "conversion", derive(frunk_enum_derive::LabelledGenericEnum))]
pub enum SubmissionType {
    #[serde(rename = "operation_sequence")]
    OperationSequence,
    #[serde(rename = "string")]
    String,
}

impl validator::Validate for SubmissionType
{
    fn validate(&self) -> std::result::Result<(), validator::ValidationErrors> {
        std::result::Result::Ok(())
    }
}

impl std::fmt::Display for SubmissionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            SubmissionType::OperationSequence => write!(f, "operation_sequence"),
            SubmissionType::String => write!(f, "string"),
        }
    }
}

impl std::str::FromStr for SubmissionType {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "operation_sequence" => std::result::Result::Ok(SubmissionType::OperationSequence),
            "string" => std::result::Result::Ok(SubmissionType::String),
            _ => std::result::Result::Err(format!(r#"Value not valid: {s}"#)),
        }
    }
}


#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, validator::Validate)]
#[cfg_attr(feature = "conversion", derive(frunk::LabelledGeneric))]
pub struct User {
    #[serde(rename = "id")]
    pub id: uuid::Uuid,

    #[serde(rename = "display_name")]
          #[validate(custom(function = "check_xss_string"))]
    pub display_name: String,

}



impl User {
    #[allow(clippy::new_without_default, clippy::too_many_arguments)]
    pub fn new(id: uuid::Uuid, display_name: String, ) -> User {
        User {
 id,
 display_name,
        }
    }
}

/// Converts the User value to the Query Parameters representation (style=form, explode=false)
/// specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde serializer
impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let params: Vec<Option<String>> = vec![
            // Skipping id in query parameter serialization


            Some("display_name".to_string()),
            Some(self.display_name.to_string()),

        ];

        write!(f, "{}", params.into_iter().flatten().collect::<Vec<_>>().join(","))
    }
}

/// Converts Query Parameters representation (style=form, explode=false) to a User value
/// as specified in https://swagger.io/docs/specification/serialization/
/// Should be implemented in a serde deserializer
impl std::str::FromStr for User {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        /// An intermediate representation of the struct to use for parsing.
        #[derive(Default)]
        #[allow(dead_code)]
        struct IntermediateRep {
            pub id: Vec<uuid::Uuid>,
            pub display_name: Vec<String>,
        }

        let mut intermediate_rep = IntermediateRep::default();

        // Parse into intermediate representation
        let mut string_iter = s.split(',');
        let mut key_result = string_iter.next();

        while key_result.is_some() {
            let val = match string_iter.next() {
                Some(x) => x,
                None => return std::result::Result::Err("Missing value while parsing User".to_string())
            };

            if let Some(key) = key_result {
                #[allow(clippy::match_single_binding)]
                match key {
                    #[allow(clippy::redundant_clone)]
                    "id" => intermediate_rep.id.push(<uuid::Uuid as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    #[allow(clippy::redundant_clone)]
                    "display_name" => intermediate_rep.display_name.push(<String as std::str::FromStr>::from_str(val).map_err(|x| x.to_string())?),
                    _ => return std::result::Result::Err("Unexpected key while parsing User".to_string())
                }
            }

            // Get the next key
            key_result = string_iter.next();
        }

        // Use the intermediate representation to return the struct
        std::result::Result::Ok(User {
            id: intermediate_rep.id.into_iter().next().ok_or_else(|| "id missing in User".to_string())?,
            display_name: intermediate_rep.display_name.into_iter().next().ok_or_else(|| "display_name missing in User".to_string())?,
        })
    }
}

// Methods for converting between header::IntoHeaderValue<User> and HeaderValue

#[cfg(feature = "server")]
impl std::convert::TryFrom<header::IntoHeaderValue<User>> for HeaderValue {
    type Error = String;

    fn try_from(hdr_value: header::IntoHeaderValue<User>) -> std::result::Result<Self, Self::Error> {
        let hdr_value = hdr_value.to_string();
        match HeaderValue::from_str(&hdr_value) {
             std::result::Result::Ok(value) => std::result::Result::Ok(value),
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Invalid header value for User - value: {hdr_value} is invalid {e}"#))
        }
    }
}

#[cfg(feature = "server")]
impl std::convert::TryFrom<HeaderValue> for header::IntoHeaderValue<User> {
    type Error = String;

    fn try_from(hdr_value: HeaderValue) -> std::result::Result<Self, Self::Error> {
        match hdr_value.to_str() {
             std::result::Result::Ok(value) => {
                    match <User as std::str::FromStr>::from_str(value) {
                        std::result::Result::Ok(value) => std::result::Result::Ok(header::IntoHeaderValue(value)),
                        std::result::Result::Err(err) => std::result::Result::Err(format!(r#"Unable to convert header value '{value}' into User - {err}"#))
                    }
             },
             std::result::Result::Err(e) => std::result::Result::Err(format!(r#"Unable to convert header: {hdr_value:?} to string: {e}"#))
        }
    }
}


