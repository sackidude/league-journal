use serde::Serialize;
use serde_json::value::{Map, Value as Json};

use std::fs::File;

use handlebars::{
    to_json, Handlebars
};

#[derive(Serialize)]
pub struct Game {
    ally_champ: String,
    enemy_champ: String,
    scoreline: String,
}

#[derive(Serialize)]
pub struct Session {
    date: String,
    games: Vec<Game>,
}

pub fn fetch_data() -> Map<String, Json> {
    let mut data = Map::new();

    data.insert("date".to_string(), to_json("2024-02-23"));

    let games = vec![
        Game {
            ally_champ: "Graves".to_string(),
            enemy_champ: "Master Yi".to_string(),
            scoreline: "2/7/6".to_string(),
        },
        Game {
            ally_champ: "Graves".to_string(),
            enemy_champ: "Rammus".to_string(),
            scoreline: "10/3/5".to_string(),
        },
    ];

    data.insert("games".to_string(), to_json(&games));
    data
}

fn main() {
    let mut handlebars = Handlebars::new();

    let data = fetch_data();

    handlebars
        .register_template_file("template", "./templates/template.hbs")
        .unwrap();

    let output_file = match File::create("target/output.md") {
        Err(why)=>panic!("{}", why),
        Ok(f)=>f
    };
    let _ = handlebars.render_to_write("template", &data, &output_file);
    println!("Generated target/output.md");
}
