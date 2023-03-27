use axum::http::Request;
use helper::uuid;
use helper::uuid::ToBase62;
pub use tower_http::request_id::*;

#[derive(Clone, Debug)]
pub(crate) struct MakeRequestBase62Uuid;

impl MakeRequestId for MakeRequestBase62Uuid {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        uuid::new_v4().to_base62().parse().ok().map(RequestId::new)
    }
}
