use toasty::stmt::Id;

#[toasty::model]
pub struct User {
    #[key]
    pub username: String,

    pub password: String,

    pub is_verified: Option<String>,
    pub is_admin: Option<String>,

    #[index]
    pub invited_with_code: Option<String>,

    #[belongs_to(key = invited_with_code, references = invite_code)]
    pub invited_with: Option<Invite>,

    #[has_many]
    pub cards: [Card],

    #[has_many]
    pub auth_tokens: [AuthToken],
}

#[toasty::model]
pub struct AuthToken {
    #[key]
    token: String,

    #[index]
    username: String,

    #[belongs_to(key = username, references = username)]
    user: User,
}

#[toasty::model]
pub struct Invite {
    #[key]
    invite_code: String,

    #[index]
    inviter_username: String,

    #[belongs_to(key = inviter_username, references = username)]
    inviter: User,
}

#[toasty::model]
pub struct CardDesign {
    #[key]
    id: String,

    #[has_many]
    cards: [Card],
}

#[toasty::model]
pub struct Card {
    #[key]
    id: String,

    #[index]
    card_design_id: String,

    #[belongs_to(key = card_design_id, references = id)]
    card_design: CardDesign,

    #[index]
    holder_username: Option<String>,

    #[belongs_to(key = holder_username, references = username)]
    holder: Option<User>,
}

pub async fn init_db() -> toasty::Db {
    let db = toasty::Db::builder()
        .register::<User>()
        .register::<AuthToken>()
        .register::<Invite>()
        .register::<CardDesign>()
        .register::<Card>()
        .connect("sqlite:./frcc.db")
        .await
        .unwrap();

    db.reset_db().await.unwrap();

    db
}

//pub static Db: Lazy<toasty::Db> = Lazy::new(|| {
//});
