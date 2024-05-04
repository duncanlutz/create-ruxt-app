use reqwest::blocking::Client;
use std::path::PathBuf;

use super::Environment;

pub struct Handler;

pub enum FileHandler {
    File(File),
    Folder(Folder),
}

pub struct File {
    name: String,
    content: String,
}

pub struct Folder {
    name: String,
    children: Vec<FileHandler>,
}

impl Handler {
    pub fn create_ruxt_app(path: PathBuf, env: Environment) {
        let file_map = Handler::get_file_map(env);
        if !Handler::validate_file_map(&file_map) {
            return;
        }

        Handler::create_files(path, file_map);
    }

    fn create_files(path: PathBuf, file_map: Vec<FileHandler>) {
        for file in file_map {
            match file {
                FileHandler::File(file) => {
                    let path = path.join(file.name);
                    std::fs::write(path, file.content).expect("Failed to write file");
                }
                FileHandler::Folder(folder) => {
                    let path = path.join(folder.name);
                    std::fs::create_dir_all(path.clone()).expect("Failed to create directory");

                    Handler::create_files(path, folder.children);
                }
            }
        }
    }

    fn validate_file_map(file_map: &Vec<FileHandler>) -> bool {
        for file in file_map {
            match file {
                FileHandler::File(file) => {
                    let path = PathBuf::from(file.name.clone());
                    if path.exists() {
                        eprintln!("File already exists: {}", file.name);
                        return false;
                    }
                }
                FileHandler::Folder(folder) => {
                    let path = PathBuf::from(folder.name.clone());
                    if path.exists() {
                        eprintln!("Folder already exists: {}", folder.name);
                        return false;
                    }

                    if !Handler::validate_file_map(&folder.children) {
                        return false;
                    }
                }
            }
        }

        true
    }

    fn get_file_map(env: Environment) -> Vec<FileHandler> {
        let mut file_map: Vec<FileHandler> = Vec::new();
        match env {
            Environment::Development => {
                let file =
                    std::fs::read_to_string("map.json").expect("Failed to read development.json");

                let map: Vec<serde_json::Value> =
                    serde_json::from_str(&file).expect("Failed to parse development.json");

                Handler::recursively_create_files(&map, &mut file_map);
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

                Handler::recursively_create_files(&map, &mut file_map);
            }
        };

        file_map
    }

    fn recursively_create_files(map: &Vec<serde_json::Value>, file_map: &mut Vec<FileHandler>) {
        for item in map {
            if item["type"] == "folder" {
                let folder = Folder {
                    name: item["name"].to_string(),
                    children: Vec::new(),
                };

                file_map.push(FileHandler::Folder(folder));

                let children = item["children"]
                    .as_array()
                    .expect("Failed to read children");
                Handler::recursively_create_files(&children, file_map);
            } else if item["type"] == "file" {
                let file = File {
                    name: item["name"].to_string(),
                    content: item["content"].to_string(),
                };

                file_map.push(FileHandler::File(file));
            }
        }
    }
}
