use openapiv3 as oa;
use openapiv3::{RefOr, Schema, SchemaKind, Type};

use crate::{OaParameter, OaSchema};

impl<T> OaSchema for http::Response<T> {
    fn schema() -> Schema {
        Schema::new_any()
    }
}

impl<T: OaSchema> OaParameter for axum::extract::Json<T> {
    fn body_schema() -> Option<RefOr<Schema>> {
        T::body_schema()
    }

    fn extractor_responses() -> Vec<(u16, String)> {
        vec![
            (
                400,
                "The request body could not be parsed as JSON.".to_string(),
            ),
            (
                422,
                "The request body is valid JSON but does not match the expected schema."
                    .to_string(),
            ),
        ]
    }
}
impl<T> OaParameter for axum::extract::Extension<T> {}
impl<T> OaParameter for axum::extract::State<T> {}
impl<T> OaParameter for http::Request<T> {}
impl<T> OaParameter for axum::extract::ConnectInfo<T> {}
impl OaParameter for http::HeaderMap {}
impl OaParameter for http::request::Parts {}

/// Expands object schemas into one query parameter per property, marking each
/// parameter required when the schema lists it in its `required` set.
fn query_parameters(schemas: Vec<RefOr<Schema>>) -> Vec<RefOr<oa::Parameter>> {
    schemas
        .into_iter()
        .flat_map(|s| s.into_item())
        .flat_map(|s| match s.kind {
            // Only object schemas decompose into named query parameters.
            SchemaKind::Type(Type::Object(o)) => {
                let required = o.required;
                Some(
                    o.properties
                        .into_iter()
                        .map(move |(k, v)| (required.contains(&k), k, v)),
                )
            }
            _ => None,
        })
        .flatten()
        .map(|(is_required, k, v)| {
            // Query strings have no concept of JSON `null`; optionality is
            // expressed by the parameter being non-required, not by a nullable
            // schema. Strip any `nullable` flag so an `Option<T>` field renders
            // the inner type's schema.
            let v = match v {
                RefOr::Item(mut schema) => {
                    schema.nullable = false;
                    RefOr::Item(schema)
                }
                other => other,
            };
            let mut parameter = oa::Parameter::query(k, v);
            parameter.required = is_required;
            RefOr::Item(parameter)
        })
        .collect()
}

impl<T: OaParameter> OaParameter for axum::extract::Query<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        query_parameters(T::parameter_schemas())
    }
}

impl<T: OaParameter> OaParameter for axum::extract::Path<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        T::parameter_schemas()
            .into_iter()
            .map(|s| RefOr::Item(oa::Parameter::path("path", s)))
            .collect()
    }
}

#[cfg(feature = "qs")]
impl<T: OaParameter> OaParameter for serde_qs::axum::QsQuery<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        query_parameters(T::parameter_schemas())
    }
}
