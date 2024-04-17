use alohomora::{bbox::BBox, policy::NoPolicy};
//use alohomora::rocket::{FromBBoxForm, RequestBBoxJson, ResponseBBoxJson};
//use alohomora_derive::ResponseBBoxJson;




//#[derive(Serialize, Deserialize, alohomora_derive::ResponseBBoxJson)]
#[derive(alohomora_derive::RequestBBoxJson)]
//#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct LoginRequest {
    pub applicationId: BBox<i32, NoPolicy>,
    pub password: BBox<String, NoPolicy>,
}

//#[derive(Serialize, Deserialize, alohomora_derive::ResponseBBoxJson)]
#[derive(alohomora_derive::RequestBBoxJson)]
//#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct RegisterRequest {
    pub applicationId: BBox<i32, NoPolicy>,
    pub personalIdNumber: BBox<String, NoPolicy>,
}

//RequestBBoxJson
//#[derive(Serialize, Deserialize, alohomora_derive::RequestBBoxJson)]
#[derive(alohomora_derive::RequestBBoxJson)]
//#[serde(crate = "rocket::serde", rename_all = "camelCase")]
pub struct AdminLoginRequest {
    pub adminId: BBox<i32, NoPolicy>,
    pub password: BBox<String, NoPolicy>,
}

// impl RequestBBoxJson for AdminLoginRequest {
//     fn from_json(value: alohomora::rocket::InputBBoxValue, request: alohomora::rocket::BBoxRequest<'_, '_>) -> Result<Self, &'static str>
//             where
//                 Self: Sized {
        
//     }
// }