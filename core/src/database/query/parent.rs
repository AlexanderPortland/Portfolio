
use entity::candidate;
use entity::parent;
use entity::parent::Model;
use sea_orm::{EntityTrait, ModelTrait};
use sea_orm::{DbConn, DbErr};

use crate::Query;

impl Query {
    pub async fn find_candidate_parents(
        db: &DbConn,
        candidate: &candidate::Model,
    ) -> Result<Vec<Model>, DbErr> {

        candidate.find_related(parent::Entity)
            .all(db)
            .await
    }
    
    pub async fn list_all_parents(
        db: &DbConn,
    ) -> Result<Vec<Model>, DbErr> {
        parent::Entity::find()
            .all(db)
            .await
    }
}

#[cfg(test)]
mod tests {
    use alohomora::bbox::BBox;
    use alohomora::policy::NoPolicy;
    use entity::{candidate, parent};
    use sea_orm::{ActiveModelTrait, Set};

    use crate::Query;
    use crate::utils::db::get_memory_sqlite_connection;

    #[tokio::test]
    async fn test_find_parent_by_id() {
        let db = get_memory_sqlite_connection().await;

        const CANDIDATE_ID: i32 = 103158;

        candidate::ActiveModel {
            id: Set(BBox::new(CANDIDATE_ID, NoPolicy::new())),
            personal_identification_number: Set(BBox::new("test".to_string(), NoPolicy::new())),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();
        let parent = parent::ActiveModel {
            candidate_id: Set(BBox::new(CANDIDATE_ID, NoPolicy::new())),
            created_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            updated_at: Set(BBox::new(chrono::offset::Local::now().naive_local(), NoPolicy::new())),
            ..Default::default()
        }
        .insert(&db)
        .await
        .unwrap();

        let parent =  Query::find_candidate_by_id(&db, parent.candidate_id)
            .await
            .unwrap();
        assert!(parent.is_some());
    }
}
