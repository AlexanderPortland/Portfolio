use crate::Query;

use ::entity::admin::{self, Model, Entity as Admin};
use alohomora::{bbox::BBox, policy::NoPolicy};
use sea_orm::*;

impl Query {
    pub async fn find_admin_by_id(db: &DbConn, id: BBox<i32, NoPolicy>) -> Result<Option<admin::Model>, DbErr> {
        let r = Admin::find_by_id(id).one(db).await;
        r
    }

    pub async fn get_all_admin_public_keys(db: &DbConn) -> Result<Vec<BBox<String, NoPolicy>>, DbErr> {
        let admins = Admin::find().all(db).await?;

        // convert them all to models
        let admins: Vec<admin::Model> = admins.into_iter().map(Model::from).collect();

        let public_keys = admins
            .into_iter()
            .map(|admin| admin.public_key)
            .collect();

        Ok(public_keys)
    }

    pub async fn get_all_admin_public_keys_together(db: &DbConn) -> Result<Vec<BBox<String, NoPolicy>>, DbErr> {
        let admins = Admin::find().all(db).await?;

        // convert them all to models
        let admins: Vec<admin::Model> = admins.into_iter().map(Model::from).collect();

        let public_keys: Vec<_> = admins
            .into_iter()
            .map(|admin| admin.public_key)
            .collect();

        Ok(public_keys)
    }
}

#[cfg(test)]
mod tests {
    use alohomora::bbox::BBox;
    use alohomora::policy::NoPolicy;
    use entity::admin;
    use sea_orm::{ActiveModelTrait, Set};

    use crate::utils::db::get_memory_sqlite_connection;
    use crate::Query;

    #[tokio::test]
    async fn test_find_admin_by_id() {
        let db = get_memory_sqlite_connection().await;
        let admin = admin::ActiveModel {
            id: Set(BBox::new(1, NoPolicy::new())),
            name: Set(BBox::new("admin_1".to_string(), NoPolicy::new())),
            public_key: Set(BBox::new("valid_public_key_1".to_string(), NoPolicy::new())),
            private_key: Set(BBox::new("test".to_string(), NoPolicy::new())),
            password: Set(BBox::new("test".to_string(), NoPolicy::new())),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let admin = Query::find_admin_by_id(&db, admin.id).await.unwrap();
        assert!(admin.is_some());
    }

    #[tokio::test]
    async fn test_get_all_admin_public_keys() {
        let db = get_memory_sqlite_connection().await;
        for index in 1..5 {
            admin::ActiveModel {
                id: Set(BBox::new(index, NoPolicy::new())),
                name: Set(BBox::new(format!("admin_{}", index), NoPolicy::new())),
                public_key: Set(BBox::new(format!("valid_public_key_{}", index), NoPolicy::new())),
                private_key: Set(BBox::new("test".to_string(), NoPolicy::new())),
                password: Set(BBox::new("test".to_string(), NoPolicy::new())),
                created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
                updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
                ..Default::default()
            }
            .insert(&db)
            .await
            .unwrap();
        }

        let public_keys = Query::get_all_admin_public_keys(&db).await.unwrap();
        assert_eq!(public_keys.len(), 4);

        let public_keys: Vec<_> = public_keys.into_iter()
            .map(|r| r.discard_box())
            .collect();
        for index in 1..5 {
            assert!(public_keys.contains(&format!("valid_public_key_{}", index)));
        }
    }
}
