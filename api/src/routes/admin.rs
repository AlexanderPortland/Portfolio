use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use portfolio_core::{
    crypto::random_12_char_string, error::ServiceError, models::{application::ApplicationResponse, auth::AuthenticableTrait, candidate::{ApplicationDetails, CreateCandidateResponse}}, Query, sea_orm::prelude::Uuid, services::{admin_service::AdminService, application_service::ApplicationService, portfolio_service::PortfolioService}
};
use requests::{AdminLoginRequest, RegisterRequest};
use rocket::http::Status;


use portfolio_core::policies::context::ContextDataType;
use alohomora::{bbox::BBox, context::Context, orm::Connection, policy::NoPolicy, pure::{execute_pure, PrivacyPureRegion}, rocket::{BBoxCookie, BBoxCookieJar, BBoxJson, ContextResponse, get, JsonResponse, post, route}};


use portfolio_core::utils::csv::{ApplicationCsv, CandidateCsv, CsvExporter};
use portfolio_core::utils::response::MyResult;

use crate::{guards::request::auth::AdminAuth, pool::Db, requests};

use super::to_custom_error;

#[post("/login", data = "<login_form>")]
pub async fn login(
    conn: Connection<'_, Db>,
    login_form: BBoxJson<AdminLoginRequest>,
    // ip_addr: SocketAddr, // TODO uncomment in production
    cookies: BBoxCookieJar<'_, '_>,
    context: Context<ContextDataType>
) -> Result<(), (rocket::http::Status, String)> {
    let ip_addr: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let db = conn.into_inner();
    let session_token_key = AdminService::login(
        db,
        login_form.adminId.clone(),
        login_form.password.clone(),
        BBox::new(ip_addr.ip().to_string(), NoPolicy::new()),
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

    let cookie: Option<BBoxCookie<'static, NoPolicy>> = cookies.get("id");
     // unwrap would be safe here because of the auth guard
    let cookie_unwrapped: BBoxCookie<'static, NoPolicy> = cookie.ok_or((Status::Unauthorized, "No session cookie".to_string()))?;
    // let session_id = cookie.value().into_ppr(PrivacyPureRegion::new(|c|{ 
    //         Uuid::try_parse(c).unwrap()
    //     }));
    let cookie_value = cookie_unwrapped.value().to_owned();
    let session_id = execute_pure(cookie_value, PrivacyPureRegion::new(|c: String|{ 
        Uuid::try_parse(c.as_str()).unwrap()
    })).unwrap().specialize_policy().unwrap();
    
    // let session_id = Uuid::try_parse(cookie.value()) // unwrap would be safe here because of the auth guard
    //     .map_err(|e| Custom(Status::BadRequest, e.to_string()))?;
    let session = Query::find_admin_session_by_uuid(db, session_id).await.unwrap().unwrap();
    
    let _res = AdminService::logout(db, session)
        .await
        .map_err(to_custom_error)?;

    cookies.remove(cookies.get::<NoPolicy>("id").unwrap());
    cookies.remove(cookies.get::<NoPolicy>("key").unwrap());

    Ok(())
}


#[get("/whoami")]
//pub async fn whoami(session: AdminAuth, context: Context<ContextDataType>) -> Result<String, (rocket::http::Status, String)> {
pub async fn whoami(session: AdminAuth, context: Context<ContextDataType>) -> MyResult<ContextResponse<String, NoPolicy, ContextDataType>, (rocket::http::Status, String)> {
    let admin: entity::admin::Model = session.into();

    let a: BBox<String, NoPolicy> = execute_pure(admin.id, PrivacyPureRegion::new(
        |n: i32|{n.to_string()}
    )).unwrap().specialize_policy().unwrap();
    MyResult::Ok(ContextResponse(a, context))
}

#[get("/hello")]
pub async fn hello(_session: AdminAuth, _context: Context<ContextDataType>) -> Result<String, (rocket::http::Status, String)> {
                                                                            //       ^^ should this be a bbox string
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

    println!("im in admin rn");

    let plain_text_password = BBox::new(random_12_char_string(), NoPolicy::new());
    //println!("trying to did the thing");

    let (application, applications, personal_id_number) = match ApplicationService::create(
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
        application_id: application.id,
        field_of_study: application.field_of_study,
        applications: applications.iter()
            .map(|a| a.id.to_owned())
            .collect(),
        personal_id_number,
        password: plain_text_password,
    };

    MyResult::Ok(JsonResponse::from((cand, context)))
}





#[allow(unused_variables)]
#[get("/candidates?<field>&<page>&<sort>")]
pub async fn list_candidates(
    conn: Connection<'_, Db>,
    session: AdminAuth,
    field: Option<BBox<String, NoPolicy>>,
    page: Option<BBox<u64, NoPolicy>>,
    sort: Option<BBox<String, NoPolicy>>, // how to do this part
    context: Context<ContextDataType>
) -> MyResult<JsonResponse<Vec<ApplicationResponse>, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let private_key = session.get_private_key();
    
    if let Some(field) = field.clone() {
        let field = field.discard_box();
        if !(field == "KB".to_string() || field == "IT".to_string() || field == "G") {
            return MyResult::Err((Status::BadRequest, "Invalid field of study".to_string()));
        }
    }

    //let field_bbox = alohomora::fold::fold(field).unwrap().specialize_policy().unwrap();
    //let page_bbox = alohomora::fold::fold(page).unwrap().specialize_policy().unwrap();
    //let sort_bbox = alohomora::fold::fold(sort).unwrap().specialize_policy().unwrap();

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
) -> MyResult<ContextResponse<Vec<u8>, NoPolicy, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let private_key = session.get_private_key();

    let candidates = ApplicationCsv::export(context.clone(), db, private_key)
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(
        ContextResponse(candidates, context)
    )
}

