use std::fs::{
    File,
    read_to_string
};
use std::fs;

use std::io::{Write, Error};

fn main() -> Result<(), Error> {
    let paths = fs::read_dir("src/views").unwrap();
    let mut contents = "".to_owned();

    for path in paths {
        let path_name = &path.as_ref().unwrap().path().display().to_string();
        let content = read_to_string(path_name)
            .expect(path_name);
        contents.push_str(&format!("pub static {}: &str = r#\"{}\"#;", path.unwrap().file_name().to_str().unwrap().replace(".html", "").to_ascii_uppercase(), content.lines().map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<&str>>().join("")));
    }
    if contents != read_to_string("src/html.rs")? {
        let mut output = File::create("src/html.rs")?;
        write!(output, "{}", contents)?;
    }
    Ok(())
}