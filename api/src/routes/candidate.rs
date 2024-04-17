use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use alohomora::context;
use alohomora::rocket::{BBoxForm, ContextResponse, JsonResponse};
use entity::application;
use portfolio_core::policies::context::ContextDataType;
use portfolio_core::utils::response::MyResult;
use portfolio_core::Query;
use portfolio_core::error::ServiceError;
use portfolio_core::models::auth::AuthenticableTrait;
use portfolio_core::models::candidate::{ApplicationDetails, NewCandidateResponse};
use portfolio_core::sea_orm::prelude::Uuid;
use portfolio_core::services::application_service::ApplicationService;
use portfolio_core::services::portfolio_service::{PortfolioService, SubmissionProgress};
// use rocket::http::Method::Delete;
use requests::LoginRequest;
use portfolio_core::models::candidate::NewCandidateResponseOut;
use rocket::http::{Cookie, CookieJar, Status};
use rocket::response::status::Custom;
use rocket::serde::json::Json;

use alohomora::{bbox::BBox, context::Context, orm::Connection, policy::{AnyPolicy, NoPolicy}, pure::{execute_pure, PrivacyPureRegion}, rocket::{get, post, route, BBoxCookie, BBoxCookieJar, BBoxJson, FromBBoxData}};
use rocket::serde::Serialize;


use crate::guards::data::letter::Letter;
use crate::guards::data::portfolio::Portfolio;
use crate::{guards::request::auth::ApplicationAuth, pool::Db, requests};

use super::to_custom_error;

#[post("/login", data = "<login_form>")]
pub async fn login(
    conn: Connection<'_, Db>,
    login_form: BBoxJson<LoginRequest>,
    // ip_addr: SocketAddr, // TODO uncomment in production
    cookies: BBoxCookieJar<'_, '_>,
    context: Context<ContextDataType>
) -> MyResult<(), (rocket::http::Status, String)> {
    let ip_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let db = conn.into_inner();

    let res = ApplicationService::login(
        db,
        login_form.application_id.clone(),
        login_form.password.clone(),
        BBox::new(ip_addr.ip().to_string(), NoPolicy::new()),
    ).await.map_err(to_custom_error);

    let (session_token, private_key) = match res {
        Ok(a) => a,
        Err(e) => return MyResult::Err(e),
    };

    cookies.add(BBoxCookie::new("id", session_token.clone()), context.clone());
    cookies.add(BBoxCookie::new("key", private_key.clone()), context.clone());

    return MyResult::Ok(());
}

#[post("/logout")]
pub async fn logout(
    conn: Connection<'_, Db>,
    _session: ApplicationAuth,
    cookies: BBoxCookieJar<'_, '_>,
    context: Context<ContextDataType>
) -> MyResult<(), (rocket::http::Status, String)> {
    let db = conn.into_inner();

    let cookie: BBoxCookie<'static, NoPolicy> = cookies
        .get("id").unwrap(); // unwrap would be safe here because of the auth guard
        //.ok_or(Custom(Status::Unauthorized,"No session cookie".to_string(),))?;

    let session_id = execute_pure(cookie.value().to_owned(), PrivacyPureRegion::new(|c: String|{ 
        Uuid::try_parse(c.as_str()).unwrap()
    })).unwrap().specialize_policy().unwrap();

    let session = Query::find_session_by_uuid(db, session_id).await.unwrap().unwrap(); // TODO
    ApplicationService::logout(db, session)
        .await
        .map_err(to_custom_error)?;

    cookies.remove(cookies.get::<NoPolicy>("id").unwrap());
    cookies.remove(cookies.get::<NoPolicy>("key").unwrap());

    MyResult::Ok(())
}

#[get("/whoami")]
pub async fn whoami(conn: Connection<'_, Db>, 
    session: ApplicationAuth, 
    context: Context<ContextDataType>
) -> MyResult<JsonResponse<NewCandidateResponse, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();

    let private_key = session.get_private_key();
    let application: entity::application::Model = session.into();
    let candidate = ApplicationService::find_related_candidate(&db, &application)
        .await.map_err(to_custom_error)?; // TODO more compact
    let applications = Query::find_applications_by_candidate_id(&db, candidate.id.clone())
        .await.map_err(|e| to_custom_error(ServiceError::DbError(e)))?; 
    let response = NewCandidateResponse::from_encrypted(
        application.id,
        applications,
        &private_key,
        candidate
    ).await
        .map_err(to_custom_error)?;

    //let a = alohomora::fold::fold(response).unwrap();

    MyResult::Ok(JsonResponse::from((response, context)))
}

// TODO: use put instead of post???
#[post("/details", data = "<details>")]
pub async fn post_details(
    conn: Connection<'_, Db>,
    details: BBoxJson<ApplicationDetails>,
    session: ApplicationAuth,
    context: Context<ContextDataType>
) -> MyResult<JsonResponse<ApplicationDetails, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let form = details.into_inner();
    form.candidate.validate_self().map_err(to_custom_error)?;
    let application: application::Model = session.into();
    let candidate = ApplicationService::find_related_candidate(&db, &application).await.map_err(to_custom_error)?; // TODO

    let _candidate_parent = ApplicationService::add_all_details(&db, &application, candidate, &form)
        .await 
        .map_err(to_custom_error)?;

    MyResult::Ok(JsonResponse::from((form, context)))
}

