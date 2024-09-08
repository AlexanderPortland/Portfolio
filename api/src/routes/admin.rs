use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use portfolio_core::{
    crypto::random_12_char_string, error::ServiceError, models::{application::ApplicationResponse, auth::AuthenticableTrait, candidate::{ApplicationDetails, CreateCandidateResponse}}, Query, sea_orm::prelude::Uuid, services::{admin_service::AdminService, application_service::ApplicationService, portfolio_service::PortfolioService}
};
use requests::{AdminLoginRequest, RegisterRequest};
use rocket::http::Status;


use portfolio_policies::context::ContextDataType;
use alohomora::{bbox::BBox, context::Context, orm::Connection, pure::{execute_pure, PrivacyPureRegion}, rocket::{BBoxCookie, BBoxCookieJar, BBoxJson, ContextResponse, get, JsonResponse, post, route}};
use alohomora::policy::{AnyPolicy, NoPolicy};

use portfolio_core::utils::csv::{ApplicationCsv, CandidateCsv, CsvExporter};
use portfolio_core::utils::response::MyResult;
use portfolio_policies::FakePolicy;

use crate::{guards::request::auth::AdminAuth, pool::Db, requests};

use super::to_custom_error;

#[post("/login", data = "<login_form>")]
pub async fn login(
    conn: Connection<'_, Db>,
    login_form: BBoxJson<AdminLoginRequest>,
    //ip_addr: BBox<SocketAddr, FakePolicy>,
    cookies: BBoxCookieJar<'_, '_>,
    context: Context<ContextDataType>
) -> Result<(), (rocket::http::Status, String)> {
    let ip_addr = BBox::new(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
        FakePolicy::new());
    let ip_addr = ip_addr.into_ppr(PrivacyPureRegion::new(|ip: SocketAddr| {
        ip.to_string()
    }));

    let db = conn.into_inner();
    let session_token_key = AdminService::login(
        db,
        login_form.adminId.clone(),
        login_form.password.clone(),
        ip_addr,
    )
    .await;

    let Ok(session_token_key) = session_token_key else {
        let e = session_token_key.unwrap_err();
        return Err((
            Status::from_code(e.code()).unwrap_or(Status::InternalServerError),
            e.to_string(),
        ));
    
    };

    let session_token = session_token_key.0;
    let private_key = session_token_key.1;

    let _ = cookies.add(BBoxCookie::new("id", session_token.clone()), context.clone());
    let _ = cookies.add(BBoxCookie::new("key", private_key.clone()), context.clone());

    return Ok(());
}

#[post("/logout")]
pub async fn logout(conn: Connection<'_, Db>, 
    _session: AdminAuth, 
    cookies: BBoxCookieJar<'_, '_>, 
    _context: Context<ContextDataType>
) -> Result<(), (rocket::http::Status, String)> {
    let db = conn.into_inner();

    let cookie: Option<BBoxCookie<'static, FakePolicy>> = cookies.get("id");
    let cookie_unwrapped: BBoxCookie<'static, FakePolicy> =
        cookie.ok_or((Status::Unauthorized, "No session cookie".to_string()))?;
    let cookie_value = cookie_unwrapped.value().to_owned();
    let session_id = cookie_value.into_ppr(PrivacyPureRegion::new(|c: String|{
        Uuid::try_parse(c.as_str()).unwrap()
    }));

    let session = Query::find_admin_session_by_uuid(db, session_id).await.unwrap().unwrap();
    
    let _res = AdminService::logout(db, session)
        .await
        .map_err(to_custom_error)?;

    cookies.remove(cookies.get::<FakePolicy>("id").unwrap());
    cookies.remove(cookies.get::<FakePolicy>("key").unwrap());

    Ok(())
}


#[get("/whoami")]
pub async fn whoami(
    session: AdminAuth,
    context: Context<ContextDataType>
) -> MyResult<ContextResponse<String, FakePolicy, ContextDataType>, (rocket::http::Status, String)> {
    let admin: entity::admin::Model = session.into();

    let a = admin.id.into_ppr(PrivacyPureRegion::new(|n: i32| {
        n.to_string()
    }));
    MyResult::Ok(ContextResponse(a, context))
}

#[get("/hello")]
pub async fn hello(_session: AdminAuth, _context: Context<ContextDataType>) -> Result<String, (rocket::http::Status, String)> {
    Ok("Hello admin".to_string())
}

