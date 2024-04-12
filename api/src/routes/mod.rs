use portfolio_core::error::ServiceError;
use rocket::{response::status::Custom, http::Status};

pub mod admin;
pub mod candidate;

pub fn to_custom_error(e: ServiceError) -> (rocket::http::Status, String) {
    if e.code() == 500 {
        warn!("Internal server error: {} ({})", e, e.inner_trace().unwrap_or("".to_string()));
    }

    (
        Status::from_code(e.code()).unwrap_or_default(),
        e.to_string()
    )
}