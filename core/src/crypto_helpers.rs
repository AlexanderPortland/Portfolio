use std::path::Path;
use alohomora::bbox::BBox;
use alohomora::context::Context;
use alohomora::policy::Policy;
use alohomora::testing::TestContextData;
use argon2::Argon2;
use crate::error::ServiceError;

pub async fn my_hash_password<P: Policy>(password_plain_text: BBox<String, P>) -> Result<BBox<String, P>, ServiceError> {
    todo!()
}

pub async fn my_encrypt_password<P2: Policy, P3: Policy>(
    password_plain_text: String,
    key: BBox<String, P2>
) -> Result<BBox<String, P3>, ServiceError> {
    todo!()
}

pub async fn my_decrypt_password<P1: Policy, P2: Policy, P3: Policy>(
    ciphertext: BBox<String, P1>, key: BBox<String, P2>
) -> Result<BBox<String, P3>, ServiceError> {
    todo!()
}

pub async fn my_encrypt_password_with_recipients<P: Policy, P2: Policy, P3: Policy>(
    password_plain_text: BBox<String, P>,
    recipients: &Vec<BBox<String, P2>>,
) -> Result<BBox<String, P3>, ServiceError> {
    todo!()
}

pub async fn my_decrypt_password_with_private_key<P1: Policy, P2: Policy, P3: Policy>(
    password_encrypted: BBox<String, P1>,
    key: BBox<String, P2>,
) -> Result<BBox<String, P3>, ServiceError> {
    todo!()
}

pub async fn my_verify_password<P1: Policy, P2: Policy>(
    password_plaint_text: BBox<String, P1>,
    hash: BBox<String, P2>,
) -> Result<bool, ServiceError> {
    todo!()
}

pub  fn get_context() -> Context<TestContextData<()>> {
    Context::empty()
}