#[post("/create", data = "<request>")]
pub async fn create_candidate(
    conn: Connection<'_, Db>,
    session: AdminAuth,
    request: BBoxJson<RegisterRequest>,
    context: Context<ContextDataType>
) -> MyResult<JsonResponse<CreateCandidateResponse, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let form = request.into_inner();
    let private_key = session.get_private_key();
    let plain_text_password = BBox::new(random_12_char_string(), FakePolicy {});

    let (application, applications, personal_id_number) = match ApplicationService::create(
        context.clone(),
        &private_key,
        &db,
        form.applicationId,
        &plain_text_password,
        form.personalIdNumber.clone()
    )
        .await
        .map_err(to_custom_error) {
            Ok(a) => a,
            Err(e) => return MyResult::Err(e),
        };

    println!("did the thing");
    let cand = CreateCandidateResponse {
        application_id: application.id.into_any_policy(),
        field_of_study: application.field_of_study.into_any_policy(),
        applications: applications.iter()
            .map(|a| a.id.to_owned().into_any_policy())
            .collect(),
        personal_id_number: personal_id_number.into_any_policy(),
        password: plain_text_password.into_any_policy(),
    };

    MyResult::Ok(JsonResponse::from((cand, context)))
}

#[allow(unused_variables)]
#[get("/candidates?<field>&<page>&<sort>")]
pub async fn list_candidates(
    conn: Connection<'_, Db>,
    session: AdminAuth,
    // These are intentionally NoPolicy. It's pagination information.
    field: Option<BBox<String, NoPolicy>>,
    page: Option<BBox<u64, NoPolicy>>,
    sort: Option<BBox<String, NoPolicy>>,
    context: Context<ContextDataType>
) -> MyResult<JsonResponse<Vec<ApplicationResponse>, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let private_key = session.get_private_key();

    let field = field.map(|b| b.discard_box());
    let page = page.map(|b| b.discard_box());
    let sort = sort.map(|b| b.discard_box());
    if let Some(field) = field.as_ref() {
        if !(field == "KB" || field == "IT" || field == "G") {
            return MyResult::Err((Status::BadRequest, "Invalid field of study".to_string()));
        }
    }

    let candidates = ApplicationService::list_applications(&private_key, db, field, page, sort)
        .await.map_err(to_custom_error)?;

    let a = MyResult::Ok(JsonResponse::from((candidates, context)));
    a
}

#[get("/candidates_csv")]
pub async fn list_candidates_csv(
    conn: Connection<'_, Db>,
    session: AdminAuth,
    context: Context<ContextDataType>
) -> MyResult<ContextResponse<Vec<u8>, AnyPolicy, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let private_key = session.get_private_key();

    let candidates = ApplicationCsv::export(db, private_key)
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(ContextResponse::from((candidates, context)))
}

#[get("/admissions_csv")]
pub async fn list_admissions_csv(
    conn: Connection<'_, Db>,
    session: AdminAuth,
    context: Context<ContextDataType>
) -> MyResult<ContextResponse<Vec<u8>, AnyPolicy, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let private_key = session.get_private_key();

    let candidates = CandidateCsv::export(db, private_key)
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(ContextResponse::from((candidates, context)))
}

#[get("/candidate/<id>")]
pub async fn get_candidate(
    conn: Connection<'_, Db>,
    session: AdminAuth,
    id: BBox<i32, FakePolicy>,
    context: Context<ContextDataType>
) -> MyResult<JsonResponse<ApplicationDetails, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let private_key = session.get_private_key();

    println!("a");

    let application = Query::find_application_by_id(db, id)
        .await
        .map_err(|e| to_custom_error(ServiceError::DbError(e)))?
        .ok_or(to_custom_error(ServiceError::CandidateNotFound))?;

        println!("b");
    
    let details = ApplicationService::decrypt_all_details(
        private_key,
        db,
        &application
    )
        .await
        .map_err(to_custom_error)?;
    println!("c");
    MyResult::Ok(
        JsonResponse::from((details, context))
    )
}

