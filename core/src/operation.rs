use openapiv3::{Operation, Parameter, RefOr, Response, Schema, StatusCode};

pub struct OperationRegister {
    pub name: &'static str,
    pub constructor: &'static (dyn Sync + Send + Fn() -> Operation),
}

/// Extends [`openapiv3::Operation`] with insertion helpers that never clobber a
/// response the handler already documents.
pub trait OperationExt {
    /// Records a bodyless response for `status`, leaving any existing entry for
    /// that status code untouched.
    fn add_response_if_absent(&mut self, status: u16, description: String);

    /// Records a 200 response that carries no body, for handlers that return
    /// nothing on success.
    fn add_response_success_empty(&mut self);
}

impl OperationExt for Operation {
    fn add_response_if_absent(&mut self, status: u16, description: String) {
        self.responses
            .responses
            .entry(StatusCode::Code(status))
            .or_insert_with(|| {
                RefOr::Item(Response {
                    description,
                    ..Response::default()
                })
            });
    }

    fn add_response_success_empty(&mut self) {
        self.responses.responses.insert(
            StatusCode::Code(200),
            RefOr::Item(Response {
                description: "OK".to_string(),
                ..Response::default()
            }),
        );
    }
}

pub trait OaParameter {
    fn body_schema() -> Option<RefOr<Schema>> {
        None
    }
    fn parameter_schemas() -> Vec<RefOr<Schema>> {
        Vec::new()
    }
    fn parameters() -> Vec<RefOr<Parameter>> {
        Vec::new()
    }
    /// Status codes this type's extractor can reject a request with before the
    /// handler body ever runs, paired with a description of the rejection.
    fn extractor_responses() -> Vec<(u16, String)> {
        Vec::new()
    }
}

impl<T, E> OaParameter for Result<T, E>
where
    T: OaParameter,
{
    fn body_schema() -> Option<RefOr<Schema>> {
        T::body_schema()
    }
}

inventory::collect!(OperationRegister);
