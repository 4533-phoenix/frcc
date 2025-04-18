use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum AuthToken {
    Table,
    User,
    Token,
}

#[derive(DeriveIden)]
enum Team {
    Table,
    Number,
    Name,
}

#[derive(DeriveIden)]
enum User {
    Table,
    Username,
    Password,
    IsAdmin,
    IsVerified,
}

#[derive(DeriveIden)]
enum UserTeam {
    Table,
    User,
    Team,
    Invite,
    IsAdmin,
}

#[derive(DeriveIden)]
enum CardDesign {
    Table,
    Id,
    Team,
    Name,
    Note,
    Year,
}

#[derive(DeriveIden)]
enum CardAbility {
    Table,
    Id,
    Card,
    Level,
    Amount,
    Title,
    Description,
}

#[derive(DeriveIden)]
enum Card {
    Table,
    Id,
    Design,
}

#[derive(DeriveIden)]
enum Scan {
    Table,
    Username,
    Card,
    ScanTime,
}

#[derive(DeriveIden)]
enum Invite {
    Table,
    Code,
    Inviter,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Team::Table)
                    .if_not_exists()
                    .col(unsigned(Team::Number).not_null().primary_key())
                    .col(string(Team::Name))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(AuthToken::Table)
                    .if_not_exists()
                    .col(string(AuthToken::Token).not_null().primary_key())
                    .col(string(AuthToken::User).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(AuthToken::Table, AuthToken::User)
                            .to(User::Table, User::Username)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(string(User::Username).not_null().primary_key())
                    .col(string(User::Password).not_null())
                    .col(boolean(User::IsAdmin).default(false))
                    .col(boolean(User::IsVerified).default(false))
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Invite::Table)
                    .if_not_exists()
                    .col(string(Invite::Code).primary_key())
                    .col(string(Invite::Inviter))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Invite::Table, Invite::Inviter)
                            .to(User::Table, User::Username)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UserTeam::Table)
                    .if_not_exists()
                    .col(string(UserTeam::User).primary_key())
                    .col(unsigned(UserTeam::Team))
                    .col(boolean(UserTeam::IsAdmin).default(false))
                    .col(string_null(UserTeam::Invite))
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserTeam::Table, UserTeam::User)
                            .to(User::Table, User::Username)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserTeam::Table, UserTeam::Team)
                            .to(Team::Table, Team::Number)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UserTeam::Table, UserTeam::Invite)
                            .to(Invite::Table, Invite::Code)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Scan::Table)
                    .if_not_exists()
                    .col(string(Scan::Card).not_null())
                    .col(string(Scan::Username).not_null())
                    .primary_key(Index::create().col(Scan::Card).col(Scan::Username))
                    .col(timestamp(Scan::ScanTime))
                    .foreign_key(
                        ForeignKey::create()
                            .from(Scan::Table, Scan::Username)
                            .to(User::Table, User::Username)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(Scan::Table, Scan::Card)
                            .to(Card::Table, Card::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(CardDesign::Table)
                    .if_not_exists()
                    .col(
                        unsigned(CardDesign::Id)
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(unsigned(CardDesign::Team).not_null())
                    .col(string(CardDesign::Name).not_null())
                    .col(string(CardDesign::Note))
                    .col(small_unsigned(CardDesign::Year))
                    .foreign_key(
                        ForeignKey::create()
                            .from(CardDesign::Table, CardDesign::Team)
                            .to(Team::Table, Team::Number)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(CardAbility::Table)
                    .if_not_exists()
                    .col(big_unsigned(CardAbility::Id).auto_increment().primary_key())
                    .col(unsigned(CardAbility::Card))
                    .col(tiny_unsigned(CardAbility::Level))
                    .col(string(CardAbility::Amount))
                    .col(string(CardAbility::Title))
                    .col(string(CardAbility::Description))
                    .foreign_key(
                        ForeignKey::create()
                            .from(CardAbility::Table, CardAbility::Card)
                            .to(CardDesign::Table, CardDesign::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Card::Table)
                    .if_not_exists()
                    .col(string(Card::Id).not_null().primary_key())
                    .col(unsigned(Card::Design).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Card::Table, Card::Design)
                            .to(CardDesign::Table, CardDesign::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Invite::Table)
                    .if_not_exists()
                    .col(string(Invite::Code).not_null().primary_key())
                    .col(string(Invite::Inviter).not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Invite::Table, Invite::Inviter)
                            .to(User::Table, User::Username)
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
            .drop_table(
                Table::drop()
                    .table(User::Table)
                    .table(Team::Table)
                    .table(UserTeam::Table)
                    .table(Scan::Table)
                    .to_owned(),
            )
            .await
    }
}
