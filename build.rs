use std::fs::{read_dir, read_to_string, File};
use std::path::Path;

use std::io::{Error, Write};

// use std::env;

fn main() -> Result<(), Error> {
    let paths = read_dir("src/views").unwrap();
    let mut contents = "".to_string();

    for path in paths {
        let path_name = &path.as_ref().unwrap().path().display().to_string();
        if path_name.starts_with("_") {
            continue;
        }
        let content = read_to_string(path_name).expect(path_name);
        contents.push_str(&format!(
            "#[allow(dead_code)]pub static {}: &str = r#\"{}\"#;",
            path.unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .replace(".html", "")
                .to_ascii_uppercase(),
            content
                .lines()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect::<Vec<&str>>()
                .join("")
        ));
    }
    if Path::new("src/html.rs").exists() {
        if contents != read_to_string("src/html.rs")? {
            let mut output = File::create("src/html.rs")?;
            write!(output, "{}", contents)?;
        }
    } else {
        let mut output = File::create("src/html.rs")?;
        write!(output, "{}", contents)?;
    }

    // export .env
    // if Path::new(".env").exists() {
    //     let env_contents = read_to_string(".env")?;
    //     for line in env_contents.split("\n") {
    //         if line.starts_with("#") {
    //             continue;
    //         };
    //         let mut kv: Vec<&str> = line.splitn(2, "=").collect();
    //         if let Some(sp) = kv[1].strip_prefix("'") {
    //             kv[1] = sp;
    //         }
    //         if let Some(ss) = kv[1].strip_suffix("'") {
    //             kv[1] = ss;
    //         }
    //         println!("{} = {}", kv[0], kv[1]);
    //         env::set_var(kv[0], kv[1]);
    //     }
    // }
    Ok(())
}
