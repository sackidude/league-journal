use std::path::{Path, PathBuf};

use chrono::{DateTime, Datelike, Local};

// This function should probably be slightly different.
pub fn get_date() -> String {
    let now = chrono::offset::Local::now();
    format!(
        "{}/{}-{}",
        now.day().to_string(),
        now.month().to_string(),
        now.year().to_string()
    )
}

pub fn get_directory_path(time: DateTime<Local>) -> PathBuf {
    Path::new(&time.year().to_string()).join(&time.month().to_string())
}

pub fn get_file_name(time: DateTime<Local>) -> String {
    format!("{}.md", time.day().to_string())
}
