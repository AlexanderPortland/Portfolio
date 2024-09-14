use alohomora::bbox::BBox;
use chrono::Local;
use entity::admin;
use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveModelTrait, Set},
};
use portfolio_policies::{key::KeyPolicy, FakePolicy};

#[derive(DeriveMigrationName)]
pub struct Migration {
    admin: admin::ActiveModel,
}

impl Default for Migration {
    fn default() -> Self {
        Self {
            admin: admin::ActiveModel {
                id: Set(BBox::new(1, FakePolicy::new())),
                name: Set(BBox::new("Admin".to_owned(), FakePolicy::new())),
                public_key: Set(BBox::new("age1u889gp407hsz309wn09kxx9anl6uns30m27lfwnctfyq9tq4qpus8tzmq5".to_owned(), FakePolicy::new())),
                // AGE-SECRET-KEY-14QG24502DMUUQDT2SPMX2YXPSES0X8UD6NT0PCTDAT6RH8V5Q3GQGSRXPS
                // TODO: shouldn't be none for keypolicy
                private_key: Set(BBox::new("5KCEGk0ueWVGnu5Xo3rmpLoilcVZ2ZWmwIcdZEJ8rrBNW7jwzZU/XTcTXtk/xyy/zjF8s+YnuVpOklQvX3EC/Sn+ZwyPY3jokM2RNwnZZlnqdehOEV1SMm/Y".to_owned(), KeyPolicy::new(None, portfolio_policies::key::KeySource::JustGenerated))),
                // test
                password: Set(BBox::new("$argon2i$v=19$m=6000,t=3,p=10$WE9xCQmmWdBK82R4SEjoqA$TZSc6PuLd4aWK2x2WAb+Lm9sLySqjK3KLbNyqyQmzPQ".to_owned(), FakePolicy::new())),
                created_at: Set(BBox::new(Local::now().naive_local(), FakePolicy::new())),
                updated_at: Set(BBox::new(Local::now().naive_local(), FakePolicy::new())),
                ..Default::default()
            },
        }
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        self.admin.to_owned().insert(db).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        self.admin.to_owned().delete(db).await?;

        Ok(())
    }
}