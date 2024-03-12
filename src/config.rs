use serde_json;
use std::{
    fs::{self, File},
    io::{self, Write},
};

use crate::game_structs::Player;

pub fn fetch_config() -> Option<Player> {
    match fs::read_to_string("./config.json") {
        Ok(str) => serde_json::from_str(&str).ok(),
        Err(_) => return None,
    }
}

fn get_general(field: &str) -> String {
    print!("Enter your {field}: ");
    io::stdout()
        .flush()
        .expect("Failed to flush standard output");
    let mut res = String::new();
    io::stdin().read_line(&mut res).unwrap();
    res.trim().to_string()
}

pub fn get_manual_config() -> Player {
    println!("Couldn't find a config file. Follow this setup to set one up quickly");
    let region = get_general("region[euw/eune/na/kr]").to_string();
    let tag = get_general("tag").to_string();
    let username = get_general("username").to_string();
    let role = crate::game_structs::Role::from_str(
        &get_general("role[top/jungle/mid/bottom/support]").to_lowercase(),
    )
    .expect("Failed to parse string as a role");
    let block_game_count: u8 =
        get_general("number of games in one block(this is used to fetch that amount of games)")
            .parse()
            .expect("Failed to parse general as number");
    Player {
        username,
        tag,
        region,
        role,
        block_game_count,
    }
}

impl crate::game_structs::Player {
    pub fn save_config(&self) {
        let mut file = File::create("config.json").expect("Failed to create file");
        let _ = file.write_all(
            serde_json::to_string(&self)
                .expect("Failed to serialize player config")
                .as_bytes(),
        );
    }
}
