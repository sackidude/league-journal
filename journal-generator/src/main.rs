use std::{io::Write, path::Path};
use std::fs;
use chrono::{self, Datelike};

fn main() {
    let now = chrono::offset::Local::now();
    let year_str = now.year().to_string();
    let month_str = now.month().to_string();

    let directory_path = Path::new(&year_str).join(&month_str);
    let dir_display = directory_path.display();

    // Create the necessary directory
    match fs::create_dir_all(&directory_path) {
        Err(why)=>println!("Failed to create directory {}: {}", dir_display, why),
        Ok(_)=>println!("Successfully created directory {}", dir_display),
    }

    let file_str = format!("{}.md", now.day().to_string());
    let file_path = directory_path.join(&file_str);
    let display = file_path.display();

    let mut file = match fs::File::create(&file_path) {
        Err(why) => panic!("Failed to create file {}: {}", display, why),
        Ok(file) => file,
    };

    let text = "test";
    match file.write_all(text.as_bytes()){
        Err(why) => panic!("Failed to write to file {}: {}", display, why),
        Ok(_)=>println!("Successfully wrote to {}", display),
    }
}