#[get("/admissions_csv")]
pub async fn list_admissions_csv(
    conn: Connection<'_, Db>,
    session: AdminAuth,
    context: Context<ContextDataType>
) -> MyResult<ContextResponse<Vec<u8>, NoPolicy, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let private_key = session.get_private_key();
    //let context = todo!();

    let candidates = CandidateCsv::export(context.clone(), db, private_key)
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(ContextResponse(candidates, context))
}

#[get("/candidate/<id>")]
pub async fn get_candidate(
    conn: Connection<'_, Db>,
    session: AdminAuth,
    id: BBox<i32, NoPolicy>,
    context: Context<ContextDataType>
) -> MyResult<JsonResponse<ApplicationDetails, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let private_key = session.get_private_key();

    let application = Query::find_application_by_id(db, id)
        .await
        .map_err(|e| to_custom_error(ServiceError::DbError(e)))?
        .ok_or(to_custom_error(ServiceError::CandidateNotFound))?;
    
    let details = ApplicationService::decrypt_all_details(
        private_key,
        db,
        &application
    )
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(
        JsonResponse::from((details, context))
    )
}

#[route(DELETE, "/candidate/<id>")]
pub async fn delete_candidate(
    conn: Connection<'_, Db>,
    _session: AdminAuth,
    id: BBox<i32, NoPolicy>,
) -> Result<(), (rocket::http::Status, String)> {
    let db = conn.into_inner();

    let application = Query::find_application_by_id(db, id)
        .await
        .map_err(|e| to_custom_error(ServiceError::DbError(e)))?
        .ok_or(to_custom_error(ServiceError::CandidateNotFound))?;


    ApplicationService::delete(db, application)
        .await
        .map_err(to_custom_error)

}

#[post("/candidate/<id>/reset_password")]
pub async fn reset_candidate_password(
    conn: Connection<'_, Db>,
    session: AdminAuth,
    id: BBox<i32, NoPolicy>,
    context: Context<ContextDataType>
) -> MyResult<JsonResponse<CreateCandidateResponse, ContextDataType>, (rocket::http::Status, String)> {
    // TODO
    let db = conn.into_inner();
    let private_key = session.get_private_key();

    let response = ApplicationService::reset_password(private_key, db, id)
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
    id: BBox<i32, NoPolicy>,
    context: Context<ContextDataType>
) -> MyResult<ContextResponse<Vec<u8>, NoPolicy, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let private_key = session.get_private_key();

    let application = Query::find_application_by_id(db, id)
        .await
        .map_err(|e| to_custom_error(ServiceError::DbError(e)))?
        .ok_or(to_custom_error(ServiceError::CandidateNotFound))?;

    let portfolio = PortfolioService::get_portfolio(application.candidate_id.discard_box(), private_key.discard_box())
        .await
        .map_err(to_custom_error)?;
    let portfolio = BBox::new(portfolio, NoPolicy::new());

    MyResult::Ok(ContextResponse(portfolio, context))
}

#[cfg(test)]
pub mod tests {
    use portfolio_core::models::candidate::CleanCreateCandidateResponse;
    use rocket::{http::{Cookie, Status}, local::blocking::Client};

    use crate::test::tests::{ADMIN_ID, ADMIN_PASSWORD, test_client};

    pub fn admin_login(client: &Client) -> (Cookie, Cookie) {
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

        println!("got response {:?}", response);

        response.into_json::<CleanCreateCandidateResponse>().unwrap()
    }

    #[test]
    fn test_create_candidate() {
        let client = test_client().lock().unwrap();
        let cookies = admin_login(&client);
        println!("+++got some cookies {:?}", cookies);
        let response = create_candidate(&client, cookies, 1031511, "0".to_string());
    
        assert_eq!(response.password.len(), 12);
    }
}