use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SearchQueries::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SearchQueries::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(SearchQueries::Query).text().not_null())
                    .col(
                        ColumnDef::new(SearchQueries::Results)
                            .text()
                            .not_null()
                            .default("[]"),
                    )
                    .col(
                        ColumnDef::new(SearchQueries::CreatedAt)
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
            .drop_table(Table::drop().table(SearchQueries::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
enum SearchQueries {
    Table,
    Id,
    Query,
    Results,
    CreatedAt,
}
