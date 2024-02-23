use serde::Serialize;
use serde_json::value::{Map, Value as Json};
use chrono::Datelike;

use std::fs::File;

use handlebars::{
    to_json, Handlebars
};

#[derive(Serialize)]
struct Game {
    num: u8,
    ally_champ: String,
    enemy_champ: String,
    scoreline: String,
}

#[derive(Serialize)]
struct Session {
    date: String,
    games: Vec<Game>,
}

// This function should probably be slightly different.
fn get_date() -> String {
    let now = chrono::offset::Local::now();
    return format!("{}/{}-{}", now.day().to_string(), now.month().to_string(), now.year().to_string())
}

fn fetch_data() -> Map<String, Json> {
    let mut data = Map::new();

    data.insert("date".to_string(), to_json(get_date()));

    let games = vec![
        Game {
            num: 1,
            ally_champ: "Graves".to_string(),
            enemy_champ: "Master Yi".to_string(),
            scoreline: "2/7/6".to_string(),
        },
        Game {
            num: 2,
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

    // TODO: Write the file in a appropriate folder/location.
    let output_file = match File::create("target/output.md") {
        Err(why)=>panic!("{}", why),
        Ok(f)=>f
    };
    let _ = handlebars.render_to_write("template", &data, &output_file);
    println!("Generated target/output.md");
}
