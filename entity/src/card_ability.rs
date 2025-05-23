//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.8

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "card_ability")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub card: i32,
    pub level: i8,
    pub amount: String,
    pub title: String,
    pub description: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::card_design::Entity",
        from = "Column::Card",
        to = "super::card_design::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    CardDesign,
}

impl Related<super::card_design::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CardDesign.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
