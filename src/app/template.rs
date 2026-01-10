use std::io::Read;

use crate::app::error::MmemoResult;

pub fn load_template(title: &str, mut file: impl Read) -> MmemoResult<String> {
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    let created_date = chrono::Utc::now().date_naive(); // "YYYY-MM-DD"
    let content = buf
        .replace("{{title}}", title)
        .replace("{{date}}", &created_date.to_string())
        .replace("{{tags}}", "tags: []");

    Ok(content)
}
