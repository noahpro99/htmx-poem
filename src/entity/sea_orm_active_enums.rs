//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.2

use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "chat_role")]
pub enum ChatRole {
    #[sea_orm(string_value = "assistant")]
    Assistant,
    #[sea_orm(string_value = "user")]
    User,
}
