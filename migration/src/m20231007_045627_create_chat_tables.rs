use sea_orm_migration::{
    prelude::*,
    sea_orm::{EnumIter, Iterable},
    sea_query::extension::postgres::Type,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Conversation {
    Table,
    Id,
    Title,
}

#[derive(DeriveIden)]
enum Chat {
    Table,
    Id,
    Role,
    Content,
    ConversationId,
}

#[derive(Iden, EnumIter)]
pub enum ChatRole {
    Table,
    User,
    Assistant,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(ChatRole::Table)
                    .values(ChatRole::iter().skip(1))
                    .to_owned(),
            )
            .await?;
        
        manager
            .create_table(
                Table::create()
                    .table(Conversation::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Conversation::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Conversation::Title).string().not_null())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Chat::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Chat::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Chat::Content).string().not_null())
                    .col(
                        ColumnDef::new(Chat::Role)
                            .enumeration(ChatRole::Table, ChatRole::iter().skip(1)),
                    )
                    .col(ColumnDef::new(Chat::ConversationId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Chat::Table, Chat::ConversationId)
                            .to(Conversation::Table, Conversation::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Chat::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Conversation::Table).to_owned())
            .await?;
        Ok(())
    }
}
