use std::sync::Arc;
use alohomora::{bbox::BBox, db::BBoxConn, policy::NoPolicy, AlohomoraType};
use std::sync::Mutex;
use ::rocket::http::Status;
use alohomora::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use ::rocket::outcome::IntoOutcome;

#[derive(AlohomoraType, Clone)]
pub struct ContextDataType {
    pub session_id: Option<BBox<String, NoPolicy>>,
    pub key: Option<BBox<String, NoPolicy>>,
    pub db: Arc<Mutex<BBoxConn>>,
}

#[::rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for ContextDataType {
    type BBoxError = ();
    
    async fn from_bbox_request(request: BBoxRequest<'a, 'r>,) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let session_id: Option<BBox<String, NoPolicy>> = request.cookies().get("id")
            .and_then(|k| Some(k.value().to_owned()));
        
        let key: Option<BBox<String, NoPolicy>> = request.cookies().get("key")
            .and_then(|k| Some(k.value().to_owned()));
        
        request.route().and_then(|_|{
            Some(ContextDataType{
                key,
                session_id,
                db: todo!()
            })
        }).into_outcome((Status::InternalServerError, ()))
    }
}