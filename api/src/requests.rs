use alohomora::{bbox::BBox, policy::NoPolicy, rocket::ResponseBBoxJson};
//use alohomora::rocket::{FromBBoxForm, RequestBBoxJson, ResponseBBoxJson};
//use alohomora_derive::ResponseBBoxJson;
use std::collections::HashMap;
use rocket::serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, alohomora_derive::ResponseBBoxJson)]
#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct LoginRequest {
    pub application_id: i32,
    pub password: String,
}

//#[derive(Serialize, Deserialize, alohomora_derive::ResponseBBoxJson)]
#[derive(alohomora_derive::RequestBBoxJson)]
//#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct RegisterRequest {
    pub application_id: BBox<i32, NoPolicy>,
    pub personal_id_number: BBox<String, NoPolicy>,
}

//RequestBBoxJson
//#[derive(Serialize, Deserialize, alohomora_derive::RequestBBoxJson)]
#[derive(alohomora_derive::RequestBBoxJson)]
//#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct AdminLoginRequest {
    pub admin_id: BBox<i32, NoPolicy>,
    pub password: BBox<String, NoPolicy>,
}

// impl RequestBBoxJson for AdminLoginRequest {
//     fn from_json(value: alohomora::rocket::InputBBoxValue, request: alohomora::rocket::BBoxRequest<'_, '_>) -> Result<Self, &'static str>
//             where
//                 Self: Sized {
        
//     }
// }