#[route(DELETE, "/candidate/<id>")]
pub async fn delete_candidate(
    conn: Connection<'_, Db>,
    _session: AdminAuth,
    id: BBox<i32, FakePolicy>,
    context: Context<ContextDataType>
) -> Result<(), (rocket::http::Status, String)> {
    let db = conn.into_inner();

    let application = Query::find_application_by_id(db, id)
        .await
        .map_err(|e| to_custom_error(ServiceError::DbError(e)))?
        .ok_or(to_custom_error(ServiceError::CandidateNotFound))?;


    ApplicationService::delete(context, db, application)
        .await
        .map_err(to_custom_error)

}

#[post("/candidate/<id>/reset_password")]
pub async fn reset_candidate_password(
    conn: Connection<'_, Db>,
    session: AdminAuth,
    id: BBox<i32, FakePolicy>,
    context: Context<ContextDataType>
) -> MyResult<JsonResponse<CreateCandidateResponse, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let private_key = session.get_private_key();

    let response = ApplicationService::reset_password(context.clone(), private_key, db, id)
        .await
        .map_err(to_custom_error)?;
    
    MyResult::Ok(
        JsonResponse::from((response, context))
    )
}

#[get("/candidate/<id>/portfolio")]
pub async fn get_candidate_portfolio(
    conn: Connection<'_, Db>,
    session: AdminAuth, 
    id: BBox<i32, FakePolicy>,
    context: Context<ContextDataType>
) -> MyResult<Vec<u8>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let private_key = session.get_private_key();

    let application = Query::find_application_by_id(db, id)
        .await
        .map_err(|e| to_custom_error(ServiceError::DbError(e)))?
        .ok_or(to_custom_error(ServiceError::CandidateNotFound))?;

    let portfolio = PortfolioService::get_portfolio(context, application.candidate_id, private_key)
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(portfolio)
}

#[cfg(test)]
pub mod tests {
    use core::panic;

    use entity::admin;
    use portfolio_core::models::{application::{ApplicationResponse, CleanApplicationResponse}, candidate::CleanCreateCandidateResponse};
    use rocket::{http::{Cookie, Status}, local::blocking::Client};

    use crate::{routes::candidate::tests::CleanApplicationDetails, test::tests::{test_client, ADMIN_ID, ADMIN_PASSWORD}};

