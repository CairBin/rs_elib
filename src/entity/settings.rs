use axum::async_trait;
use chrono::Utc;
use sea_orm::{ActiveValue::Set, entity::prelude::*};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "settings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub key: String,
    pub value: String,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(mut self, db: &C, insert: bool) -> Result<Self, DbErr>
        where C: ConnectionTrait
    {
        let now = Utc::now().naive_utc();
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
