use std::path::Path;

#[derive(Clone)]
pub struct User {
    pub name: String,
    pub password: String,
}

pub fn read_from_file(file: &Path) -> std::io::Result<Vec<User>> {
    let text = std::fs::read_to_string(file)?;
    Ok(text
        .lines()
        .enumerate()
        .map(|(index, line)| (index, line.trim()))
        .filter(|&(_, line)| line != "")
        .filter_map(|(index, line)| {
            let mut parts = line.split_whitespace();
            let name = parts.next().map(str::trim);
            let password = parts.next().map(str::trim);
            let trailing = parts.next();
            match (name, password, trailing) {
                (Some(name), Some(password), None) => Some(User {
                    name: name.to_owned(),
                    password: password.to_owned(),
                }),
                _ => {
                    log::warn!("bad config line {}, skipping", index + 1);
                    None
                }
            }
        })
        .collect())
}
