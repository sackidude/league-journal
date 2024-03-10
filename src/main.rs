use chrono::Datelike;
use reqwest::header::{HeaderMap, ACCEPT, ACCEPT_LANGUAGE, CONNECTION, COOKIE, DNT, USER_AGENT};
use serde::Serialize;
use serde_json::value::{Map, Value as Json};

use std::{fs::File, path::Path};

use handlebars::{to_json, Handlebars};

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

    std::fs::write("./league-of-graphs.html", &res).unwrap();

    let tbody_selector = scraper::Selector::parse(
        "table.data_table.relative.recentGamesTable.inverted_rows_color>tbody",
    )
    .unwrap();

    let tr_selector = scraper::Selector::parse("tr").unwrap();

    let kills_selector = scraper::Selector::parse("span.kills").unwrap();
    let deaths_selector = scraper::Selector::parse("span.deaths").unwrap();
    let assists_selector = scraper::Selector::parse("span.assists").unwrap();
    let duration_selector = scraper::Selector::parse(".gameDuration").unwrap();
    let ally_champ_selector = scraper::Selector::parse("img").unwrap();
    let victory_defeat_selector = scraper::Selector::parse(".victoryDefeatText").unwrap();

    let tbody = document.select(&tbody_selector).next().unwrap();

    let mut games: Vec<Game> = vec![];

    for (tr, tr_num) in tbody.select(&tr_selector).zip(1..6) {
        if tr_num < 2 {
            continue;
        }
        let num = tr_num - 2;
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
            .select(&ally_champ_selector)
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

        println!(
            "Game num: {}, duration: {:#?}, kda: {:#?}, ally_champ: {}",
            num, duration, kda, &ally_champ
        );

        games.push(Game {
            num,
            ally_champ,
            enemy_champ: "todo!".to_string(),
            win,
            kda,
            duration,
        })
    }

    data.insert("games".to_string(), to_json(&games));
    data
}

#[tokio::main]
async fn main() {
    let mut handlebars = Handlebars::new();

    let data = fetch_data().await;

    handlebars
        .register_template_file("template", "./templates/template.hbs")
        .unwrap();

    let now = chrono::offset::Local::now();
    let dir_path = Path::new(&now.year().to_string()).join(&now.month().to_string());

    std::fs::create_dir_all(&dir_path).unwrap();

    let path = dir_path.join(format!("{}.md", now.day().to_string()));

    let output_file = match File::create(&path) {
        Err(why) => panic!("Failed to create output file {}: {}", &path.display(), why),
        Ok(f) => f,
    };
    let _ = handlebars.render_to_write("template", &data, &output_file);
    println!("Generated {}", path.display());
}
