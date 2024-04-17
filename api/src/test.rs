#[cfg(test)]
pub mod tests {
    use crate::rocket;
    use alohomora::{bbox::BBox, policy::NoPolicy, testing::BBoxClient};
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
        let priv_key = crypto::encrypt_password(priv_key.discard_box(), ADMIN_PASSWORD.to_string())
            .await
            .unwrap();
        let password_hash = crypto::hash_password(ADMIN_PASSWORD.to_string())
            .await
            .unwrap();

        admin::ActiveModel {
            id: Set(BBox::new(ADMIN_ID, NoPolicy::new())),
            name: Set(BBox::new("admin pepa".to_string(), NoPolicy::new())),
            public_key: Set(pubkey),
            private_key: Set(BBox::new(priv_key, NoPolicy::new())),
            password: Set(BBox::new(password_hash, NoPolicy::new())),
            created_at: Set(BBox::new(chrono::Utc::now().naive_utc(), NoPolicy::new())),
            updated_at: Set(BBox::new(chrono::Utc::now().naive_utc(), NoPolicy::new())),
        }
        .insert(db)
        .await
        .unwrap();

        ApplicationService::create(
            &BBox::new("".to_string(), NoPolicy::new()),
            db,
            BBox::new(APPLICATION_ID, NoPolicy::new()),
            &BBox::new(CANDIDATE_PASSWORD.to_string(), NoPolicy::new()),
            BBox::new(PERSONAL_ID_NUMBER.to_string(), NoPolicy::new()))
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
