use serde::Serialize;

#[derive(Debug)]
pub enum Role {
    Top,
    Jungle,
    Mid,
    Bottom,
    Support,
}

impl Role {
    pub fn value(&self) -> u8 {
        match self {
            Role::Top => 0,
            Role::Jungle => 1,
            Role::Mid => 2,
            Role::Bottom => 3,
            Role::Support => 4,
        }
    }

    // `input` is lowercase string slice representing a role.
    // Returns None if unable to parse into Role.
    pub fn from_str(input: &str) -> Option<Role> {
        match input {
            "top" => Some(Role::Top),
            "jungle" => Some(Role::Jungle),
            "mid" => Some(Role::Mid),
            "bottom" => Some(Role::Bottom),
            "support" => Some(Role::Support),
            _ => None,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Kda {
    pub kills: u8,
    pub deaths: u8,
    pub assists: u8,
}

#[derive(Serialize, Debug)]
pub struct Game {
    pub num: u8,
    pub ally_champ: String,
    pub enemy_champ: String,
    pub win: bool,
    pub kda: Kda,
    pub duration: String,
}

#[derive(Serialize, Debug)]
pub struct Session {
    pub date: String,
    pub games: Vec<Game>,
}

pub struct Player {
    pub username: String,
    pub tag: String,
    pub region: String,
    pub role: Role,
    pub block_game_count: u8,
}

impl Player {
    // https://www.leagueofgraphs.com/summoner/euw/Chaoborus-Spec
    pub fn get_url_from_player(&self) -> String {
        format!(
            "https://www.leagueofgraphs.com/summoner/{}/{}-{}",
            self.region, self.username, self.tag
        )
    }
}
