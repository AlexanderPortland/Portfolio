use alohomora::{bbox::BBox, policy::NoPolicy};
use portfolio_policies::{data::CandidateDataPolicy, FakePolicy};



#[allow(non_snake_case)]
#[derive(alohomora_derive::RequestBBoxJson)]
pub struct RegisterRequest {
    pub applicationId: BBox<i32, CandidateDataPolicy>,
    pub personalIdNumber: BBox<String, CandidateDataPolicy>,
}

pub use portfolio_policies::request::LoginRequest;
pub use portfolio_policies::request::AdminLoginRequest;