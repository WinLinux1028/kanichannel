//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.10

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "post")]
pub struct Model {
    #[sea_orm(column_type = "Text")]
    pub board_id: String,
    pub thread_id: i64,
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: i64,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub mail: String,
    #[sea_orm(column_type = "Text")]
    pub poster_id: String,
    #[sea_orm(column_type = "Text")]
    pub body: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::thread::Entity",
        from = "(Column::ThreadId, Column::BoardId)",
        to = "(super::thread::Column::Id, super::thread::Column::BoardId)",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Thread,
}

impl Related<super::thread::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Thread.def()
    }
}

impl Related<super::board::Entity> for Entity {
    fn to() -> RelationDef {
        super::thread::Relation::Board.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::thread::Relation::Post.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}