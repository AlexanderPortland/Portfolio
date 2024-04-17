use alohomora::bbox::BBox;
use alohomora::policy::FrontendPolicy;
use alohomora::pure::PrivacyPureRegion;
use alohomora::rocket::{BBoxData, BBoxDataOutcome, BBoxRequest, FromBBoxData};

use rocket::data::{Capped, ToByteUnit};
use rocket::http::{ContentType, Status};
use rocket::outcome::Outcome;


//
pub struct Letter<P: FrontendPolicy>(BBox<Vec<u8>, P>);

impl<P: FrontendPolicy> Into<BBox<Vec<u8>, P>> for Letter<P> {
    fn into(self) -> BBox<Vec<u8>, P> {
        self.0
    }
}

#[rocket::async_trait]
impl<'a, 'r, P: FrontendPolicy> FromBBoxData<'a, 'r> for Letter<P> {
    type BBoxError = Option<String>;

    async fn from_data(req: BBoxRequest<'a, 'r>, data: BBoxData<'a>) -> BBoxDataOutcome<'a, 'r, Letter<P>> {
        if req.content_type() != Some(&ContentType::PDF) {
            return Outcome::Failure((Status::BadRequest, None))
        }

        let data = data.open::<P>(11.megabytes(), req);
        let data_bytes: alohomora::bbox::BBox<Capped<Vec<u8>>, P> = data.into_bytes().await.unwrap();
        let result: BBox<Result<Vec<u8>, ()>, P> = data_bytes.into_ppr(PrivacyPureRegion::new(|data_bytes: Capped<Vec<u8>>| {
            if !data_bytes.is_complete() {
                return Err(());
            }

            let data_bytes = data_bytes.into_inner();

            if !portfolio_core::utils::filetype::filetype_is_pdf(&data_bytes) {
                return Err(());
            }
            return Ok(data_bytes);
        }));
        match result.transpose() {
            Err(_) => Outcome::Failure((Status::BadRequest, None)),
            Ok(data_bytes) => Outcome::Success(Letter(data_bytes)),
        }
    }
}