#[get("/details")]
pub async fn get_details(
    conn: Connection<'_, Db>,
    session: ApplicationAuth,
    context: Context<ContextDataType>
) -> MyResult<JsonResponse<ApplicationDetails, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let private_key = session.get_private_key();
    let application: entity::application::Model = session.into();

    let details = ApplicationService::decrypt_all_details(
        private_key,
        db,
        &application
    )
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(JsonResponse::from((details, context)))
}
#[post("/cover_letter", data = "<letter>")]
pub async fn upload_cover_letter(
    session: ApplicationAuth,
    letter: Letter<NoPolicy>,
) -> MyResult<(), (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    PortfolioService::add_cover_letter_to_cache(application.candidate_id.discard_box(), Into::<BBox<Vec<u8>, NoPolicy>>::into(letter).discard_box())
        .await
        .map_err(to_custom_error)?;
    MyResult::Ok(())
}

#[route(DELETE, "/cover_letter")]
pub async fn delete_cover_letter(session: ApplicationAuth) -> MyResult<(), (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    PortfolioService::delete_cover_letter_from_cache(application.candidate_id.discard_box())
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(())
}

#[post("/portfolio_letter", data = "<letter>")]
pub async fn upload_portfolio_letter(
    session: ApplicationAuth,
    letter: Letter<NoPolicy>,
) -> MyResult<(), (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    PortfolioService::add_portfolio_letter_to_cache(application.candidate_id.discard_box(), Into::<BBox<Vec<u8>, NoPolicy>>::into(letter).discard_box())
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(())
}

#[route(DELETE, "/portfolio_letter")]
pub async fn delete_portfolio_letter(session: ApplicationAuth) -> Result<(), (rocket::http::Status, String)> {
    let candidate: entity::application::Model = session.into();

    PortfolioService::delete_portfolio_letter_from_cache(candidate.candidate_id.discard_box())
        .await
        .map_err(to_custom_error)?;

    Ok(())
}

#[post("/portfolio_zip", data = "<portfolio>")]
pub async fn upload_portfolio_zip(
    session: ApplicationAuth,
    portfolio: Portfolio<NoPolicy>,
) -> Result<(), (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    PortfolioService::add_portfolio_zip_to_cache(application.candidate_id.discard_box(), Into::<BBox<Vec<u8>, NoPolicy>>::into(portfolio).discard_box())
        .await
        .map_err(to_custom_error)?;

    Ok(())
}

#[route(DELETE, "/portfolio_zip")]
pub async fn delete_portfolio_zip(session: ApplicationAuth) -> Result<(), (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    PortfolioService::delete_portfolio_zip_from_cache(application.candidate_id.discard_box())
        .await
        .map_err(to_custom_error)?;

    Ok(())
}

#[get("/submission_progress")]
pub async fn submission_progress(
    session: ApplicationAuth,
    context: Context<ContextDataType>
) -> MyResult<ContextResponse<String, NoPolicy, ContextDataType>, (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    let submission_progress = execute_pure(application.candidate_id, 
        PrivacyPureRegion::new(|id| {
            PortfolioService::get_submission_progress(id)
                .map(|x| {
                    let s = serde_json::to_string(&x).unwrap();
                    s
                }).map_err(to_custom_error)
        })
    ).unwrap().specialize_policy().unwrap();
    
    match submission_progress.transpose() {
        Ok(o) => MyResult::Ok(ContextResponse(o, context)),
        Err(e) => MyResult::Err(e),
    }
}

#[post("/submit")]
pub async fn submit_portfolio(
    conn: Connection<'_, Db>,
    session: ApplicationAuth,
) -> MyResult<(), (rocket::http::Status, String)> {
    let db = conn.into_inner();

    let application: entity::application::Model = session.into();
    let candidate = ApplicationService::find_related_candidate(&db, &application).await.map_err(to_custom_error)?; // TODO

    let submit = PortfolioService::submit(&candidate, &db).await;

    if submit.is_err() {
        let e = submit.err().unwrap();
        // Delete on critical error
        if e.code() == 500 {
            // Cleanup
            PortfolioService::delete_portfolio(application.id.discard_box())
                .await
                .unwrap();
        }
        return MyResult::Err(to_custom_error(e));
    }

    MyResult::Ok(())
}

#[post("/delete")]
pub async fn delete_portfolio(
    session: ApplicationAuth,
) -> MyResult<(), (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    PortfolioService::delete_portfolio(application.candidate_id.discard_box())
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(())
}

#[get("/download")]
pub async fn download_portfolio(session: ApplicationAuth) -> Result<Vec<u8>, (rocket::http::Status, String)> {
    let private_key = session.get_private_key();
    let application: entity::application::Model = session.into();

    let file = PortfolioService::get_portfolio(application.candidate_id.discard_box(), private_key.discard_box())
        .await
        .map_err(to_custom_error);

    file
}

