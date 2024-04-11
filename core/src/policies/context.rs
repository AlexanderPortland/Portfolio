use alohomora::AlohomoraType;
use ::rocket::http::Status;
use alohomora::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use ::rocket::outcome::IntoOutcome;
use ::rocket::State;

#[derive(AlohomoraType, Clone)]
//#[alohomora_out_type(to_derive = [db, config])]
pub struct ContextDataType {

}

#[::rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for ContextDataType {
    type BBoxError = ();
    
    async fn from_bbox_request(request: BBoxRequest<'a, 'r>,) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        request.route().and_then(|_|{
            Some(ContextDataType{
                
            })
        }).into_outcome((Status::InternalServerError, 
            ()))
    }
}