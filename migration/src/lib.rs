pub use sea_orm_migration::prelude::*;

mod m20231007_045627_create_chat_tables;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20231007_045627_create_chat_tables::Migration),
        ]
    }
}
