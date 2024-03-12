use handlebars::to_json;
use reqwest::header::{HeaderMap, ACCEPT, ACCEPT_LANGUAGE, CONNECTION, COOKIE, DNT, USER_AGENT};
use scraper::selectable::Selectable;
use serde_json::value::{Map, Value as Json};

pub(crate) mod date;

pub async fn fetch_data(player: &crate::game_structs::Player) -> Result<Map<String, Json>, ()> {
    let mut data = Map::new();

    data.insert("date".to_string(), to_json(date::get_date()));

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
        .get(player.get_url_from_player())
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

    let mut games: Vec<crate::game_structs::Game> = vec![];

    const USELESS_DIV_COUNT: u8 = 2;
    for (tr, tr_num) in tbody
        .select(&tr_selector)
        .zip(1..player.block_game_count + USELESS_DIV_COUNT + 1)
    {
        if tr_num < USELESS_DIV_COUNT {
            continue;
        }
        let num = player.block_game_count + USELESS_DIV_COUNT + 1 - tr_num; // element 0 and 1 are dubious
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

        let kda = crate::game_structs::Kda {
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
            .nth(player.role.value().into())
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
                .nth(player.role.value().into())
                .unwrap()
                .attr("alt")
                .unwrap()
                .to_string();
        }

        games.push(crate::game_structs::Game {
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
    Ok(data)
}
