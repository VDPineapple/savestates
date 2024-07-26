use crate::tas::Tas;
use crate::worlds;
use crate::console;

use std::path::PathBuf;
use dirs::home_dir;
use std::fs::File;
use std::io::{self, Write, BufRead};
use serde_json;
use crossterm::style::Color;

// Gets dotfile path
pub fn get_dotfile_path() -> PathBuf {
    let home = home_dir().unwrap();
    home.join(".savestates")
}

// Creates a dotfile folder in the home directory
// if it does not already exist.
pub fn create_dotfile_ifndef() {
    let dotfile = get_dotfile_path();
    if !dotfile.exists() {
        std::fs::create_dir(&dotfile).unwrap();
        // Create TASes folder
        std::fs::create_dir(dotfile.join("tases")).unwrap();
        console::write_line(&Color::Green, true, "Created .savestates folder in home directory");
    }
}

// Adds all .minecraft paths to the dotfile
pub fn add_minecraft_folders(minecraft_folders: Vec<PathBuf>) {
    let dotfile = get_dotfile_path();
    if !dotfile.exists() {
        console::write_line(&Color::Yellow, true, "No dotfile found, creating new dotfile");
        create_dotfile_ifndef();
    }

    // Create .minecrafts file
    let minecrafts = dotfile.join(".minecrafts");
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(&minecrafts)
        .unwrap();

    // Read all .minecraft paths from the dotfile
    let mut existing_minecraft_folders = get_minecraft_folders();

    for folder in minecraft_folders {
        if existing_minecraft_folders.contains(&folder) {
            continue;
        }
        writeln!(file, "{}", folder.display()).unwrap();
    }
}

// Gets all .minecraft paths from the dotfile
pub fn get_minecraft_folders() -> Vec<PathBuf> {
    let dotfile = get_dotfile_path();
    let minecrafts_file = dotfile.join(".minecrafts");
    let mut paths = Vec::new();

    if let Ok(file) = File::open(minecrafts_file) {
        let reader = io::BufReader::new(file);
        for line in reader.lines() {
            if let Ok(path_str) = line {
                paths.push(PathBuf::from(path_str));
            }
        }
    }

    paths
}

// Gets all TAS files from the dotfile
pub fn get_tases() -> Vec<Tas> {
    let dotfile = get_dotfile_path();
    let tas_folder = dotfile.join("tases");
    let mut paths = Vec::new();

    if let Ok(entries) = std::fs::read_dir(tas_folder) {
        for entry in entries {
            if let Ok(entry) = entry {
                // For each TAS folder, find the TAS json file
                if entry.file_type().unwrap().is_dir() {
                    let tas_file = entry.path().join(entry.file_name()).with_extension("json");
                    paths.push(tas_file);
                }
            }
        }
    }

    let mut tases = Vec::new();
    for path in paths {
        let file = File::open(&path).unwrap();
        let tas: Tas = serde_json::from_reader(file).unwrap();
        tases.push(tas);
    }

    tases
}

// Creates a new TAS file in the dotfile
pub fn create_tas(minecraft_folder: PathBuf) -> Tas {
    let name = console::get_input("Enter a name for the new TAS file: ");
    let dotfile = get_dotfile_path();
    
    // Make the TAS a folder
    let tas_folder = dotfile.join("tases").join(&name);
    std::fs::create_dir(&tas_folder).unwrap();
    std::fs::create_dir(tas_folder.join("savestates")).unwrap();

    let mut tas = Tas::new(name, minecraft_folder.clone(), tas_folder.clone());

    let tas_file = tas_folder.join(&format!("{}.json", tas.name));
    let file = File::create(&tas_file).unwrap();
    serde_json::to_writer(file, &tas).unwrap();

    let world = worlds::choose_world(minecraft_folder);
    let nickname = console::get_input("Enter a nickname for the savestate: ");
    tas.create_savestate(world, nickname);


    tas
}

// Updates a TAS file in the dotfile
pub fn update_tas(tas: &Tas) {
    let dotfile = get_dotfile_path();
    let tas_folder = dotfile.join("tases").join(&tas.name);
    let tas_file = tas_folder.join(&format!("{}.json", tas.name));
    let file = File::create(&tas_file).unwrap();
    serde_json::to_writer(file, &tas).unwrap();
}