use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Board::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Board::Id).text().not_null())
                    .col(ColumnDef::new(Board::Title).text().not_null())
                    .primary_key(Index::create().col(Board::Id))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Thread::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Thread::BoardId).text().not_null())
                    .col(ColumnDef::new(Thread::Id).big_integer().not_null())
                    .col(ColumnDef::new(Thread::Name).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Thread::Table, Thread::BoardId)
                            .to(Board::Table, Board::Id),
                    )
                    .primary_key(Index::create().col(Thread::Id).col(Thread::BoardId))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Post::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Post::BoardId).text().not_null())
                    .col(ColumnDef::new(Post::ThreadId).big_integer().not_null())
                    .col(ColumnDef::new(Post::Id).big_integer().not_null())
                    .col(ColumnDef::new(Post::Name).text().not_null())
                    .col(ColumnDef::new(Post::Mail).text().not_null())
                    .col(ColumnDef::new(Post::PosterId).text().not_null())
                    .col(ColumnDef::new(Post::Body).text().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Post::Table, (Post::ThreadId, Post::BoardId))
                            .to(Thread::Table, (Thread::Id, Thread::BoardId)),
                    )
                    .primary_key(Index::create().col(Post::Id))
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(Index::create().table(Post::Table).col(Post::Mail).take())
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Post::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Thread::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Board::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Board {
    Table,
    Id,
    Title,
}

#[derive(DeriveIden)]
pub enum Thread {
    Table,
    BoardId,
    Id,
    Name,
}

#[derive(DeriveIden)]
pub enum Post {
    Table,
    BoardId,
    ThreadId,
    Id,
    Name,
    Mail,
    PosterId,
    Body,
}
