use std::{
    fs::File,
    io::{self, Write},
    process::exit,
};

use handlebars::Handlebars;

mod fetcher;
mod game_structs;

#[tokio::main]
async fn main() {
    let username = "chaoborus".to_string();
    let tag = "spec".to_string();
    let region = "euw".to_string();
    let role = game_structs::Role::from_str(&"Jungle".to_lowercase())
        .expect("Failed to parse string as a role");
    let block_game_count: u8 = 3;
    let player = game_structs::Player {
        username,
        tag,
        region,
        role,
        block_game_count,
    };
    let data = fetcher::fetch_data(&player).await.unwrap();

    let mut handlebars = Handlebars::new();

    handlebars
        .register_template_file("template", "./templates/template.hbs")
        .expect("Couldn't find template file");

    let now = chrono::offset::Local::now();
    let dir_path = fetcher::date::get_directory_path(now);
    let path = dir_path.join(fetcher::date::get_file_name(now));

    std::fs::create_dir_all(&dir_path).expect("Failed to create directory");

    if std::path::Path::exists(&path) {
        print!("File already exists, do you want to overwrite it [Y/n]: ");
        io::stdout()
            .flush()
            .expect("Failed to flush standard output");
        let mut buffer_str = String::new();
        match io::stdin().read_line(&mut buffer_str) {
            Ok(_) => {
                let choice = buffer_str.trim();
                if choice == "n" {
                    println!("Quitting.");
                    exit(0);
                }
            }
            Err(why) => panic!("Failed to read from buffer: {why}"),
        }
    }

    let output_file = match File::create(&path) {
        Err(why) => panic!("Failed to create output file {}: {}", &path.display(), why),
        Ok(f) => f,
    };
    let _ = handlebars.render_to_write("template", &data, &output_file);
    println!("Generated {}", path.display());
}
