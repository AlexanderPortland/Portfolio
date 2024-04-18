use alohomora::bbox::BBox;
use alohomora::policy::Policy;
use chrono::NaiveDate;

use crate::error::ServiceError;

pub fn parse_naive_date_from_opt_str<P: Policy>(date: Option<BBox<String, P>>, fmt: &str)
    -> Result<BBox<NaiveDate, P>, ServiceError>
{
    match date {
        None => Err(ServiceError::InvalidDate),
        Some(date) => {
            match date.into_date(fmt) {
                Ok(date) => Ok(date),
                Err(_) =>  Err(ServiceError::InvalidDate),
            }
        }
    }
}