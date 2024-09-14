use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use alohomora::pcr::{execute_pcr, PrivacyCriticalRegion, Signature};
use alohomora::rocket::{ContextResponse, JsonResponse};
use alohomora::{bbox::BBox, context::Context, orm::Connection, pure::{execute_pure, PrivacyPureRegion}, rocket::{get, post, route, BBoxCookie, BBoxCookieJar, BBoxJson}};
use alohomora_derive::{RequestBBoxJson, ResponseBBoxJson};
use chrono::NaiveDate;
use entity::application;
use portfolio_policies::data::CandidateDataPolicy;
use crate::pool::ContextDataType;
use portfolio_core::utils::response::MyResult;
use portfolio_core::Query;
use portfolio_core::error::ServiceError;
use portfolio_core::models::auth::AuthenticableTrait;
use portfolio_core::models::candidate::{ApplicationDetails, CandidateDetails, NewCandidateResponse, ParentDetails};
use portfolio_core::models::grade::GradeList;
use portfolio_core::models::school::School;
use portfolio_core::sea_orm::prelude::Uuid;
use portfolio_core::services::application_service::ApplicationService;
use portfolio_core::services::portfolio_service::PortfolioService;
use portfolio_policies::FakePolicy;
use requests::LoginRequest;
use crate::guards::data::letter::Letter;
use crate::guards::data::portfolio::Portfolio;
use crate::{guards::request::auth::ApplicationAuth, pool::Db, requests};

use super::to_custom_error;

#[post("/login", data = "<login_form>")]
pub async fn login(
    conn: Connection<'_, Db>,
    login_form: BBoxJson<LoginRequest>,
    //ip_addr: BBox<SocketAddr, FakePolicy>,
    cookies: BBoxCookieJar<'_, '_>,
    context: Context<ContextDataType>
) -> MyResult<(), (rocket::http::Status, String)> {
    let ip_addr = BBox::new(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0),
        FakePolicy::new());
    let ip_addr = ip_addr.into_ppr(PrivacyPureRegion::new(|ip: SocketAddr| {
        ip.to_string()
    }));

    let db = conn.into_inner();

    let res = ApplicationService::login(
        db,
        login_form.applicationId.clone().into_any_policy(),
        login_form.password.clone().into_any_policy(),
        ip_addr,
    ).await.map_err(to_custom_error);

    let (session_token, private_key) = match res {
        Ok(a) => a,
        Err(e) => return MyResult::Err(e),
    };

    let _ = cookies.add(BBoxCookie::new("id", session_token.clone()), context.clone());
    let _ = cookies.add(BBoxCookie::new("key", private_key.clone()), context.clone());

    return MyResult::Ok(());
}

