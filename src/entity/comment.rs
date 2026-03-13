use axum::async_trait;
use sea_orm::{ActiveValue::Set, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "comments")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub book_id: Option<i32>,
    pub chapter_id: Option<i32>,
    pub user_id: i32,
    pub content: String,
    pub status: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::book::Entity",
        from = "Column::BookId",
        to = "super::book::Column::Id"
    )]
    Book,
    #[sea_orm(
        belongs_to = "super::chapter::Entity",
        from = "Column::ChapterId",
        to = "super::chapter::Column::Id"
    )]
    Chapter,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, db: &C, insert: bool) -> Result<Self, DbErr>
        where C: ConnectionTrait
    {
        let now = chrono::Utc::now().naive_utc();
        if insert {
            // 插入时设置
            if self.created_at.is_set() == false {
                self.created_at = Set(now);
            }
        }

        // 无论插入还是更新，都更新 updated_at
        self.updated_at = Set(now);

        Ok(self)
    }
}
