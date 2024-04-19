#[cfg(test)]
pub mod tests {
    use crate::rocket;
    use alohomora::{bbox::BBox, testing::BBoxClient};
    use portfolio_policies::FakePolicy;
    use entity::admin;
    use once_cell::sync::OnceCell;
    use portfolio_core::{
        crypto,
        sea_orm::{ActiveModelTrait, DbConn, Set},
        services::application_service::ApplicationService,
    };
    
    use std::sync::Mutex;

    pub const ADMIN_ID: i32 = 1;
    pub const ADMIN_PASSWORD: &'static str = "test";

    pub const APPLICATION_ID: i32 = 103151;
    pub const CANDIDATE_PASSWORD: &'static str = "test";
    pub const PERSONAL_ID_NUMBER: &'static str = "0101010000";

    pub async fn run_test_migrations(db: &DbConn) {
        let (pubkey, priv_key) = crypto::create_identity();
        let priv_key = crypto::encrypt_password(priv_key, ADMIN_PASSWORD.to_string())
            .await
            .unwrap();
        let password_hash = crypto::hash_password(ADMIN_PASSWORD.to_string())
            .await
            .unwrap();

        admin::ActiveModel {
            id: Set(BBox::new(ADMIN_ID, FakePolicy::new())),
            name: Set(BBox::new("admin pepa".to_string(), FakePolicy::new())),
            public_key: Set(BBox::new(pubkey, FakePolicy::new())),
            private_key: Set(BBox::new(priv_key, FakePolicy::new())),
            password: Set(BBox::new(password_hash, FakePolicy::new())),
            created_at: Set(BBox::new(chrono::Utc::now().naive_utc(), FakePolicy::new())),
            updated_at: Set(BBox::new(chrono::Utc::now().naive_utc(), FakePolicy::new())),
        }
        .insert(db)
        .await
        .unwrap();

        ApplicationService::create(
            todo!(),
            &BBox::new("".to_string(), FakePolicy::new()),
            db,
            BBox::new(APPLICATION_ID, FakePolicy::new()),
            &BBox::new(CANDIDATE_PASSWORD.to_string(), FakePolicy::new()),
            BBox::new(PERSONAL_ID_NUMBER.to_string(), FakePolicy::new()))
            .await.unwrap();
    }

    pub fn test_client() -> &'static Mutex<BBoxClient> {
        static INSTANCE: OnceCell<Mutex<BBoxClient>> = OnceCell::new();
        INSTANCE.get_or_init(|| {
            let rocket = rocket();
            Mutex::from(BBoxClient::tracked(rocket).expect("valid rocket instance"))
        })
    }
}