    pub fn admin_login(client: &Client) -> (Cookie, Cookie) {
        let _ = client.post("/admin/logout").dispatch();
        let response = client
            .post("/admin/login")
            .body(format!(
                "{{
            \"adminId\": {},
            \"password\": \"{}\"
        }}",
                ADMIN_ID, ADMIN_PASSWORD
            ))
            .dispatch();

        println!("res {:?}", response);
        println!("id cookee {:?}", response.cookies().get("id"));
        println!("key cookee {:?}", response.cookies().get("key"));

        (
            response.cookies().get("id").unwrap().to_owned(),
            response.cookies().get("key").unwrap().to_owned(),
        )
    }

    fn create_candidate(
        client: &Client,
        cookies: (Cookie, Cookie),
        id: i32,
        pid: String,
    ) -> CleanCreateCandidateResponse {
        let response = client
            .post("/admin/create")
            .body(format!(
                "{{
            \"applicationId\": {},
            \"personalIdNumber\": \"{}\"
        }}",
                id, pid
            ))
            .cookie(cookies.0)
            .cookie(cookies.1)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);

        response.into_json::<CleanCreateCandidateResponse>().unwrap()
    }

    fn check_incomplete_candidate(
        client: &Client,
        cookies: (Cookie, Cookie),
        id: i32,
    ) {
        let response = client
            .get(format!("/admin/candidate/{id}"))
            .cookie(cookies.0)
            .cookie(cookies.1)
            .dispatch();

        // we want to get 406 bc the portfolio is incomplete
        assert_eq!(response.status(), Status::from_code(406).unwrap());
    }

    fn get_candidate_info(
        client: &Client,
        cookies: (Cookie, Cookie),
        id: i32,
    ) -> CleanApplicationDetails {
        let response = client
            .get(format!("/admin/candidate/{id}"))
            .cookie(cookies.0)
            .cookie(cookies.1)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        response.into_json::<CleanApplicationDetails>().unwrap()
    }

    fn list_candidates(
        client: &Client,
        cookies: (Cookie, Cookie)
    ) -> Vec<CleanApplicationResponse> {
        let response = client
            .get("/admin/list/candidates")
            .cookie(cookies.0)
            .cookie(cookies.1)
            .dispatch();

        assert_eq!(response.status(), Status::Ok);
        response.into_json::<Vec<CleanApplicationResponse>>().unwrap()
    }

    // fn csv_export(
    //     client: &Client,
    //     cookies: (Cookie, Cookie)
    // ) -> Vec<u8> {
    //     let response = client
    //         .get("/admin/list/candidates_csv")
    //         .cookie(cookies.0)
    //         .cookie(cookies.1)
    //         .dispatch();

    //     assert_eq!(response.status(), Status::Ok);
    //     response.into_json::<Vec<u8>>().unwrap()
    // }

    #[test]
    fn test_create_candidate() {
        let client = test_client().lock().unwrap();
        let cookies = admin_login(&client);
        let response = create_candidate(&client, cookies, 1031511, "0".to_string());
    
        assert_eq!(response.password.len(), 12);
    }

    #[test]
    // Added by aportlan for additional Sesame testing
    fn test_create_list_candidates() {
        let client = test_client().lock().unwrap();
        let to_create = vec![(1013132, "4"), (1013133, "1"), (1024193, "2"), (1015678, "9"), (1013456, "12"), (1021234, "23")];
        
        // add all candidates to system
        for (app_id, pid) in to_create.clone() {
            let cookies = admin_login(&client);

            let response = create_candidate(&client, cookies.clone(), app_id, pid.to_string());
            assert_eq!(response.password.len(), 12);
        }

        // get a list of candidates
        let cookies = admin_login(&client);
        let response = list_candidates(&client, cookies);

        // assert_eq!(response.len(), 0);
        // make sure they all show up in system
        for (app_id, pid) in to_create {
            let matches = response.iter().filter(|app|{
                app.personal_id_number == pid && app.application_id == app_id
            }).count();
            assert!(matches >= 1);
        }
    }

    #[test]
    // Added by aportlan for additional Sesame testing
    fn test_add_get_candidates() {
        let client = test_client().lock().unwrap();
        let to_create = vec![(1019132, "40"), (1019133, "10"), (1029193, "20"), (1019678, "90"), (1019456, "120"), (1029234, "230")];
        
        for (app_id, pid) in to_create {
            let cookies = admin_login(&client);

            let response = create_candidate(&client, cookies.clone(), app_id, pid.to_string());
            assert_eq!(response.password.len(), 12);

            // test the candidate exists, but is incomplete
            check_incomplete_candidate(&client, cookies, app_id);
        }
    }

    #[test]
    // Added by aportlan for additional Sesame testing
    fn test_get_candidate_details() {
        let client = test_client().lock().unwrap();
        let candidate_cookies = crate::routes::candidate::tests::candidate_login(&client);

        // Post candidate details
        let details_orig: CleanApplicationDetails = serde_json::from_str(crate::routes::candidate::tests::CANDIDATE_DETAILS).unwrap();

        let response = client
            .post("/candidate/details")
            .cookie(candidate_cookies.0.clone())
            .cookie(candidate_cookies.1.clone())
            .body(crate::routes::candidate::tests::CANDIDATE_DETAILS.to_string())
            .dispatch();
        assert_eq!(response.status(), Status::Ok);

        let admin_cookies = admin_login(&client);
        let details_new = get_candidate_info(&client, admin_cookies, crate::test::tests::APPLICATION_ID);

        assert_eq!(details_orig, details_new);
    }

    // NOTE: (aportlan) tabling this test for now bc any incomplete candidates mess up the datetime
    // #[test]
    // fn test_list_csv() {
    //     let client = test_client().lock().unwrap();
    //     let to_create: Vec<(i32, &str)> = vec![(1012345, "111"), (1023456, "123"), (1034567, "135")];

    //     for (app_id, pid) in to_create.clone() {
    //         let cookies = admin_login(&client);

    //         let response = create_candidate(&client, cookies.clone(), app_id, pid.to_string());
    //         assert_eq!(response.password.len(), 12);
    //     }

    //     let cookies = admin_login(&client);
    //     let response = csv_export(&client, cookies);
    //     let response = String::from_utf8(response).unwrap();

    //     for (app_id, pid) in to_create.clone() {
    //         println!("{}", response);
    //         panic!();
    //     }
    // }
}