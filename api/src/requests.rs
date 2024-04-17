use alohomora::{bbox::BBox, policy::NoPolicy};

#[allow(non_snake_case)]
#[derive(alohomora_derive::RequestBBoxJson)]
pub struct LoginRequest {
    pub applicationId: BBox<i32, NoPolicy>,
    pub password: BBox<String, NoPolicy>,
}

#[allow(non_snake_case)]
#[derive(alohomora_derive::RequestBBoxJson)]
pub struct RegisterRequest {
    pub applicationId: BBox<i32, NoPolicy>,
    pub personalIdNumber: BBox<String, NoPolicy>,
}

#[allow(non_snake_case)]
#[derive(alohomora_derive::RequestBBoxJson)]
pub struct AdminLoginRequest {
    pub adminId: BBox<i32, NoPolicy>,
    pub password: BBox<String, NoPolicy>,
}
