use alohomora::{bbox::BBox, AlohomoraType};
use crate::{data::CandidateDataPolicy, FakePolicy};

#[allow(non_snake_case)]
#[derive(alohomora_derive::RequestBBoxJson, AlohomoraType, Clone, Debug)]
#[alohomora_out_type(to_derive = [Debug, Clone])]
pub struct AdminLoginRequest {
    pub adminId: BBox<i32, FakePolicy>,
    pub password: BBox<String, FakePolicy>,
}

#[allow(non_snake_case)]
#[derive(alohomora_derive::RequestBBoxJson, AlohomoraType, Clone, Debug)]
#[alohomora_out_type(to_derive = [Debug, Clone])]
pub struct LoginRequest {
    pub applicationId: BBox<i32, CandidateDataPolicy>,
    pub password: BBox<String, CandidateDataPolicy>,
}