#[cfg(test)]
mod tests {
    use portfolio_core::{crypto, models::candidate::{ApplicationDetails, CleanApplicationDetails, CleanNewCandidateResponse, NewCandidateResponse}, sea_orm::prelude::Uuid};
    use rocket::{
        http::{Cookie, Status},
        local::blocking::Client,
    };

    use crate::{
        routes::admin::tests::admin_login,
        test::tests::{test_client, APPLICATION_ID, CANDIDATE_PASSWORD, PERSONAL_ID_NUMBER},
    };

    fn candidate_login(client: &Client) -> (Cookie, Cookie) {
        let response = client
            .post("/candidate/login")
            .body(format!(
                "{{
            \"applicationId\": {},
            \"password\": \"{}\"
        }}",
                APPLICATION_ID, CANDIDATE_PASSWORD
            ))
            .dispatch();

        (
            response.cookies().get("id").unwrap().to_owned(),
            response.cookies().get("key").unwrap().to_owned(),
        )
    }

    const CANDIDATE_DETAILS: &'static str = "{
        \"candidate\": {
            \"name\": \"idk\",
            \"surname\": \"idk\",
            \"birthSurname\": \"surname\",
            \"birthplace\": \"Praha 1\",
            \"birthdate\": \"2015-09-18\",
            \"address\": \"Stefanikova jidelna\",
            \"letterAddress\": \"Stefanikova jidelna\",
            \"telephone\": \"000111222333\",
            \"citizenship\": \"Czech Republic\",
            \"email\": \"magor@magor.cz\",
            \"sex\": \"MALE\",
            \"personalIdNumber\": \"0101010000\",
            \"schoolName\": \"29988383\",
            \"healthInsurance\": \"000\",
            \"grades\": [],
            \"firstSchool\": {\"name\": \"SSPŠ\", \"field\": \"KB\"},
            \"secondSchool\": {\"name\": \"SSPŠ\", \"field\": \"IT\"},
            \"testLanguage\": \"CZ\"
        },
        \"parents\": [
            {
                \"name\": \"maminka\",
                \"surname\": \"chad\",
                \"telephone\": \"420111222333\",
                \"email\": \"maminka@centrum.cz\"
            }
        ]
    }";

    #[test]
    fn test_login_valid_credentials() {
        let client = test_client().lock().unwrap();
        let _response = candidate_login(&client);
    }

    #[test]
    fn test_auth_candidate() {
        let client = test_client().lock().unwrap();
        let cookies = candidate_login(&client);
        let response = client
            .get("/candidate/whoami")
            .cookie(cookies.0)
            .cookie(cookies.1)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);

        let candidate = response.into_json::<CleanNewCandidateResponse>().unwrap();
        // assert_eq!(candidate.id, APPLICATION_ID); // TODO
        assert_eq!(candidate.personal_id_number, PERSONAL_ID_NUMBER);
    }

    #[test]
    fn test_add_get_candidate_details() {
        let client = test_client().lock().unwrap();
        let cookies = candidate_login(&client);

        let details_orig: CleanApplicationDetails = serde_json::from_str(CANDIDATE_DETAILS).unwrap();

        let response = client
            .post("/candidate/details")
            .cookie(cookies.0.clone())
            .cookie(cookies.1.clone())
            .body(CANDIDATE_DETAILS.to_string())
            .dispatch();

        assert_eq!(response.status(), Status::Ok);

        let response = client
            .get("/candidate/details")
            .cookie(cookies.0)
            .cookie(cookies.1)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);

        let details_resp: CleanApplicationDetails = serde_json::from_str(&response.into_string().unwrap()).unwrap();
        assert_eq!(details_orig, details_resp);
    }

    #[test]
    fn test_invalid_token_every_secured_endpoint() {
        let client = test_client().lock().unwrap();

        let id = Cookie::new("id", Uuid::new_v4().to_string());
        let (private_key, _) = crypto::create_identity();
        let key = Cookie::new("key", private_key.discard_box());

        let response = client
            .post("/candidate/details")
            .cookie(id.clone())
            .cookie(key.clone())
            .body(CANDIDATE_DETAILS.to_string())
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);

        let response = client
            .get("/candidate/details")
            .cookie(id.clone())
            .cookie(key.clone())
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);

        let response = client
            .get("/candidate/whoami")
            .cookie(id.clone())
            .cookie(key.clone())
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
    }

    #[test]
    fn test_admin_token_on_secured_candidate_endpoints() {
        let client = test_client().lock().unwrap();
        let cookies = admin_login(&client);

        let response = client
            .post("/candidate/details")
            .cookie(cookies.0.clone())
            .cookie(cookies.1.clone())
            .body(CANDIDATE_DETAILS.to_string())
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);

        let response = client
            .get("/candidate/details")
            .cookie(cookies.0.clone())
            .cookie(cookies.1.clone())
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);

        let response = client
            .get("/candidate/whoami")
            .cookie(cookies.0.clone())
            .cookie(cookies.1.clone())
            .dispatch();
        assert_eq!(response.status(), Status::Unauthorized);
    }
}
