use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(Migration20260313000001InitialMigration),
        ]
    }
}

struct Migration20260313000001InitialMigration;

impl MigrationName for Migration20260313000001InitialMigration {
    fn name(&self) -> &str {
        "m20260313_000001_initial_migration"
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration20260313000001InitialMigration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 创建 users 表
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Username).string().not_null().unique_key())
                    .col(ColumnDef::new(Users::PasswordHash).text().not_null())
                    .col(ColumnDef::new(Users::Role).string().not_null())
                    .col(ColumnDef::new(Users::Disabled).boolean().not_null().default(false))
                    .col(
                        ColumnDef::new(Users::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Users::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 groups 表
        manager
            .create_table(
                Table::create()
                    .table(Groups::Table)
                    .col(
                        ColumnDef::new(Groups::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Groups::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Groups::Description).text())
                    .col(ColumnDef::new(Groups::CreatedBy).integer())
                    .col(
                        ColumnDef::new(Groups::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Groups::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 books 表
        manager
            .create_table(
                Table::create()
                    .table(Books::Table)
                    .col(
                        ColumnDef::new(Books::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Books::Title).string().not_null())
                    .col(ColumnDef::new(Books::Author).string())
                    .col(ColumnDef::new(Books::Description).text())
                    .col(ColumnDef::new(Books::FilePath).string().not_null())
                    .col(ColumnDef::new(Books::FileType).string().not_null())
                    .col(ColumnDef::new(Books::FileSize).big_integer().not_null())
                    .col(ColumnDef::new(Books::CoverPath).string())
                    .col(ColumnDef::new(Books::Isbn).string())
                    .col(ColumnDef::new(Books::Category).string())
                    .col(ColumnDef::new(Books::FileHash).string())
                    .col(ColumnDef::new(Books::CreatedBy).integer())
                    .col(ColumnDef::new(Books::Status).string().not_null().default("approved"))
                    .col(
                        ColumnDef::new(Books::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Books::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 user_groups 表
        manager
            .create_table(
                Table::create()
                    .table(UserGroups::Table)
                    .col(
                        ColumnDef::new(UserGroups::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(UserGroups::UserId).integer().not_null())
                    .col(ColumnDef::new(UserGroups::GroupId).integer().not_null())
                    .col(
                        ColumnDef::new(UserGroups::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_groups_user_id")
                            .from(UserGroups::Table, UserGroups::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_user_groups_group_id")
                            .from(UserGroups::Table, UserGroups::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 book_groups 表
        manager
            .create_table(
                Table::create()
                    .table(BookGroups::Table)
                    .col(
                        ColumnDef::new(BookGroups::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BookGroups::BookId).integer().not_null())
                    .col(ColumnDef::new(BookGroups::GroupId).integer().not_null())
                    .col(
                        ColumnDef::new(BookGroups::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_book_groups_book_id")
                            .from(BookGroups::Table, BookGroups::BookId)
                            .to(Books::Table, Books::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_book_groups_group_id")
                            .from(BookGroups::Table, BookGroups::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 settings 表
        manager
            .create_table(
                Table::create()
                    .table(Settings::Table)
                    .col(
                        ColumnDef::new(Settings::Key)
                            .string()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Settings::Value).string().not_null())
                    .col(
                        ColumnDef::new(Settings::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Settings::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 invite_codes 表
        manager
            .create_table(
                Table::create()
                    .table(InviteCodes::Table)
                    .col(
                        ColumnDef::new(InviteCodes::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(InviteCodes::Code).string().not_null().unique_key())
                    .col(ColumnDef::new(InviteCodes::GroupId).integer().not_null())
                    .col(ColumnDef::new(InviteCodes::CreatedBy).integer().not_null())
                    .col(ColumnDef::new(InviteCodes::MaxUsers).integer())
                    .col(ColumnDef::new(InviteCodes::UsedCount).integer().not_null().default(0))
                    .col(ColumnDef::new(InviteCodes::ExpiresAt).date_time())
                    .col(ColumnDef::new(InviteCodes::IsActive).boolean().not_null().default(true))
                    .col(
                        ColumnDef::new(InviteCodes::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(InviteCodes::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invite_codes_group_id")
                            .from(InviteCodes::Table, InviteCodes::GroupId)
                            .to(Groups::Table, Groups::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_invite_codes_created_by")
                            .from(InviteCodes::Table, InviteCodes::CreatedBy)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 book_uploaders 表
        manager
            .create_table(
                Table::create()
                    .table(BookUploaders::Table)
                    .col(
                        ColumnDef::new(BookUploaders::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(BookUploaders::BookId).integer().not_null())
                    .col(ColumnDef::new(BookUploaders::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(BookUploaders::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_book_uploaders_book_id")
                            .from(BookUploaders::Table, BookUploaders::BookId)
                            .to(Books::Table, Books::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_book_uploaders_user_id")
                            .from(BookUploaders::Table, BookUploaders::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 chapters 表
        manager
            .create_table(
                Table::create()
                    .table(Chapters::Table)
                    .col(
                        ColumnDef::new(Chapters::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Chapters::BookId).integer().not_null())
                    .col(ColumnDef::new(Chapters::ChapterNumber).integer().not_null())
                    .col(ColumnDef::new(Chapters::Title).string().not_null())
                    .col(ColumnDef::new(Chapters::Content).text().not_null())
                    .col(
                        ColumnDef::new(Chapters::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Chapters::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_chapters_book_id")
                            .from(Chapters::Table, Chapters::BookId)
                            .to(Books::Table, Books::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // 创建 comments 表
        manager
            .create_table(
                Table::create()
                    .table(Comments::Table)
                    .col(
                        ColumnDef::new(Comments::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Comments::BookId).integer())
                    .col(ColumnDef::new(Comments::ChapterId).integer())
                    .col(ColumnDef::new(Comments::UserId).integer().not_null())
                    .col(ColumnDef::new(Comments::Content).text().not_null())
                    .col(ColumnDef::new(Comments::Status).string().not_null().default("approved"))
                    .col(
                        ColumnDef::new(Comments::CreatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Comments::UpdatedAt)
                            .date_time()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comments_book_id")
                            .from(Comments::Table, Comments::BookId)
                            .to(Books::Table, Books::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comments_chapter_id")
                            .from(Comments::Table, Comments::ChapterId)
                            .to(Chapters::Table, Chapters::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_comments_user_id")
                            .from(Comments::Table, Comments::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 按依赖关系顺序删除表
        manager.drop_table(Table::drop().table(Comments::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Chapters::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(BookUploaders::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(InviteCodes::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(BookGroups::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(UserGroups::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Books::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Settings::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Groups::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Users::Table).to_owned()).await
    }
}

#[derive(Iden)]
enum Users {
    Table,
    Id,
    Username,
    PasswordHash,
    Role,
    CreatedAt,
    UpdatedAt,
    Disabled,
}

#[derive(Iden)]
enum Groups {
    Table,
    Id,
    Name,
    Description,
    CreatedAt,
    UpdatedAt,
    CreatedBy,
}

#[derive(Iden)]
enum Books {
    Table,
    Id,
    Title,
    Author,
    Description,
    FilePath,
    FileType,
    FileSize,
    CoverPath,
    CreatedAt,
    UpdatedAt,
    Isbn,
    Category,
    FileHash,
    CreatedBy,
    Status,
}

#[derive(Iden)]
enum UserGroups {
    Table,
    Id,
    UserId,
    GroupId,
    CreatedAt,
}

#[derive(Iden)]
enum BookGroups {
    Table,
    Id,
    BookId,
    GroupId,
    CreatedAt,
}

#[derive(Iden)]
enum Settings {
    Table,
    Key,
    Value,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum InviteCodes {
    Table,
    Id,
    Code,
    GroupId,
    CreatedBy,
    MaxUsers,
    UsedCount,
    ExpiresAt,
    IsActive,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum BookUploaders {
    Table,
    Id,
    BookId,
    UserId,
    CreatedAt,
}

#[derive(Iden)]
enum Chapters {
    Table,
    Id,
    BookId,
    ChapterNumber,
    Title,
    Content,
    CreatedAt,
    UpdatedAt,
}

#[derive(Iden)]
enum Comments {
    Table,
    Id,
    BookId,
    ChapterId,
    UserId,
    Content,
    Status,
    CreatedAt,
    UpdatedAt,
}