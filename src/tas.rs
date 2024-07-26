use crate::dotfile;
use crate::worlds;
use crate::console;
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use fs_extra::dir::{self, CopyOptions, get_size};
use crossterm::style::Color;
use chrono::offset::Utc;
use chrono::DateTime;

#[derive(Serialize, Deserialize, Clone)]
pub struct Tas {
    pub name: String,
    pub minecraft_folder: PathBuf,
    pub path: PathBuf,
    pub num_savestates: usize,
    pub attempts: HashMap<String, usize>,
}

impl Tas {
    pub fn new(name: String, minecraft_folder: PathBuf, path: PathBuf) -> Tas {
        Tas {
            name,
            minecraft_folder,
            path,
            num_savestates: 0,
            attempts: HashMap::new(),
        }
    }

    // Copy the world folder to the savestates folder
    pub fn create_savestate(&mut self, world: PathBuf, nickname: String) {
        let savestate_name = format!("{}-{}-{}", self.name, self.num_savestates, nickname);
        let savestate_folder = self.path.join("savestates").join(&savestate_name);
        std::fs::create_dir(&savestate_folder).unwrap();

        let mut options = CopyOptions::new(); 
        options.overwrite = true;
        dir::copy(world.clone(), &savestate_folder, &options).unwrap();

        // Now the world folder is inside the savestate folder
        // Move contents of world folder to savestate folder and delete world folder
        let world_folder = savestate_folder.join(world.file_name().unwrap());
        for entry in std::fs::read_dir(&world_folder).unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            let new_path = savestate_folder.join(entry.file_name());
            std::fs::rename(entry_path, new_path).unwrap();
        }

        std::fs::remove_dir_all(&world_folder).unwrap();

        self.num_savestates += 1;
        dotfile::update_tas(self);
    }

    // Get all savestates for this TAS
    pub fn get_savestates(&self) -> Vec<PathBuf> {
        let mut savestates = Vec::new();
        let savestates_folder = self.path.join("savestates");
        for entry in std::fs::read_dir(&savestates_folder).unwrap() {
            savestates.push(entry.unwrap().path());
        }
        // Sort by last modified date
        savestates.sort_by(|a, b| {
            let a_time = a.metadata().unwrap().modified().unwrap();
            let b_time = b.metadata().unwrap().modified().unwrap();
            b_time.cmp(&a_time)
        });

        savestates
    }

    // Load a savestate by copying the savestate folder to the .minecraft
    // saves folder. Return the new path to the savestate folder
    pub fn load_savestate(&mut self, savestate: &PathBuf) -> PathBuf {
        let saves_folder = self.minecraft_folder.join("saves");
        let mut options = CopyOptions::new();
        options.overwrite = true;
        // dir::copy(savestate, &saves_folder, &options).unwrap();
        // Copy the savestate folder to the saves folder with a new name
        let savestate_name = savestate.file_name().unwrap().to_string_lossy().to_string();
        if !self.attempts.contains_key(&savestate_name) {
            self.attempts.insert(savestate_name.clone(), 0);
        } else {
            let attempt = self.attempts.get(&savestate_name).unwrap();
            self.attempts.insert(savestate_name.clone(), attempt + 1);
        }
        dotfile::update_tas(self);

        let new_savestate_name = format!("{}-{}", savestate_name, self.attempts.get(&savestate_name).unwrap());
        let new_savestate = saves_folder.join(&new_savestate_name);
        std::fs::create_dir(&new_savestate).unwrap();
        dir::copy(savestate, &new_savestate, &options).unwrap();

        // Move the contents of the new save folder to the saves folder
        // and delete the new save folder
        let world_folder = new_savestate.join(savestate_name);
        for entry in std::fs::read_dir(&world_folder).unwrap() {
            let entry = entry.unwrap();
            let entry_path = entry.path();
            let new_path = new_savestate.join(entry.file_name());
            std::fs::rename(entry_path, new_path).unwrap();
        }

        std::fs::remove_dir_all(&world_folder).unwrap();

        new_savestate
    }

    // Delete a savestate
    pub fn delete_savestate(&self, savestate: &PathBuf) {
        // Ensure the folder is a savestate folder
        if !worlds::is_minecraft_save_folder(savestate) {
            console::write_line(&Color::Red, true, "Invalid savestate folder, deletion cancelled");
            return;
        }
        // Ensure the folder is < 5GB
        let size = get_size(savestate).unwrap();
        if size > 5_000_000_000 {
            console::write_line(&Color::Red, true, "Savestate is too large to delete automatically for safety reasons. Please delete manually.");
            return;
        }
        // Ensure the folder has 2 hyphens
        let savestate_name = savestate.file_name().unwrap().to_string_lossy();
        let hyphens: Vec<&str> = savestate_name.split("-").collect();
        if hyphens.len() != 3 {
            console::write_line(&Color::Red, true, "Unfamiliar savestate name format, deletion cancelled");
            return;
        }

        std::fs::remove_dir_all(savestate).unwrap();
    }

    pub fn format_names(savestates: &Vec<PathBuf>) -> Vec<String> {
        savestates.iter().map(
            |savestate| {
                let name = savestate.file_name().unwrap().to_string_lossy().to_string();
                let last_modified = savestate.metadata().unwrap().modified().unwrap();
                let datetime: DateTime<Utc> = last_modified.into();
                let formatted_date = format!("{}", datetime.format("%H:%M:%S %d/%m/%Y"));
                let limit = 35 - name.len();
                format!("{}{}{}", name, " ".repeat(limit), formatted_date)
            }
        ).collect()
    }

    // Choose a savestate to load
    pub fn choose_savestate(&self) -> Option<PathBuf> {
        let mut savestates = self.get_savestates();
        if savestates.is_empty() {
            console::write_line(&Color::Red, true, "No savestates found");
            return None;
        }
        let savestate_names = Tas::format_names(&savestates);
        let savestate_choice = console::present_choices("Choose a savestate to load".to_string(), savestate_names);
        Some(savestates[savestate_choice].clone())
    }
}

pub fn choose_tas() -> Tas {
    let tases = dotfile::get_tases();
    if tases.is_empty() {
        console::write_line(&Color::Yellow, true, "No TAS files found, creating new TAS file");
    }

    let mut tas_names: Vec<String> = tases.iter().map(|tas| tas.name.clone()).collect();
    tas_names.push("Create new TAS file".to_string());
    let tas_file_choice = console::present_choices("Choose a TAS file to load".to_string(), tas_names.clone());
    
    let tas: Tas;
    if tas_file_choice == tas_names.len() - 1 {
        let minecraft_folder = worlds::get_chosen_minecraft_folder();
        tas = dotfile::create_tas(minecraft_folder);
        console::write_line(&Color::Green, false, &format!("Created new TAS file: {}", tas.name));
    } else {
        tas = tases[tas_file_choice].clone();
        console::write_line(&Color::Green, false, &format!("Loaded TAS file: {}", tas.name));
    }


    tas
}