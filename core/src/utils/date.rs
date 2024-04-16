use alohomora::bbox::BBox;
use alohomora::policy::Policy;
use alohomora::pure::PrivacyPureRegion;
use chrono::NaiveDate;

use crate::error::ServiceError;

pub fn parse_naive_date_from_opt_str<P: Policy>(date: Option<BBox<String, P>>, fmt: &str)
    -> Result<BBox<NaiveDate, P>, ServiceError>
{
    match date {
        None => Err(ServiceError::InvalidDate),
        Some(date) => {
            let date = date.into_ppr(PrivacyPureRegion::new(|date: String| {
                let date = NaiveDate::parse_from_str(date.as_str(), fmt);
                match date {
                    Ok(date) => Ok(date),
                    Err(_) => Err(ServiceError::InvalidDate),
                }
            }));
            date.transpose()
        }
    }
}