#[post("/logout")]
pub async fn logout(
    conn: Connection<'_, Db>,
    _session: ApplicationAuth,
    cookies: BBoxCookieJar<'_, '_>,
    _context: Context<ContextDataType>
) -> MyResult<(), (rocket::http::Status, String)> {
    let db = conn.into_inner();

    let cookie: BBoxCookie<'static, FakePolicy> = cookies
        .get("id").unwrap(); // unwrap would be safe here because of the auth guard
        //.ok_or(Custom(Status::Unauthorized,"No session cookie".to_string(),))?;

    let session_id = execute_pure(cookie.value().to_owned(), PrivacyPureRegion::new(|c: String|{ 
        Uuid::try_parse(c.as_str()).unwrap()
    })).unwrap().specialize_policy().unwrap();

    let session = Query::find_session_by_uuid(db, session_id).await.unwrap().unwrap(); // TODO
    ApplicationService::logout(db, session)
        .await
        .map_err(to_custom_error)?;

    cookies.remove(cookies.get::<FakePolicy>("id").unwrap());
    cookies.remove(cookies.get::<FakePolicy>("key").unwrap());

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
#[allow(non_snake_case)]
#[derive(Debug, Clone, PartialEq, RequestBBoxJson)]
pub struct RequestCandidateDetails {
    pub name: BBox<String, CandidateDataPolicy>,
    pub surname: BBox<String, CandidateDataPolicy>,
    pub birthSurname: BBox<String, CandidateDataPolicy>,
    pub birthplace: BBox<String, CandidateDataPolicy>,
    pub birthdate: BBox<NaiveDate, CandidateDataPolicy>,
    pub address: BBox<String, CandidateDataPolicy>,
    pub letterAddress: BBox<String, CandidateDataPolicy>,
    pub telephone: BBox<String, CandidateDataPolicy>,
    pub citizenship: BBox<String, CandidateDataPolicy>,
    pub email: BBox<String, CandidateDataPolicy>,
    pub sex: BBox<String, CandidateDataPolicy>,
    pub personalIdNumber: BBox<String, CandidateDataPolicy>,
    pub schoolName: BBox<String, CandidateDataPolicy>,
    pub healthInsurance: BBox<String, CandidateDataPolicy>,
    pub grades: BBox<GradeList, CandidateDataPolicy>,
    pub firstSchool: BBox<School, CandidateDataPolicy>,
    pub secondSchool: BBox<School, CandidateDataPolicy>,
    pub testLanguage: BBox<String, CandidateDataPolicy>,
}
impl RequestCandidateDetails {
    pub fn validate_self(&self) -> Result<(), ServiceError> {
        Ok(())
    }
}
impl RequestCandidateDetails {
    pub fn to_any(self) -> CandidateDetails {
        CandidateDetails {
            name: self.name.into_any_policy(),
            surname: self.surname.into_any_policy(),
            birthSurname: self.birthSurname.into_any_policy(),
            birthplace: self.birthplace.into_any_policy(),
            birthdate: self.birthdate.into_any_policy(),
            address: self.address.into_any_policy(),
            letterAddress: self.letterAddress.into_any_policy(),
            telephone: self.telephone.into_any_policy(),
            citizenship: self.citizenship.into_any_policy(),
            email: self.email.into_any_policy(),
            sex: self.sex.into_any_policy(),
            personalIdNumber: self.personalIdNumber.into_any_policy(),
            schoolName: self.schoolName.into_any_policy(),
            healthInsurance: self.healthInsurance.into_any_policy(),
            grades: self.grades.into_any_policy(),
            firstSchool: self.firstSchool.into_any_policy(),
            secondSchool: self.secondSchool.into_any_policy(),
            testLanguage: self.testLanguage.into_any_policy(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, RequestBBoxJson)]
pub struct RequestParentDetails {
    pub name: BBox<String, CandidateDataPolicy>,
    pub surname: BBox<String, CandidateDataPolicy>,
    pub telephone: BBox<String, CandidateDataPolicy>,
    pub email: BBox<String, CandidateDataPolicy>,
}
impl RequestParentDetails {
    pub fn to_any(self) -> ParentDetails {
        ParentDetails {
            name: self.name.into_any_policy(),
            surname: self.surname.into_any_policy(),
            telephone: self.telephone.into_any_policy(),
            email: self.email.into_any_policy(),
        }
    }
}

#[derive(Debug, Clone, RequestBBoxJson)]
pub struct RequestApplicationDetails {
    pub candidate: RequestCandidateDetails,
    pub parents: Vec<RequestParentDetails>,
}
impl RequestApplicationDetails {
    pub fn to_any(self) -> ApplicationDetails {
        ApplicationDetails {
            candidate: self.candidate.to_any(),
            parents: self.parents.into_iter().map(|b| b.to_any()).collect(),
        }
    }
}

#[post("/details", data = "<details>")]
pub async fn post_details(
    conn: Connection<'_, Db>,
    details: BBoxJson<RequestApplicationDetails>,
    session: ApplicationAuth,
    context: Context<ContextDataType>
) -> MyResult<JsonResponse<ApplicationDetails, ContextDataType>, (rocket::http::Status, String)> {
    let db = conn.into_inner();
    let form = details.into_inner();
    form.candidate.validate_self().map_err(to_custom_error)?;
    let form = form.to_any();

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
    letter: Letter<FakePolicy>,
    context: Context<ContextDataType>,
) -> MyResult<(), (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    let a: BBox<Vec<u8>, FakePolicy> = letter.into();

    PortfolioService::add_cover_letter_to_cache(context, application.candidate_id, a)
        .await
        .map_err(to_custom_error)?;
    MyResult::Ok(())
}

#[route(DELETE, "/cover_letter")]
pub async fn delete_cover_letter(
    session: ApplicationAuth,
    context: Context<ContextDataType>,
) -> MyResult<(), (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    PortfolioService::delete_cover_letter_from_cache(context, application.candidate_id)
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(())
}

#[post("/portfolio_letter", data = "<letter>")]
pub async fn upload_portfolio_letter(
    session: ApplicationAuth,
    letter: Letter<FakePolicy>,
    context: Context<ContextDataType>,
) -> MyResult<(), (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    PortfolioService::add_portfolio_letter_to_cache(context, application.candidate_id, letter.into())
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(())
}

#[route(DELETE, "/portfolio_letter")]
pub async fn delete_portfolio_letter(
    session: ApplicationAuth,
    context: Context<ContextDataType>,
) -> Result<(), (rocket::http::Status, String)> {
    let candidate: entity::application::Model = session.into();

    PortfolioService::delete_portfolio_letter_from_cache(context, candidate.id)
        .await
        .map_err(to_custom_error)?;

    Ok(())
}

#[post("/portfolio_zip", data = "<portfolio>")]
pub async fn upload_portfolio_zip(
    session: ApplicationAuth,
    portfolio: Portfolio<FakePolicy>,
    context: Context<ContextDataType>,
) -> Result<(), (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    PortfolioService::add_portfolio_zip_to_cache(context, application.candidate_id, portfolio.into())
        .await
        .map_err(to_custom_error)?;

    Ok(())
}

#[route(DELETE, "/portfolio_zip")]
pub async fn delete_portfolio_zip(
    session: ApplicationAuth,
    context: Context<ContextDataType>,
) -> Result<(), (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    PortfolioService::delete_portfolio_zip_from_cache(context, application.candidate_id)
        .await
        .map_err(to_custom_error)?;

    Ok(())
}

#[get("/submission_progress")]
pub async fn submission_progress(
    session: ApplicationAuth,
    context: Context<ContextDataType>
) -> MyResult<String, (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    let progress = PortfolioService::get_submission_progress(context, application.candidate_id)
        .map(|x| serde_json::to_string(&x).unwrap())
        .map_err(to_custom_error);
        
    match progress {
        Ok(progress) => MyResult::Ok(progress),
        Err(e) => MyResult::Err(e),
    }
}

#[post("/submit")]
pub async fn submit_portfolio(
    conn: Connection<'_, Db>,
    session: ApplicationAuth,
    context: Context<ContextDataType>
) -> MyResult<(), (rocket::http::Status, String)> {
    let db = conn.into_inner();

    let application: entity::application::Model = session.into();
    let candidate = ApplicationService::find_related_candidate(&db, &application).await.map_err(to_custom_error)?; // TODO

    let submit = PortfolioService::submit(context.clone(), &candidate, &db).await;

    if submit.is_err() {
        let e = submit.err().unwrap();
        // Delete on critical error
        if e.code() == 500 {
            // Cleanup
            PortfolioService::delete_portfolio(context, application.id)
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
    context: Context<ContextDataType>
) -> MyResult<(), (rocket::http::Status, String)> {
    let application: entity::application::Model = session.into();

    PortfolioService::delete_portfolio(context, application.candidate_id)
        .await
        .map_err(to_custom_error)?;

    MyResult::Ok(())
}

#[get("/download")]
pub async fn download_portfolio(
    session: ApplicationAuth,
    context: Context<ContextDataType>,
) -> Result<Vec<u8>, (rocket::http::Status, String)> {
    let private_key = session.get_private_key();
    let application: entity::application::Model = session.into();

    let file = PortfolioService::get_portfolio(context, application.candidate_id, private_key)
        .await
        .map_err(to_custom_error);

    file
}

#[cfg(test)]
pub mod tests {
    use chrono::NaiveDate;
    use portfolio_core::{crypto, models::candidate::CleanNewCandidateResponse, sea_orm::prelude::Uuid};
    use rocket::{
        http::{Cookie, Status},
        local::blocking::Client,
    };
    use rocket::serde::{Deserialize, Serialize};
    use portfolio_core::models::grade::GradeList;
    use portfolio_core::models::school::School;
    use validator::Validate;

    use crate::{
        routes::admin::tests::admin_login,
        test::tests::{test_client, APPLICATION_ID, CANDIDATE_PASSWORD, PERSONAL_ID_NUMBER},
    };

    #[derive(Debug, Serialize, Deserialize, Validate, Clone, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    pub struct CleanCandidateDetails {
        pub name: String,
        pub surname: String,
        pub birth_surname: String,
        pub birthplace: String,
        pub birthdate: NaiveDate,
        pub address: String,
        pub letter_address: String,
        pub telephone: String,
        pub citizenship: String,
        #[validate(email)]
        pub email: String,
        pub sex: String,
        pub personal_id_number: String,
        pub school_name: String,
        pub health_insurance: String,
        pub grades: GradeList,
        pub first_school: School,
        pub second_school: School,
        pub test_language: String,
    }
    #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    pub struct CleanParentDetails {
        pub name: String,
        pub surname: String,
        pub telephone: String,
        pub email: String,
    }
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct CleanApplicationDetails {
        pub candidate: CleanCandidateDetails,
        pub parents: Vec<CleanParentDetails>,
    }

    pub fn candidate_login(client: &Client) -> (Cookie, Cookie) {
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

    pub const CANDIDATE_DETAILS: &'static str = "{
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

    const CANDIDATE_COVER_LETTER: &str = "hello, how are you doing? this is a test cover letter for upload!
    I'd really like to get into high school please, I hope I get admitted! 
    If I don't i'll probably be pretty grumpy and unhappy and stuff. :(
    - idk";

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

    // #[test]
    // fn test_candidate_upload() {
    //     let client = test_client().lock().unwrap();
    //     let cookies = candidate_login(&client);

    //     let response = client
    //         .post("/candidate/add/cover_letter")
    //         .cookie(cookies.0.clone())
    //         .cookie(cookies.1.clone())
    //         .body(CANDIDATE_COVER_LETTER.as_bytes())
    //         .dispatch();
    //     assert_eq!(response.status(), Status::Ok);
    // }

    #[test]
    fn test_invalid_token_every_secured_endpoint() {
        let client = test_client().lock().unwrap();

        let id = Cookie::new("id", Uuid::new_v4().to_string());
        let (private_key, _) = crypto::create_identity();
        let key = Cookie::new("key", private_key);

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
