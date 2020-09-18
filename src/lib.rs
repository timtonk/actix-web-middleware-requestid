//! # actix-web-middleware-requestid
//!
//! Request ID middleware for the actix-web framework v3.0+. Adds a custom header with a unique token to every request.
//!
//! # Usage
//!
//! Add the package to Cargo.toml:
//!
//! ```toml
//! [dependencies]
//! actix-web-middleware-requestid = "3.0"
//! ```
//!
//! Import and add middleware to your server definition:
//!
//! ```rust
//! use actix_web_middleware_requestid::RequestID;
//!
//! ...
//!
//! App::new()
//!     ...
//!     .wrap(RequestID)
//! ```
//!
//! For actix-web v1.x use version "1.0" of the same package. The usage pattern and all exported names remain the same.
//!
//! # For actix-web < 1.0
//!
//! Consider using a similar crate [actix-web-requestid](https://crates.io/crates/actix-web-requestid)
//!

use std::task::{Context, Poll};

use actix_service::{Service, Transform};
use actix_web::error::ErrorBadRequest;
use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::Result;
use actix_web::{dev, Error, FromRequest, HttpMessage, HttpRequest};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse};
use futures::future::{err, ok, Ready};

/// The header set by the middleware
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// Request ID wrapper.
pub struct RequestIDWrapper;

impl<S, B> Transform<S> for RequestIDWrapper
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIDMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestIDMiddleware { service })
    }
}

/// Actual actix-web middleware
pub struct RequestIDMiddleware<S> {
    service: S,
}

impl<S, B> Service for RequestIDMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        use rand::{distributions::Alphanumeric, thread_rng, Rng};

        // generate request id token
        let request_id: String = thread_rng().sample_iter(&Alphanumeric).take(10).collect();

        // make object mutable (required as the header must be used inside `.call`)
        let mut req = req;

        // add request id header (for using in the log wrapper)
        req.headers_mut().append(
            HeaderName::from_static(REQUEST_ID_HEADER),
            HeaderValue::from_str(&request_id).unwrap(),
        );

        // add request id extension (for extractor)
        req.extensions_mut().insert(RequestID(request_id));

        // propagate the call
        self.service.call(req)
    }
}

/// Request ID extractor
pub struct RequestID(pub String);

impl FromRequest for RequestID {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        if let Some(RequestID(req_id)) = req.extensions().get::<RequestID>() {
            ok(RequestID(req_id.clone()))
        } else {
            err(ErrorBadRequest("request id is missing"))
        }
    }
}
