use openapiv3 as oa;
use openapiv3::{RefOr, Schema, SchemaKind, Type};

use crate::{OaParameter, OaSchema};

impl<T: OaSchema> OaParameter for actix_web::web::Json<T> {
    fn body_schema() -> Option<RefOr<Schema>> {
        T::body_schema()
    }
}

impl OaSchema for actix_web::HttpResponse {
    fn schema() -> Schema {
        Schema::new_any()
    }
}

impl<T> OaParameter for actix_web::web::Data<T> {}
impl OaParameter for actix_web::HttpRequest {}

impl<T: OaParameter> OaParameter for actix_web::web::Path<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        T::parameter_schemas()
            .into_iter()
            .map(|s| RefOr::Item(oa::Parameter::path("path", s)))
            .collect()
    }
}

impl<T: OaParameter> OaParameter for actix_web::web::Query<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        T::parameter_schemas()
            .into_iter()
            .flat_map(|s| s.into_item())
            .flat_map(|s| match s.kind {
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
                let mut parameter = oa::Parameter::query(k, v);
                parameter.required = is_required;
                RefOr::Item(parameter)
            })
            .collect()
    }
}

#[cfg(feature = "qs")]
impl<T: OaParameter> OaParameter for serde_qs::actix::QsQuery<T> {
    fn parameters() -> Vec<RefOr<oa::Parameter>> {
        T::parameter_schemas()
            .into_iter()
            .map(|s| RefOr::Item(oa::Parameter::query("query", s)))
            .collect()
    }
}
