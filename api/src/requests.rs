use alohomora::{bbox::BBox, policy::NoPolicy};
use portfolio_policies::{data::CandidateDataPolicy, FakePolicy};

#[allow(non_snake_case)]
#[derive(alohomora_derive::RequestBBoxJson)]
pub struct LoginRequest {
    pub applicationId: BBox<i32, FakePolicy>,
    pub password: BBox<String, FakePolicy>,
}

#[allow(non_snake_case)]
#[derive(alohomora_derive::RequestBBoxJson)]
pub struct RegisterRequest {
    pub applicationId: BBox<i32, CandidateDataPolicy>,
    pub personalIdNumber: BBox<String, CandidateDataPolicy>,
}

#[allow(non_snake_case)]
#[derive(alohomora_derive::RequestBBoxJson)]
pub struct AdminLoginRequest {
    pub adminId: BBox<i32, FakePolicy>,
    pub password: BBox<String, FakePolicy>,
}
