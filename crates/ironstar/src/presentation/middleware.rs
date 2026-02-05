//! Request correlation middleware for HTTP request tracing.
//!
//! Provides a UUID v7 request ID generator for `tower-http`'s request ID
//! middleware. Each incoming request receives a time-ordered unique identifier
//! that is:
//!
//! 1. Set in the `x-request-id` request header
//! 2. Propagated to the `x-request-id` response header
//! 3. Available in request extensions as `tower_http::request_id::RequestId`
//! 4. Included in the tracing span context for structured logging
//!
//! # Request ID precedence
//!
//! If the incoming request already carries an `x-request-id` header (e.g.,
//! from an upstream proxy or gateway), the existing value is preserved and
//! propagated. New IDs are generated only when the header is absent.
//!
//! # UUID v7
//!
//! UUID v7 is used instead of v4 because its time-ordered prefix enables
//! natural chronological sorting of request IDs in log analysis tools.

use tower_http::request_id::MakeRequestId;
use uuid::Uuid;

/// Generates UUID v7 request identifiers.
///
/// Implements `tower_http::request_id::MakeRequestId` to integrate with
/// `SetRequestIdLayer`. UUID v7 provides time-ordered identifiers that
/// sort chronologically, making log analysis and correlation easier.
#[derive(Clone, Copy, Debug, Default)]
pub struct MakeRequestUuidV7;

impl MakeRequestId for MakeRequestUuidV7 {
    fn make_request_id<B>(
        &mut self,
        _request: &http::Request<B>,
    ) -> Option<tower_http::request_id::RequestId> {
        let id = Uuid::now_v7();
        let header_value = http::HeaderValue::from_str(&id.to_string()).ok()?;
        Some(tower_http::request_id::RequestId::new(header_value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tower_http::request_id::MakeRequestId;

    #[test]
    fn generates_valid_uuid_v7() {
        let mut maker = MakeRequestUuidV7;
        let request = http::Request::builder().body(()).expect("test request");

        let id = maker.make_request_id(&request);
        assert!(id.is_some());

        let header = id.expect("id is some").header_value().clone();
        let parsed = Uuid::parse_str(header.to_str().expect("valid str"));
        assert!(parsed.is_ok());

        let uuid = parsed.expect("valid uuid");
        assert_eq!(uuid.get_version(), Some(uuid::Version::SortRand));
    }

    #[test]
    fn generates_unique_ids() {
        let mut maker = MakeRequestUuidV7;
        let request = http::Request::builder().body(()).expect("test request");

        let id1 = maker
            .make_request_id(&request)
            .expect("id1")
            .header_value()
            .clone();
        let id2 = maker
            .make_request_id(&request)
            .expect("id2")
            .header_value()
            .clone();

        assert_ne!(id1, id2);
    }
}
