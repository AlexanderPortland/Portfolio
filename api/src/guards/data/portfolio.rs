use alohomora::bbox::BBox;
use alohomora::policy::FrontendPolicy;
use alohomora::pure::PrivacyPureRegion;
use alohomora::rocket::{BBoxData, BBoxDataOutcome, BBoxRequest, FromBBoxData};
use rocket::data::{Capped, ToByteUnit};
use rocket::http::{ContentType, Status};
use rocket::outcome::Outcome;


pub struct Portfolio<P: FrontendPolicy>(BBox<Vec<u8>, P>);

impl<P: FrontendPolicy> Into<BBox<Vec<u8>, P>> for Portfolio<P> {
    fn into(self) -> BBox<Vec<u8>, P> {
        self.0
    }
}

#[rocket::async_trait]
impl<'a, 'r, P: FrontendPolicy> FromBBoxData<'a, 'r> for Portfolio<P> {
    type BBoxError = Option<String>;

    async fn from_data(req: BBoxRequest<'a, 'r>, data: BBoxData<'a>) -> BBoxDataOutcome<'a, 'r, Portfolio<P>> {
        if req.content_type() != Some(&ContentType::ZIP) {
            return Outcome::Failure((Status::BadRequest, None))
        }

        let data = data.open::<P>(101.megabytes(), req);

        let data_bytes = data.into_bytes().await.unwrap();

        let result = data_bytes.into_ppr(PrivacyPureRegion::new(
            |data_bytes: Capped<Vec<u8>>| {
                if !data_bytes.is_complete() { return Err(()); }

                let data_bytes = data_bytes.into_inner();

                let is_zip = portfolio_core::utils::filetype::filetype_is_zip(&data_bytes);
                if !is_zip { return Err(()); }

                return Ok(data_bytes);
            }
        ));

        match result.transpose() {
            Err(_) => Outcome::Failure((Status::BadRequest, None)),
            Ok(data_bytes) => Outcome::Success(Portfolio(data_bytes)),
        }
    }
}
