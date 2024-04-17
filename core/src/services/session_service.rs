use std::cmp::min;

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
        if now >= session.expires_at().await.discard_box() {
            Ok(false)
        } else {
            Ok(true)
        }
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
    use alohomora::{bbox::BBox, policy::NoPolicy};
    use sea_orm::{
        prelude::Uuid,
    };

    use crate::{
        crypto,
        services::{application_service::ApplicationService},
        utils::db::get_memory_sqlite_connection, models::auth::AuthenticableTrait,
    };
    const SECRET: &str = "Tajny_kod";

    #[tokio::test]
    async fn test_create_candidate() {

        let db = get_memory_sqlite_connection().await;

        let application = ApplicationService::create(
            &BBox::new("".to_string(), NoPolicy::new()), 
            &db, 
            BBox::new(103151, NoPolicy::new()), 
            &BBox::new(SECRET.to_string(), NoPolicy::new()), 
            BBox::new("".to_string(), NoPolicy::new())).await.unwrap().0;

        assert_eq!(application.id.to_owned().discard_box(), 103151);
        assert_ne!(application.password.to_owned().discard_box(), SECRET.to_string());
        assert!(crypto::verify_password(SECRET.to_string(), application.password.discard_box())
            .await
            .ok()
            .unwrap());
    }

    #[tokio::test]
    async fn test_candidate_session_correct_password() {
        let db = &get_memory_sqlite_connection().await;

        let application = ApplicationService::create(
            &BBox::new("".to_string(), NoPolicy::new()), 
            &db, 
            BBox::new(103151, NoPolicy::new()), 
            &BBox::new(SECRET.to_string(), NoPolicy::new()), 
            BBox::new("".to_string(), NoPolicy::new())).await.unwrap().0;

        // correct password
        let session = ApplicationService::new_session(
            db,
            &application,
            BBox::new(SECRET.to_string(), NoPolicy::new()),
            BBox::new("127.0.0.1".to_string(), NoPolicy::new()),
        )
        .await
        .unwrap();
        assert!(
            ApplicationService::auth(db, BBox::new(Uuid::parse_str(&session.discard_box()).unwrap(), NoPolicy::new()))
                .await
                .is_ok()
        );
    }

    #[tokio::test]
    async fn test_candidate_session_incorrect_password() {
        let db = &get_memory_sqlite_connection().await;

        let application = ApplicationService::create(
            &BBox::new("".to_string(), NoPolicy::new()), 
            &db, 
            BBox::new(103151, NoPolicy::new()), 
            &BBox::new(SECRET.to_string(), NoPolicy::new()), 
            BBox::new("".to_string(), NoPolicy::new())).await.unwrap().0;

        // incorrect password
        assert!(ApplicationService::new_session(
            db,
            &application,
            BBox::new("Spatny_kod".to_string(), NoPolicy::new()),
            BBox::new("127.0.0.1".to_string(), NoPolicy::new())
        )
        .await
        .is_err());
    }
}
