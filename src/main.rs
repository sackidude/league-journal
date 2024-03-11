use chrono::Datelike;
use reqwest::header::{HeaderMap, ACCEPT, ACCEPT_LANGUAGE, CONNECTION, COOKIE, DNT, USER_AGENT};
use scraper::selectable::Selectable;
use serde::Serialize;
use serde_json::value::{Map, Value as Json};

use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

use handlebars::{to_json, Handlebars};

#[derive(Debug)]
enum Role {
    Top,
    Jungle,
    Mid,
    Bottom,
    Support,
}

const GAMES_IN_BLOCK: u8 = 3;
const ROLE: Role = Role::Jungle;

struct ParseError;

impl Role {
    fn value(&self) -> u8 {
        match self {
            Role::Top => 0,
            Role::Jungle => 1,
            Role::Mid => 2,
            Role::Bottom => 3,
            Role::Support => 4,
        }
    }

    fn from_str(input: &str) -> Result<Role, ParseError> {
        match input {
            "top" => Ok(Role::Top),
            "jungle" => Ok(Role::Jungle),
            "mid" => Ok(Role::Mid),
            "bottom" => Ok(Role::Bottom),
            "support" => Ok(Role::Support),
            _ => Err(ParseError {}),
        }
    }
}

#[derive(Serialize, Debug)]
struct Kda {
    kills: u8,
    deaths: u8,
    assists: u8,
}

#[derive(Serialize, Debug)]
struct Game {
    num: u8,
    ally_champ: String,
    enemy_champ: String,
    win: bool,
    kda: Kda,
    duration: String,
}

#[derive(Serialize, Debug)]
struct Session {
    date: String,
    games: Vec<Game>,
}

// This function should probably be slightly different.
fn get_date() -> String {
    let now = chrono::offset::Local::now();
    format!(
        "{}/{}-{}",
        now.day().to_string(),
        now.month().to_string(),
        now.year().to_string()
    )
}

async fn fetch_data() -> Map<String, Json> {
    let mut data = Map::new();

    data.insert("date".to_string(), to_json(get_date()));

    let mut headers = HeaderMap::new();

    headers.insert(
        ACCEPT,
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"
            .parse()
            .unwrap(),
    );
    // headers.insert(ACCEPT_ENCODING, "gzip, deflate, br");
    headers.insert(ACCEPT_LANGUAGE, "en-US,en;q=0.5".parse().unwrap());
    headers.insert(CONNECTION, "keep-alive".parse().unwrap());
    headers.insert(COOKIE, "lolg_euconsent=nitro".parse().unwrap());
    headers.insert(DNT, "1".parse().unwrap());
    headers.insert(
        USER_AGENT,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:123.0) Gecko/20100101 Firefox/123.0"
            .parse()
            .unwrap(),
    );

    let client = reqwest::Client::new();

    let res = client
        .get("https://www.leagueofgraphs.com/summoner/euw/Chaoborus-Spec")
        .headers(headers)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let document = scraper::Html::parse_document(&res);

    let tbody_selector = scraper::Selector::parse(
        "table.data_table.relative.recentGamesTable.inverted_rows_color>tbody",
    )
    .unwrap();

    let tr_selector = scraper::Selector::parse("tr").unwrap();

    let kills_selector = scraper::Selector::parse("span.kills").unwrap();
    let deaths_selector = scraper::Selector::parse("span.deaths").unwrap();
    let assists_selector = scraper::Selector::parse("span.assists").unwrap();
    let duration_selector = scraper::Selector::parse(".gameDuration").unwrap();
    let champ_img_selector = scraper::Selector::parse("img").unwrap();
    let victory_defeat_selector = scraper::Selector::parse(".victoryDefeatText").unwrap();
    let summoner_column_selector = scraper::Selector::parse(".summonerColumn").unwrap();

    let tbody = document.select(&tbody_selector).next().unwrap();

    let mut games: Vec<Game> = vec![];

    const USELESS_DIV_COUNT: u8 = 2;
    for (tr, tr_num) in tbody
        .select(&tr_selector)
        .zip(1..GAMES_IN_BLOCK + USELESS_DIV_COUNT + 1)
    {
        if tr_num < USELESS_DIV_COUNT {
            continue;
        }
        let num = GAMES_IN_BLOCK + USELESS_DIV_COUNT + 1 - tr_num; // element 0 and 1 are dubious
        let duration = match tr.select(&duration_selector).next() {
            Some(selected) => selected.inner_html().trim().to_string(),
            None => {
                continue;
            }
        };
        let kills = tr
            .select(&kills_selector)
            .next()
            .unwrap()
            .inner_html()
            .parse::<u8>()
            .unwrap();
        let deaths = tr
            .select(&deaths_selector)
            .next()
            .unwrap()
            .inner_html()
            .parse::<u8>()
            .unwrap();
        let assists = tr
            .select(&assists_selector)
            .next()
            .unwrap()
            .inner_html()
            .parse::<u8>()
            .unwrap();

        let kda = Kda {
            kills,
            deaths,
            assists,
        };

        let ally_champ = tr
            .select(&champ_img_selector)
            .next()
            .unwrap()
            .attr("alt")
            .unwrap()
            .to_string();

        let win = tr
            .select(&victory_defeat_selector)
            .next()
            .unwrap()
            .inner_html()
            == "Victory";

        let left_div = tr.select(&summoner_column_selector).next().unwrap();

        let mut enemy_champ = left_div
            .select(&champ_img_selector)
            .nth(ROLE.value().into())
            .unwrap()
            .attr("alt")
            .unwrap()
            .to_string();

        if enemy_champ == ally_champ {
            enemy_champ = tr
                .select(&summoner_column_selector)
                .nth(1)
                .unwrap()
                .select(&champ_img_selector)
                .nth(ROLE.value().into())
                .unwrap()
                .attr("alt")
                .unwrap()
                .to_string();
        }

        games.push(Game {
            num,
            ally_champ,
            enemy_champ,
            win,
            kda,
            duration,
        })
    }
    games.reverse();

    data.insert("games".to_string(), to_json(&games));
    data
}

#[tokio::main]
async fn main() {
    let mut handlebars = Handlebars::new();

    handlebars
        .register_template_file("template", "./templates/template.hbs")
        .unwrap();

    let now = chrono::offset::Local::now();
    let dir_path = Path::new(&now.year().to_string()).join(&now.month().to_string());

    std::fs::create_dir_all(&dir_path).unwrap();

    let path = dir_path.join(format!("{}.md", now.day().to_string()));

    if std::path::Path::exists(&path) {
        print!("File already exists, do you want to overwrite it [Y/n]: ");
        io::stdout().flush().unwrap();
        let mut buffer_str = String::new();
        match io::stdin().read_line(&mut buffer_str) {
            Ok(_) => {
                let choice = buffer_str.trim();
                if choice == "n" {
                    panic!("File not overwriting.")
                }
            }
            Err(why) => panic!("Failed to read from buffer: {why}"),
        }
    }

    let output_file = match File::create(&path) {
        Err(why) => panic!("Failed to create output file {}: {}", &path.display(), why),
        Ok(f) => f,
    };
    let data = fetch_data().await;
    let _ = handlebars.render_to_write("template", &data, &output_file);
    println!("Generated {}", path.display());
}
