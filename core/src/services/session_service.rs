use std::cmp::min;
use alohomora::pure::PrivacyPureRegion;

use entity::{session_trait::UserSession};
use sea_orm::{DbConn, ActiveModelTrait, ActiveModelBehavior};

use crate::{
    error::ServiceError,
    Mutation,
};

pub(in crate::services) struct SessionService;

impl SessionService {
    /// Check if session is valid
    pub async fn is_valid<T>(session: &T) -> Result<bool, ServiceError> where T: UserSession {
        let now = chrono::Utc::now().naive_utc();
        let result = session.expires_at().await.into_ppr(PrivacyPureRegion::new(|expiry| {
            if now >= expiry {
                Err(())
            } else {
                Ok(())
            }
        }));

        Ok(result.transpose().is_ok())
    }

    /// Delete list of sessions
    pub async fn delete_sessions<T>(db: &DbConn, sessions: Vec<T>, keep_n_recent: usize) -> Result<(), ServiceError> where T: ActiveModelTrait + std::marker::Send + ActiveModelBehavior {
        for session in sessions
            .iter()
            .take(sessions.len() - min(sessions.len(), keep_n_recent))
        {
            Mutation::delete_session(db, session.to_owned()).await?;
        }

        Ok(())

    }
}

#[cfg(test)]
mod tests {
    use alohomora::{bbox::BBox, context::Context, pcr::{execute_pcr, PrivacyCriticalRegion, Signature}, policy::NoPolicy, testing::TestContextData};
    use sea_orm::{
        prelude::Uuid,
    };
    use portfolio_policies::{context::ContextDataType, FakePolicy};

    use crate::{
        crypto, models::auth::AuthenticableTrait, services::application_service::ApplicationService, utils::{self, db::get_memory_sqlite_connection}
    };
    const SECRET: &str = "Tajny_kod";

    fn get_test_context() -> Context<TestContextData<ContextDataType>> {
        Context::test(ContextDataType{
            session_id: Some(BBox::new(utils::db::TESTING_ADMIN_COOKIE.to_string(), NoPolicy::new())),
            key: Some(BBox::new(utils::db::TESTING_ADMIN_KEY.to_string(), NoPolicy::new())),
        })
    }

    #[tokio::test]
    async fn test_create_candidate() {

        let db = get_memory_sqlite_connection().await;

        let application = ApplicationService::create(
            get_test_context(),
            &BBox::new("".to_string(), FakePolicy::new()),
            &db, 
            BBox::new(103151, FakePolicy::new()),
            &BBox::new(SECRET.to_string(), FakePolicy::new()),
            BBox::new("".to_string(), FakePolicy::new())).await.unwrap().0;

        let (id, password) = execute_pcr((application.id, application.password), 
        PrivacyCriticalRegion::new(|(id, password), _, _| {
            (id, password)
        },
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}, 
        Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap();
        assert_eq!(id, 103151);
        assert_ne!(password, SECRET.to_string());
        assert!(crypto::verify_password(SECRET.to_string(), password)
            .await
            .ok()
            .unwrap());
    }

    #[tokio::test]
    async fn test_candidate_session_correct_password() {
        let db = &get_memory_sqlite_connection().await;

        let application = ApplicationService::create(
            get_test_context(),
            &BBox::new("".to_string(), FakePolicy::new()),
            &db, 
            BBox::new(103151, FakePolicy::new()),
            &BBox::new(SECRET.to_string(), FakePolicy::new()),
            BBox::new("".to_string(), FakePolicy::new())).await.unwrap().0;

        // correct password
        let session = ApplicationService::new_session(
            db,
            &application,
            BBox::new(SECRET.to_string(), FakePolicy::new()),
            BBox::new("127.0.0.1".to_string(), FakePolicy::new()),
        )
        .await
        .unwrap();
        let session = execute_pcr(session, 
            PrivacyCriticalRegion::new(|s, _, _|{s},
                Signature{username: "AlexanderPortland", signature: ""}, 
                Signature{username: "AlexanderPortland", signature: ""}, 
                Signature{username: "AlexanderPortland", signature: ""}), ()).unwrap();
        assert!(
            ApplicationService::auth(db, BBox::new(Uuid::parse_str(&session).unwrap(), FakePolicy::new()))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_candidate_session_incorrect_password() {
        let db = &get_memory_sqlite_connection().await;

        let application = ApplicationService::create(
            get_test_context(),
            &BBox::new("".to_string(), FakePolicy::new()),
            &db, 
            BBox::new(103151, FakePolicy::new()),
            &BBox::new(SECRET.to_string(), FakePolicy::new()),
            BBox::new("".to_string(), FakePolicy::new())).await.unwrap().0;

        // incorrect password
        assert!(ApplicationService::new_session(
            db,
            &application,
            BBox::new("Spatny_kod".to_string(), FakePolicy::new()),
            BBox::new("127.0.0.1".to_string(), FakePolicy::new())
        )
        .await
        .is_err());
    }
}
