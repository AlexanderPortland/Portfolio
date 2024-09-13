use std::{collections::HashMap, marker::PhantomData, sync::Arc};
use alohomora::{bbox::BBox, db::BBoxConn, policy::NoPolicy, AlohomoraType};
use std::sync::Mutex;
use ::rocket::http::Status;
use alohomora::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use ::rocket::outcome::IntoOutcome;

// #[derive(Clone)]
pub struct RealContextDataType<Db: sea_orm_rocket::Database>  {
    pub session_id: Option<BBox<String, NoPolicy>>,
    pub key: Option<BBox<String, NoPolicy>>,
    pub conn: &'static sea_orm::DatabaseConnection,  // sea_orm::DatabaseConnection,
    pub phantom: PhantomData<Db>,
    //pub db: Arc<Mutex<BBoxConn>>,
}

impl<Db: sea_orm_rocket::Database> Clone for RealContextDataType<Db> {
    fn clone(&self) -> Self {
        RealContextDataType{
            session_id: self.session_id.clone(),
            key: self.key.clone(),
            conn: self.conn,
            phantom: self.phantom,
        }
    }
}

// impl<Db> Clone for RealContextDataType<Db> {
//     fn clone(&self) -> Self {
//         todo!()
//     }
// }

pub struct ContextDataTypeOut {
    pub session_id: Option<String>,
    pub key: Option<String>,
    pub conn: &'static sea_orm::DatabaseConnection,
}

impl<Db: sea_orm_rocket::Database> AlohomoraType for RealContextDataType<Db> {
    type Out = ContextDataTypeOut;
    fn from_enum(e: alohomora::AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        match e {
            alohomora::AlohomoraTypeEnum::Struct(mut map) => {
                let conn = map.remove("conn").unwrap();
                let conn = match conn {
                    alohomora::AlohomoraTypeEnum::Value(conn) => {
                        conn.downcast().unwrap()
                    }
                    _ => panic!("bad"),
                };

                Ok(ContextDataTypeOut {
                    session_id: Option::<BBox<String, NoPolicy>>::from_enum(map.remove("session_id").unwrap())?,
                    key: Option::<BBox<String, NoPolicy>>::from_enum(map.remove("key").unwrap())?,
                    conn: *conn,
                })
            },
            _ => panic!("bad"),
        }
    }
    fn to_enum(self) -> alohomora::AlohomoraTypeEnum {
        let mut map = HashMap::new();
        map.insert(String::from("session_id"), self.session_id.to_enum());
        map.insert(String::from("key"), self.key.to_enum());
        map.insert(String::from("conn"), alohomora::AlohomoraTypeEnum::Value(Box::new(self.conn)));
        alohomora::AlohomoraTypeEnum::Struct(map)
    }
}

// #[derive(AlohomoraType, Clone)]
// pub struct FakeContextDataType {
    
// }

#[::rocket::async_trait]
impl<'a, 'r, P: sea_orm_rocket::Pool<Connection = sea_orm::DatabaseConnection>, Db: sea_orm_rocket::Database<Pool = P>> FromBBoxRequest<'a, 'r> for RealContextDataType<Db> {
    type BBoxError = ();
    
    async fn from_bbox_request(request: BBoxRequest<'a, 'r>,) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let session_id: Option<BBox<String, NoPolicy>> = request.cookies().get("id")
            .and_then(|k| Some(k.value().to_owned()));
        
        let key: Option<BBox<String, NoPolicy>> = request.cookies().get("key")
            .and_then(|k| Some(k.value().to_owned()));

        let conn: alohomora::orm::Connection<'a, Db> = match FromBBoxRequest::from_bbox_request(request).await {
            BBoxRequestOutcome::Success(conn) => conn,
            BBoxRequestOutcome::Failure(f) => { return BBoxRequestOutcome::Failure(f); },
            BBoxRequestOutcome::Forward(f) => { return BBoxRequestOutcome::Forward(f); },
        };
        
        let y: &sea_orm::DatabaseConnection  = conn.into_inner();
        let conn: &'static sea_orm::DatabaseConnection = unsafe { std::mem::transmute(y) };

        request.route().and_then(|_|{
            Some(RealContextDataType{
                key,
                session_id,
                conn,
                phantom: PhantomData,
                //conn,
                //db: todo!()
            })
        }).into_outcome((Status::InternalServerError, ()))
    }
}

// #[::rocket::async_trait]
// impl<'a, 'r> FromBBoxRequest<'a, 'r> for FakeContextDataType {
//     type BBoxError = ();
    
//     async fn from_bbox_request(request: BBoxRequest<'a, 'r>,) -> BBoxRequestOutcome<Self, Self::BBoxError> {
//         request.route().and_then(|_|{
//             Some(FakeContextDataType{})
//         }).into_outcome((Status::InternalServerError, ()))
//     }
// }