use alohomora::testing::BBoxClient;
use portfolio_api::*;
use portfolio_core::models::candidate::CleanCreateCandidateResponse;
use rocket::{http::{Cookie, Status}, local::blocking::Client};

fn get_portfolio() -> BBoxClient {
    BBoxClient::tracked(portfolio_api::rocket()).expect("invalid rocket")
}

pub const ADMIN_ID: i32 = 3;
pub const ADMIN_PASSWORD: &'static str = "test";

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
    assert_eq!(response.status(), Status::Ok);
    (response.cookies().get("id").unwrap().to_owned(), response.cookies().get("key").unwrap().to_owned())
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

fn make_candidates(client: &Client) {
    let cookies = admin_login(&client);
    let id_to_make = 103152;
    let mut success = 0;
    for id in 103152..103160 {
        let response = create_candidate(&client, cookies.clone(), id, "0".to_string());
        println!("res is {:?}", response);
        success += 1;
    }
    println!("{success} successes!");
}



fn main(){
    let client = get_portfolio();
    make_candidates(&client);

    // test();
}