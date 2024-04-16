use crate::Query;

use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use ::entity::prelude::AdminSession;
use ::entity::{admin, admin_session, application};
use ::entity::{session, session::Entity as Session};
use sea_orm::prelude::Uuid;
use sea_orm::*;

impl Query {
    pub async fn find_session_by_uuid(
        db: &DbConn,
        uuid: BBox<Uuid, NoPolicy>,
    ) -> Result<Option<session::Model>, DbErr> {
        Session::find_by_id(uuid).one(db).await
    }

    pub async fn find_admin_session_by_uuid(
        db: &DbConn,
        uuid: BBox<Uuid, NoPolicy>,
    ) -> Result<Option<admin_session::Model>, DbErr> {
        AdminSession::find_by_id(uuid).one(db).await
    }

    pub async fn find_related_application_sessions(db: &DbConn, application: &application::Model) -> Result<Vec<session::Model>, DbErr> {
        application.find_related(Session)
            .order_by_asc(session::Column::UpdatedAt)
            .all(db)
            .await
    }

    pub async fn find_related_admin_sessions(db: &DbConn, admin: &admin::Model) -> Result<Vec<admin_session::Model>, DbErr> {
        admin.find_related(admin_session::Entity)
            .order_by_asc(admin_session::Column::UpdatedAt)
            .all(db)
            .await
    }
}

#[cfg(test)]
mod tests {
    use alohomora::bbox::BBox;
    use alohomora::policy::NoPolicy;
    use entity::{session, admin, admin_session};
    use sea_orm::{prelude::Uuid, ActiveModelTrait, Set};

    use crate::services::candidate_service::tests::put_user_data;
    use crate::utils::db::get_memory_sqlite_connection;
    use crate::{Query};

    #[tokio::test]
    async fn test_find_session_by_uuid() {
        let db = get_memory_sqlite_connection().await;

        let (application, _, _) = put_user_data(&db).await;
        let session = session::ActiveModel {
            id: Set(BBox::new(Uuid::new_v4(), NoPolicy::new())),
            candidate_id: Set(application.id),
            ip_address: Set(BBox::new("10.10.10.10".to_string(), NoPolicy::new())),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            expires_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new()))
        }
        .insert(&db)
        .await
        .unwrap();

        let session = Query::find_session_by_uuid(&db, session.id).await.unwrap();
        assert!(session.is_some());
    }

    #[tokio::test]
    async fn test_find_sessions_by_user_id() {
        let db = get_memory_sqlite_connection().await;

        const APPLICATION_ID: i32 = 103158;

        let (application, _, _) = put_user_data(&db).await;

        session::ActiveModel {
            id: Set(BBox::new(Uuid::new_v4(), NoPolicy::new())),
            candidate_id: Set(application.id.clone()),
            ip_address: Set(BBox::new("10.10.10.10".to_string(), NoPolicy::new())),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            expires_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            ..Default::default()
        }
            .insert(&db)
            .await
            .unwrap();

        const ADMIN_ID: i32 = 1;

        let admin = admin::ActiveModel {
            id: Set(BBox::new(ADMIN_ID, NoPolicy::new())),
            name: Set(BBox::new("admin".to_string(), NoPolicy::new())),
            public_key: Set(BBox::new("test".to_string(), NoPolicy::new())),
            private_key: Set(BBox::new("test".to_string(), NoPolicy::new())),
            password: Set(BBox::new("test".to_string(), NoPolicy::new())),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            ..Default::default()
        }
            .insert(&db)
            .await
            .unwrap();

        admin_session::ActiveModel {
            id: Set(BBox::new(Uuid::new_v4(), NoPolicy::new())),
            admin_id: Set(admin.id.clone()),
            ip_address: Set(BBox::new("10.10.10.10".to_string(), NoPolicy::new())),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            expires_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            ..Default::default()
        }
            .insert(&db)
            .await
            .unwrap();

        let sessions = Query::find_related_application_sessions(&db, &application).await.unwrap();
        assert_eq!(sessions.len(), 1);

        let sessions = Query::find_related_admin_sessions(&db, &admin).await.unwrap();
        assert_eq!(sessions.len(), 1);
    }
}
