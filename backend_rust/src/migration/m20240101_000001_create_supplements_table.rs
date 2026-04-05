use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Supplements::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Supplements::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Supplements::Name).text().not_null())
                    .col(
                        ColumnDef::new(Supplements::Brand)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(Supplements::Category)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(Supplements::Description)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(Supplements::Ingredients)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(
                        ColumnDef::new(Supplements::ServingSize)
                            .text()
                            .not_null()
                            .default(""),
                    )
                    .col(ColumnDef::new(Supplements::SpladeVector).text().null())
                    .col(
                        ColumnDef::new(Supplements::CreatedAt)
                            .text()
                            .not_null()
                            .default(Expr::cust("(strftime('%Y-%m-%dT%H:%M:%SZ','now'))")),
                    )
                    .col(
                        ColumnDef::new(Supplements::UpdatedAt)
                            .text()
                            .not_null()
                            .default(Expr::cust("(strftime('%Y-%m-%dT%H:%M:%SZ','now'))")),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Supplements::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum Supplements {
    Table,
    Id,
    Name,
    Brand,
    Category,
    Description,
    Ingredients,
    ServingSize,
    SpladeVector,
    CreatedAt,
    UpdatedAt,
}
