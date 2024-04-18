use alohomora::bbox::BBox;
use alohomora::pure::PrivacyPureRegion;
use alohomora::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use entity::admin::Model as Admin;


use portfolio_core::models::auth::AuthenticableTrait;
use portfolio_core::sea_orm::prelude::Uuid;
use portfolio_core::services::admin_service::AdminService;
use rocket::http::Status;
use rocket::outcome::Outcome;
use portfolio_policies::FakePolicy;


use crate::pool::Db;

pub struct AdminAuth(Admin, BBox<String, FakePolicy>);

impl Into<Admin> for AdminAuth {
    fn into(self) -> Admin {
        self.0
    }
}

impl AdminAuth {
    pub fn get_private_key(&self) -> BBox<String, FakePolicy> {
        self.1.clone()
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for AdminAuth {
    type BBoxError = ();

    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError>{
        let cookie_id = request.cookies().get::<FakePolicy>("id");
        let cookie_private_key = request.cookies().get::<FakePolicy>("key");

        let Some(cookie_id) = cookie_id else {
            return BBoxRequestOutcome::Failure((Status::Unauthorized, ()));
        };

        let Some(cookie_private_key) = cookie_private_key else {
            return BBoxRequestOutcome::Failure((Status::Unauthorized, ()));
        };

        let session_id = cookie_id.value().to_owned();
        let private_key = cookie_private_key.value().to_owned();

        let conn: &rocket::State<Db> = request.guard().await.unwrap();

        let uuid_bbox = session_id.into_ppr(
            PrivacyPureRegion::new(|session_id: String| {
                Uuid::parse_str(session_id.as_str()).unwrap()
            })
        );

        // let uuid = match Uuid::parse_str(&session_id) {
        //     Ok(uuid) => uuid,
        //     Err(_) => return Outcome::Failure((Status::BadRequest, None)),
        // };

        let session = AdminService::auth(&conn.conn, uuid_bbox).await;

        match session {
            Ok(model) => {
                //warn!("{}: ADMIN {} AUTHENTICATED", format_request(request), model.id);
                Outcome::Success(AdminAuth(model, private_key))
            },
            Err(_e) => {
                //info!("{}: ADMIN AUTHENTICATION FAILED: {}", format_request(request), e);
                Outcome::Failure((Status::Unauthorized, ()))
        },
        }
    }
}

// #[rocket::async_trait]
// impl<'r> FromRequest<'r> for AdminAuth {
//     type Error = Option<String>;
//     async fn from_request(req: &'r Request<'_>) -> Outcome<AdminAuth, (Status, Self::Error), ()> {
//         let cookie_id = req.cookies().get_private("id");
//         let cookie_private_key = req.cookies().get_private("key");

//         let Some(cookie_id) = cookie_id else {
//             return Outcome::Failure((Status::Unauthorized, None));
//         };

//         let Some(cookie_private_key) = cookie_private_key else {
//             return Outcome::Failure((Status::Unauthorized, None));
//         };

//         let session_id = cookie_id.value();
//         let private_key = cookie_private_key.value();

//         let conn = &req.rocket().state::<Db>().unwrap().conn;

//         let uuid = match Uuid::parse_str(&session_id) {
//             Ok(uuid) => uuid,
//             Err(_) => return Outcome::Failure((Status::BadRequest, None)),
//         };

//         let session = AdminService::auth(conn, uuid).await;

//         match session {
//             Ok(model) => {
//                 warn!("{}: ADMIN {} AUTHENTICATED", format_request(req), model.id);
//                 Outcome::Success(AdminAuth(model, private_key.to_string()))
//             },
//             Err(e) => {
//                 info!("{}: ADMIN AUTHENTICATION FAILED: {}", format_request(req), e);
//                 Outcome::Failure((Status::Unauthorized, None))
//         },
//         }

//     }
// }