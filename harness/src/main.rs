use alohomora::testing::BBoxClient;
use portfolio_api::*;
use rocket::{http::{Cookie, Status}, local::blocking::Client};

fn get_portfolio() -> BBoxClient {
    BBoxClient::tracked(portfolio_api::rocket()).expect("invalid rocket")
}

    pub const ADMIN_ID: i32 = 3;
    pub const ADMIN_PASSWORD: &'static str = "test";

pub fn admin_login(client: &Client) -> (Cookie, Cookie) {
    // let _ = client.post("/admin/logout").dispatch();
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
    assert!(response.status() == Status::Ok);
    (response.cookies().get("id").unwrap().to_owned(), response.cookies().get("key").unwrap().to_owned())
}

#[test]
fn test_thing(){
    test();
}

fn main(){
    test();
}

fn test() {
    println!("init");
    let client = get_portfolio();
    println!("init done");
    let to_create = vec![(1019132, "40"), (1019133, "10"), (1029193, "20"), (1019678, "90"), (1019456, "120"), (1029234, "230")];
        
    for (app_id, pid) in to_create {
        let cookies = admin_login(&client);

        // let response = create_candidate(&client, cookies.clone(), app_id, pid.to_string());
        // assert_eq!(response.password.len(), 12);

        // // test the candidate exists, but is incomplete
        // check_incomplete_candidate(&client, cookies, app_id);
    }
}