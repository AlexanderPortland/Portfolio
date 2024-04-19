use std::sync::Arc;
use alohomora::{bbox::BBox, db::BBoxConn, policy::NoPolicy, AlohomoraType};
use std::sync::Mutex;
use ::rocket::http::Status;
use alohomora::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use ::rocket::outcome::IntoOutcome;

pub type ContextDataType = FakeContextDataType;

#[derive(AlohomoraType, Clone)]
pub struct RealContextDataType {
    pub session_id: Option<BBox<String, NoPolicy>>,
    pub key: Option<BBox<String, NoPolicy>>,
    //pub db: Arc<Mutex<BBoxConn>>,
}

#[derive(AlohomoraType, Clone)]
pub struct FakeContextDataType {
    
}

#[::rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for RealContextDataType {
    type BBoxError = ();
    
    async fn from_bbox_request(request: BBoxRequest<'a, 'r>,) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let session_id: Option<BBox<String, NoPolicy>> = request.cookies().get("id")
            .and_then(|k| Some(k.value().to_owned()));
        
        let key: Option<BBox<String, NoPolicy>> = request.cookies().get("key")
            .and_then(|k| Some(k.value().to_owned()));
        
        request.route().and_then(|_|{
            Some(RealContextDataType{
                key,
                session_id,
                //db: todo!()
            })
        }).into_outcome((Status::InternalServerError, ()))
    }
}

#[::rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for FakeContextDataType {
    type BBoxError = ();
    
    async fn from_bbox_request(request: BBoxRequest<'a, 'r>,) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        request.route().and_then(|_|{
            Some(FakeContextDataType{})
        }).into_outcome((Status::InternalServerError, ()))
    }
}