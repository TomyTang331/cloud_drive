use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "file_permissions")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub id: i32,

    /// File ID
    pub file_id: i32,

    /// Authorized user ID
    pub user_id: i32,

    /// Read permission
    pub can_read: bool,

    /// Write permission
    pub can_write: bool,

    /// Delete permission
    pub can_delete: bool,

    /// Granter ID (usually admin)
    pub granted_by: i32,

    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::file::Entity",
        from = "Column::FileId",
        to = "super::file::Column::Id"
    )]
    File,

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,

    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::GrantedBy",
        to = "super::user::Column::Id"
    )]
    GrantedByUser,
}

impl Related<super::file::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::File.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
