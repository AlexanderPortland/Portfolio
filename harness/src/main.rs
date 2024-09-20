use std::sync::{Arc, Mutex};

use alohomora::testing::BBoxClient;
use portfolio_api::*;
use portfolio_core::models::{application::CleanApplicationResponse, candidate::CleanCreateCandidateResponse};
use rocket::{http::{Cookie, Header, Status}, local::blocking::Client};

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

pub fn candidate_login(client: &Client, id: i32, password: String) -> (Cookie, Cookie) {
    let response = client
        .post("/candidate/login")
        .body(format!(
            "{{
        \"applicationId\": {},
        \"password\": \"{}\"
    }}",
            id, password
        ))
        .dispatch();

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

fn make_candidates(client: &Client, ids: Vec<i32>) -> Vec<(i32, String)> {
    let mut cands = Vec::new();
    let cookies = admin_login(&client);

    for id in ids {
        let response = create_candidate(&client, cookies.clone(), id, "0".to_string());
        // println!("res is {:?}", response);
        cands.push((id, response.password));
    }
    // println!("{:?} successes!", cands);
    cands
}

fn do_list_candidates(
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

fn list_candidates(client: &Client, len: usize) {
    let cookies = admin_login(&client);
    let res = do_list_candidates(&client, cookies);
    assert_eq!(res.len(), len + 1);
}

fn upload_letters(client: &Client, cands: Vec<(i32, String)>, letter: Vec<u8>) {
    for (id, password) in cands {
        // login
        let cookies = candidate_login(&client, id, password);


        // post letter
        let response = client
            .post("/candidate/add/portfolio_letter")
            .cookie(cookies.0.clone())
            .cookie(cookies.1.clone())
            .body(letter.clone()) // TODO: this clone is probably shitty
            .header(Header::new("Content-Type", "application/pdf"))
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}

fn read_portfolio(filename: String) -> Vec<u8> {
    let mut f = std::fs::File::open(&filename).expect("no file found");
    let metadata = std::fs::metadata(&filename).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    std::io::Read::read(&mut f, &mut buffer).expect("buffer overflow");
    assert_eq!(buffer.len(), 681555);

    buffer
}

fn main(){
    // setup
    let PORTFOLIO = read_portfolio("../cover_letter.pdf".to_string());
    let client = get_portfolio();
    
    let ids: Vec<i32> = (103152..103260).collect();
    let ids_len = ids.len();

    let candidates = make_candidates(&client, ids);
    upload_letters(&client, candidates, PORTFOLIO);
    list_candidates(&client, ids_len);
}