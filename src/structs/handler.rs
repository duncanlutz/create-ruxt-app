use reqwest::blocking::Client;
use std::io::Write;
use std::{io::LineWriter, path::PathBuf};

use super::Environment;

pub struct Handler;

#[derive(Debug)]
pub enum FileHandler {
    File(File),
    Folder(Folder),
}

#[derive(Debug)]
pub struct File {
    name: String,
    content: String,
}

#[derive(Debug)]
pub struct Folder {
    name: String,
    children: Vec<FileHandler>,
}

impl Handler {
    pub fn create_ruxt_app(path: PathBuf, env: Environment) {
        let name = Handler::get_name(path.clone());
        let file_map = Handler::get_file_map(env, name.clone());

        if !Handler::validate_file_map(&file_map, path.clone()) {
            return;
        }

        Handler::create_files(path, file_map);
    }

    fn get_name(path: PathBuf) -> String {
        if path == PathBuf::from(".") {
            return std::env::current_dir()
                .expect("Failed to get current directory")
                .file_name()
                .expect("Failed to get file name")
                .to_str()
                .expect("Failed to convert file name to string")
                .to_string();
        } else {
            return path
                .file_name()
                .expect("Failed to get file name")
                .to_str()
                .expect("Failed to convert file name to string")
                .to_string();
        }
    }

    fn create_files(path: PathBuf, file_map: Vec<FileHandler>) {
        for handler in file_map {
            match handler {
                FileHandler::File(file) => {
                    let new_path = path.join(file.name);
                    let mut file_writer = LineWriter::new(
                        std::fs::File::create(new_path.clone()).expect("Failed to create file"),
                    );

                    file_writer
                        .write_all(file.content.as_bytes())
                        .expect("Failed to write to file");
                }
                FileHandler::Folder(folder) => {
                    let new_path = path.join(folder.name);
                    std::fs::create_dir_all(new_path.clone()).expect("Failed to create directory");
                    Handler::create_files(new_path, folder.children);
                }
            }
        }
    }

    fn validate_file_map(file_map: &Vec<FileHandler>, path: PathBuf) -> bool {
        for file in file_map {
            match file {
                FileHandler::File(file) => {
                    let path = path.join(file.name.clone());
                    if path.exists() {
                        eprintln!("File already exists: {}", file.name);
                        return false;
                    }
                }
                FileHandler::Folder(folder) => {
                    let path = path.join(folder.name.clone());
                    if path.exists() {
                        eprintln!("Folder already exists: {}", folder.name);
                        return false;
                    }

                    if !Handler::validate_file_map(&folder.children, path) {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn get_file_map(env: Environment, name: String) -> Vec<FileHandler> {
        let mut file_map: Vec<FileHandler> = Vec::new();
        match env {
            Environment::Development => {
                let file =
                    std::fs::read_to_string("map.json").expect("Failed to read development.json");

                let map: Vec<serde_json::Value> =
                    serde_json::from_str(&file).expect("Failed to parse development.json");

                Handler::recursively_create_files(&map, &mut file_map, name.clone());
            }
            Environment::Production => {
                let client = Client::new();
                let response = client
                    .get("https://raw.githubusercontent.com/duncanlutz/create-ruxt-app/main/map.json")
                    .send()
                    .expect("Failed to fetch map.json");

                let map: Vec<serde_json::Value> = response
                    .json()
                    .expect("Failed to parse map.json from remote");

                Handler::recursively_create_files(&map, &mut file_map, name.clone());
            }
        };

        file_map
    }

    fn recursively_create_files(
        map: &Vec<serde_json::Value>,
        file_map: &mut Vec<FileHandler>,
        name: String,
    ) {
        for item in map {
            if item["type"] == "folder" {
                let children = item["children"]
                    .as_array()
                    .expect("Failed to read children");

                let mut new_children = Vec::new();

                Handler::recursively_create_files(&children, &mut new_children, name.clone());

                let folder = Folder {
                    name: item["name"].to_string().replace("\"", ""),
                    children: new_children,
                };

                file_map.push(FileHandler::Folder(folder));
            } else if item["type"] == "file" {
                let content = item["content"].to_string().replace(r#"\n"#, "\n");
                let mut conent = content.chars();
                conent.next(); // Remove first escaped quote
                conent.next_back(); // Remove last escaped quote
                let mut content = conent.collect::<String>().replace(r#"\""#, "\"");

                if item["name"] == "Cargo.toml" {
                    content = content.replace("{name}", &name);
                }

                let file = File {
                    name: item["name"].to_string().replace("\"", ""),
                    content: content,
                };

                file_map.push(FileHandler::File(file));
            }
        }
    }
}
