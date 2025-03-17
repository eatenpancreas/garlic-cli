use derived_deref::{Deref, DerefMut};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader, Read, Write},
    path::Path,
};

#[derive(Deref, DerefMut, Debug)]
pub struct DotEnvs(HashMap<String, String>);

impl DotEnvs {
    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let mut envs = Self(HashMap::new());

        for line in envs.get_file(path)?.lines() {
            if line.trim().is_empty() || line.trim().starts_with('#') {
                continue;
            }

            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();

                envs.insert(key, value);
            }
        }

        Ok(envs)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let lines: Vec<String> = self
            .get_file(path.as_ref())?
            .lines()
            .map(|line| line.to_owned())
            .collect();

        let lines = self.apply_to_lines(lines);

        // // Write content back to file
        let mut file = File::create(path)?;
        file.write_all(lines.join("\n").as_bytes())?;

        Ok(())
    }

    fn apply_to_lines(&self, mut lines: Vec<String>) -> Vec<String> {
        for (key, value) in &**self {
            let line = lines.iter_mut().find(|line| {
                if line.trim().is_empty() || line.trim().starts_with('#') {
                    return false;
                }

                if let Some(pos) = line.find('=') {
                    let key2 = line[..pos].trim();
                    if key2 == key {
                        return true;
                    }
                }

                return false;
            });

            if let Some(line) = line {
                *line = format!("{key}={value}")
            } else {
                lines.push(format!("{key}={value}"));
            }
        }

        lines
    }

    fn get_file(&self, path: impl AsRef<Path>) -> io::Result<String> {
        let file = File::open(path)?;
        let mut file_str = String::new();
        BufReader::new(file).read_to_string(&mut file_str)?;
        Ok(file_str)
    }
}
