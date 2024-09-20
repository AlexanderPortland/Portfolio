use std::{collections::HashMap, marker::PhantomData, ops::Deref};
use alohomora::{bbox::BBox, policy::NoPolicy, AlohomoraType};
use sea_orm_rocket::Database;
use ::rocket::http::Status;
use alohomora::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use ::rocket::outcome::IntoOutcome;

use crate::request::{AdminLoginRequest, LoginRequest};

// #[derive(Clone)]
pub struct RealContextDataType<Db: sea_orm_rocket::Database>  {
    pub session_id: Option<BBox<String, NoPolicy>>,
    pub key: Option<BBox<String, NoPolicy>>,
    pub conn: &'static sea_orm::DatabaseConnection, 
    pub admin_login: Option<crate::request::AdminLoginRequest>,
    pub candidate_login: Option<crate::request::LoginRequest>,
    pub phantom: PhantomData<Db>,
}

impl<Db: sea_orm_rocket::Database> Clone for RealContextDataType<Db> {
    fn clone(&self) -> Self {
        RealContextDataType{
            session_id: self.session_id.clone(),
            key: self.key.clone(),
            admin_login: self.admin_login.clone(),
            candidate_login: self.candidate_login.clone(),
            conn: self.conn,
            phantom: self.phantom,
        }
    }
}

#[derive(Debug)]
pub struct ContextDataTypeOut {
    pub session_id: Option<String>,
    pub key: Option<String>,
    pub conn: &'static sea_orm::DatabaseConnection,
    pub admin_login: Option<crate::request::AdminLoginRequestOut>,
    pub candidate_login: Option<crate::request::LoginRequestOut>,
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
                let c = ContextDataTypeOut {
                    session_id: Option::<BBox<String, NoPolicy>>::from_enum(map.remove("session_id").unwrap())?,
                    key: Option::<BBox<String, NoPolicy>>::from_enum(map.remove("key").unwrap())?,
                    conn: *conn,
                    admin_login: Option::<AdminLoginRequest>::from_enum(map.remove("admin_login").unwrap())?,
                    candidate_login: Option::<LoginRequest>::from_enum(map.remove("candidate_login").unwrap())?,
                };
                // println!("after from we have {:?}", c);
                Ok(c)
            },
            _ => panic!("bad"),
        }
    }
    fn to_enum(self) -> alohomora::AlohomoraTypeEnum {
        // println!("before to we have admin: {:?}", self.admin_login);
        // println!("before to we have candidate: {:?}", self.candidate_login);
        // todo!();
        let mut map = HashMap::new();
        map.insert(String::from("session_id"), self.session_id.to_enum());
        map.insert(String::from("key"), self.key.to_enum());
        map.insert(String::from("conn"), alohomora::AlohomoraTypeEnum::Value(Box::new(self.conn)));
        map.insert(String::from("admin_login"), self.admin_login.to_enum());
        map.insert(String::from("candidate_login"), self.candidate_login.to_enum());
        alohomora::AlohomoraTypeEnum::Struct(map)
    }
}

// #[derive(AlohomoraType, Clone)]
// pub struct FakeContextDataType {
    
// }

// #[allow(non_snake_case)]
// #[derive(alohomora_derive::RequestBBoxJson)]
// pub struct LoginRequest {
//     pub applicationId: BBox<i32, crate::FakePolicy>,
//     pub password: BBox<String, crate::FakePolicy>,
// }

#[::rocket::async_trait]
impl<'a, 'r, P: sea_orm_rocket::Pool<Connection = sea_orm::DatabaseConnection>, Db: sea_orm_rocket::Database<Pool = P>> FromBBoxRequest<'a, 'r> for RealContextDataType<Db> {
    type BBoxError = ();
    
    async fn from_bbox_request(request: BBoxRequest<'a, 'r>,) -> BBoxRequestOutcome<Self, Self::BBoxError> {

        // println!("in from normal bbox req");
        // todo!();
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
                admin_login: None,
                candidate_login: None,
                phantom: PhantomData,
            })
        }).into_outcome((Status::InternalServerError, ()))
    }
}

#[rocket::async_trait]
impl <'a, 'r, Db: Database> alohomora::rocket::FromBBoxRequestAndData<'a, 'r, alohomora::rocket::BBoxJson<AdminLoginRequest>> for RealContextDataType<Db> where 
    RealContextDataType<Db>: FromBBoxRequest<'a, 'r> {
    type BBoxError = ();
    async fn from_bbox_request_and_data(
        request: BBoxRequest<'a, 'r>,
        data: &'_ alohomora::rocket::BBoxJson<AdminLoginRequest>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        // println!("in from with data ADMIN");
        let mut context = RealContextDataType::<Db>::from_bbox_request(request).await.unwrap();
        context.admin_login = Some(data.deref().to_owned());
        // println!("have cointext to prove it{:?}", context.admin_login);
        // println!("have sesh id to prove it{:?}", context.session_id);
        // println!("have key to prove it{:?}", context.key);
        // todo!();
        rocket::outcome::Outcome::Success(context)
    }
}

#[rocket::async_trait]
impl <'a, 'r, Db: Database> alohomora::rocket::FromBBoxRequestAndData<'a, 'r, alohomora::rocket::BBoxJson<LoginRequest>> for RealContextDataType<Db> where 
    RealContextDataType<Db>: FromBBoxRequest<'a, 'r> {
    type BBoxError = ();
    async fn from_bbox_request_and_data(
        request: BBoxRequest<'a, 'r>,
        data: &'_ alohomora::rocket::BBoxJson<LoginRequest>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        // println!("in from with data CAND");
        let mut context = RealContextDataType::<Db>::from_bbox_request(request).await.unwrap();
        context.candidate_login = Some(data.deref().to_owned());
        // println!("have cointext to prove it {:?}", context.candidate_login);
        // todo!();
        rocket::outcome::Outcome::Success(context)
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