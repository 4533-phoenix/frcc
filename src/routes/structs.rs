use crate::state::AppState;
use entity::prelude::*;
use sea_orm::EntityTrait;

#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardAbilityData {
    pub stat: u8,
    pub title: String,
    pub description: String,
}

#[derive(Deserialize)]
pub struct GetCardsParams {
    pub user: Option<String>,
    pub team: Option<i32>,
    pub year: Option<i16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserData {
    pub username: String,
    pub is_admin: bool,
    pub is_verified: bool,
    pub team: Option<String>,
}

#[derive(Deserialize)]
pub struct JoinTeamData {
    pub team_number: String,
    pub join_code: Option<String>,
}

#[derive(Deserialize)]
pub struct ChangePassword {
    pub current_password: String,
    pub new_password: String,
    pub confirm_password: String,
}

#[derive(Serialize, Deserialize)]
pub struct TeamData {
    pub number: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserTeamData {
    pub username: String,
    pub is_admin: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CardData {
    pub card_id: Option<String>,
    pub card_design_id: i32,
    pub team_number: i32,
    pub year: u16,
    pub team_name: String,
    pub card_name: Option<String>,
    pub card_note: Option<String>,
    pub abilities: Option<Vec<CardAbilityData>>,
}
impl CardData {
    pub async fn from_card(cardd: entity::card::Model, state: AppState, unlocked: bool) -> Self {
        let design = CardDesign::find_by_id(cardd.design)
            .one(&*state.db)
            .await
            .unwrap()
            .unwrap();
        Self {
            card_id: if unlocked { Some(cardd.id) } else { None },
            card_design_id: cardd.design,
            team_number: design.team,
            year: design.year as u16,
            team_name: state.get_team(design.team).await.name,
            card_name: if unlocked { Some(design.name) } else { None },
            card_note: if unlocked { Some(design.note) } else { None },
            abilities: if unlocked {
                Some(
                    state
                        .get_card_design_abilities(cardd.design)
                        .await
                        .into_iter()
                        .map(|a| CardAbilityData {
                            stat: a.level as u8,
                            title: a.title,
                            description: a.description,
                        })
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            },
        }
    }
}

#[derive(Deserialize)]
pub struct ErrorParams {
    pub error: Option<String>,
}
