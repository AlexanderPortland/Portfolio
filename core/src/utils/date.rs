use alohomora::{bbox::BBox, policy::NoPolicy};
use chrono::NaiveDate;

use crate::error::ServiceError;

pub fn parse_naive_date_from_opt_str(date: Option<BBox<String, NoPolicy>>, fmt: &str) -> Result<NaiveDate, ServiceError> {
    Ok(
        NaiveDate::parse_from_str(&date.unwrap_or(BBox::new("".to_string(), NoPolicy::new())).discard_box(), fmt)
            .unwrap_or(
            NaiveDate::from_ymd_opt(1, 1, 1)
                .ok_or(ServiceError::InvalidDate)?
            )
    )
}