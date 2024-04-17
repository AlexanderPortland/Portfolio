use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::pure::execute_pure;
use alohomora::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use entity::application::Model as Application;
use portfolio_core::models::auth::AuthenticableTrait;
use portfolio_core::sea_orm::prelude::Uuid;
use portfolio_core::services::application_service::ApplicationService;
use rocket::http::Status;
use rocket::outcome::Outcome;


use alohomora::pure::PrivacyPureRegion;


use crate::pool::Db;

pub struct ApplicationAuth(Application, BBox<String, NoPolicy>);

impl Into<Application> for ApplicationAuth {
    fn into(self) -> Application {
        self.0
    }
}

impl ApplicationAuth {
    pub fn get_private_key(&self) -> BBox<String, NoPolicy> {
        self.1.clone()
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for ApplicationAuth {
    type BBoxError = ();

    async fn from_bbox_request(
        req: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<ApplicationAuth, Self::BBoxError> {
        let cookie_id = req.cookies().get::<NoPolicy>("id");
        let cookie_private_key = req.cookies().get("key");

        let Some(cookie_id) = cookie_id else {
            return BBoxRequestOutcome::Failure((Status::Unauthorized, ()));
        };

        let Some(cookie_private_key) = cookie_private_key else {
            return BBoxRequestOutcome::Failure((Status::Unauthorized, ()));
        };

        let session_id = cookie_id.value().to_owned();
        let private_key = cookie_private_key.value().to_owned();

        let conn: &rocket::State<Db> = req.guard().await.unwrap();

        let uuid_bbox = session_id.into_ppr(
            PrivacyPureRegion::new(|session_id: String| {
                Uuid::parse_str(session_id.as_str()).unwrap()
            })
        );

        // let uuid = match Uuid::parse_str(&session_id) {
        //     Ok(uuid) => uuid,
        //     Err(_) => return Outcome::Failure((Status::BadRequest, None)),
        // };

        let session = ApplicationService::auth(&conn.conn, uuid_bbox).await;

        match session {
            Ok(model) => {
                //info!("{}: CANDIDATE {} AUTHENTICATED", format_request(req), model.id);
                Outcome::Success(ApplicationAuth(model, private_key))
            },
            Err(_e) => {
                //info!("{}: CANDIDATE {} AUTHENTICATION FAILED", format_request(req), e);
                Outcome::Failure((Status::Unauthorized, ()))
            },
        }
    }
}

// impl<'r> FromRequest<'r> for ApplicationAuth {
//     type Error = Option<String>;
//     async fn from_request(
//         req: &'r Request<'_>,
//     ) -> Outcome<ApplicationAuth, (Status, Self::Error), Status> {
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

//         let session = ApplicationService::auth(conn, uuid).await;

//         match session {
//             Ok(model) => {
//                 info!("{}: CANDIDATE {} AUTHENTICATED", format_request(req), model.id);
//                 Outcome::Success(ApplicationAuth(model, private_key.to_string().to_string()))
//             },
//             Err(e) => {
//                 info!("{}: CANDIDATE {} AUTHENTICATION FAILED", format_request(req), e);
//                 Outcome::Failure((Status::Unauthorized, None))
//             },
//         }
//     }
// }
