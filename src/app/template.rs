use std::{fs::File, io::Read};

use chrono::{DateTime, Datelike, Utc};

use crate::app::error::MmemoResult;

pub struct Template;

impl Template {
    pub fn load(title: &str, mut file: File) -> MmemoResult<String> {
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;

        let date_time: DateTime<Utc> = Utc::now();
        let created_time = format!(
            "{}-{:02}-{:02}",
            date_time.year(),
            date_time.month(),
            date_time.day()
        );

        let content = buf
            .replace("{{title}}", title)
            .replace("{{date}}", &created_time);

        Ok(content)
    }
}
