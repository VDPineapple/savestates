use crate::dotfile;
use crate::console;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crossterm::style::Color;
use fs_extra::dir::get_size;
use chrono::offset::Utc;
use chrono::DateTime;

// Find all .minecraft folders on the system
// Return paths to .minecraft folders
pub fn search_for_minecraft_folders() -> Vec<PathBuf> {
    // Find the root directory of the system
    let root_directory;
    #[cfg(target_os = "linux")] {
        root_directory = "/".to_string();
    }
    #[cfg(target_os = "windows")]
    {
        root_directory = "C:\\".to_string();
    }

    let mut minecraft_folders = vec![];
    for entry in WalkDir::new(root_directory).into_iter().filter_entry(|e| e.file_type().is_dir()) {
        match entry {
            Ok(e) => {
                if e.file_name() == ".minecraft" {
                    minecraft_folders.push(e.path().to_path_buf());
                }
            }
            Err(_) => continue,
        }
    }

    if minecraft_folders.is_empty() {
        console::write_line(&Color::Red, true, "Failed to find any .minecraft folders");
    } else {
        console::write_line(&Color::Green, true, &format!("Found {} .minecraft folders", minecraft_folders.len()));
    }

    minecraft_folders
}

fn user_choose_minecraft_folder() -> PathBuf {
    while true {
        let prompt = "Please enter the path to your .minecraft folder: ";
        let path = console::get_input(prompt);
        // ensure the path is a directory called .minecraft
        if std::fs::metadata(&path).unwrap().is_dir() && path.ends_with(".minecraft") {
            return path.into();
        } else {
            console::write_line(&Color::Red, true, "Invalid path, please try again");
        }
    }

    unreachable!();
}

// Either get .minecraft folders from dotfile, search for it
// or ask the user for it. It can't be 
fn get_minecraft_folders() -> Vec<PathBuf> {
    // Check for existing search results in dotfile.
    let mut minecraft_folders = dotfile::get_minecraft_folders();
    if minecraft_folders.is_empty() {
        // If no results found, search for .minecraft folder.
        let choices = vec![
            "Search for .minecraft folders on system".to_string(),
            "Manually enter .minecraft folder path".to_string(),
        ];
        let choice = console::present_choices("No .minecraft folders found. Choose an option:".to_string(), choices);
        match choice {
            0 => {
                minecraft_folders = search_for_minecraft_folders();
                // Add search results to dotfile.
                dotfile::add_minecraft_folders(minecraft_folders.clone());
            }
            1 => {
                let path = user_choose_minecraft_folder();
                minecraft_folders.push(path.clone());
                dotfile::add_minecraft_folders(vec![path]);
            }
            _ => unreachable!(),
        }
    } else {
        console::write_line(&Color::Green, true, &format!("Found {} .minecraft folders in dotfile", minecraft_folders.len()));
        let choices = vec![
            "Use .minecraft folders found in dotfile".to_string(),
            "Manually enter .minecraft folder path".to_string(),
            "Perform another search for .minecraft folders on system".to_string(),
        ];
        let choice = console::present_choices("Choose an option:".to_string(), choices);
        match choice {
            0 => {
                console::write_line(&Color::Green, true, "Using .minecraft folders found in dotfile");
            }
            1 => {
                let path = user_choose_minecraft_folder();
                minecraft_folders.push(path.clone());
                dotfile::add_minecraft_folders(vec![path]);
            }
            2 => {
                minecraft_folders = search_for_minecraft_folders();
                dotfile::add_minecraft_folders(minecraft_folders.clone());
            }
            _ => unreachable!(),
        }
    }

    if minecraft_folders.is_empty() {
        let prompt = "No .minecraft folders found. Please enter the path to your .minecraft folder: ";
        let path = console::get_input(prompt);
        minecraft_folders.push(std::path::PathBuf::from(path));
        dotfile::add_minecraft_folders(minecraft_folders.clone());
    }

    minecraft_folders
}

// Get chosen TAS .minecraft folder
pub fn get_chosen_minecraft_folder() -> std::path::PathBuf {
    let minecraft_folders = get_minecraft_folders();
    let mut choices: Vec<PathBuf> = vec![];
    for folder in minecraft_folders.clone() {
        choices.push(folder.clone());
    }

    let prompt = "Please select the .minecraft folder you would like to use: ";
    let choice = console::present_choices(prompt.to_string(), choices);
    minecraft_folders[choice].clone()
}

// Perform a basic check to ensure a folder
// is a minecraft save folder.
pub fn is_minecraft_save_folder<T>(path: T) -> bool
where
    T: AsRef<Path>,
{
    let path = path.as_ref();
    let mut has_dat = false;
    let mut has_level_dat = false;

    for entry in WalkDir::new(path)
            .min_depth(1).max_depth(1).into_iter()
            .filter_entry(|e| e.file_type().is_file()) {
        match entry {
            Ok(e) => {
                if e.file_name() == "level.dat" {
                    has_level_dat = true;
                } else if e.file_name() == "session.lock" {
                    has_dat = true;
                }
            }
            Err(_) => continue,
        }
    }

    has_dat && has_level_dat
}

// Search .minecraft folder for all world folders
// Return a list of all paths to world folders
pub fn get_all_worlds<T>(path: T) -> Vec<PathBuf>
where
    T: AsRef<Path>,
{
    let saves_folder = path.as_ref().join("saves");
    let mut world_folders = vec![];

    for entry in WalkDir::new(saves_folder)
            .min_depth(1).max_depth(1).into_iter()
            .filter_entry(|e| e.file_type().is_dir()) {
        match entry {
            Ok(e) => {
                if e.file_type().is_dir() && is_minecraft_save_folder(e.path()) {
                    world_folders.push(e.path().to_path_buf());
                }
            }
            Err(_) => continue,
        }
    }

    if world_folders.is_empty() {
        console::write_line(&Color::Red, true, "Failed to find any world folders in .minecraft");
    } else {
        console::write_line(&Color::Yellow, true, &format!("Found {} world folders in .minecraft", world_folders.len()));
    }

    // Sort by last modified date
    world_folders.sort_by(|a, b| {
        let a_time = a.metadata().unwrap().modified().unwrap();
        let b_time = b.metadata().unwrap().modified().unwrap();
        b_time.cmp(&a_time)
    });

    world_folders
}

// Ask the user to choose a world folder, sorted by
// last modified date.
pub fn choose_world(minecraft_folder: PathBuf) -> PathBuf {
    let world_folders = get_all_worlds(minecraft_folder);

    let mut choices: Vec<String> = vec![];
    for (i, folder) in world_folders.iter().enumerate() {
        let name: String = folder.file_name().unwrap().to_string_lossy().into();
        // Clamp name to 25 characters
        let name = if name.len() > 25 { format!("{}...", &name[..25]) } else { name };
        let last_modified = folder.metadata().unwrap().modified().unwrap();
        // Format the last modified date to HH:MM:SS DD/MM/YYYY
        let datetime: DateTime<Utc> = last_modified.into();
        let formatted_date = format!("{}", datetime.format("%H:%M:%S %d/%m/%Y")).to_string();
        // Space until 42 characters, then add the formatted date
        // Position number occupies log10(i) + 1 characters
        let pos_length = ((i+1) as f64).log10().floor() as usize + 1;
        let limit = 38 - pos_length - name.len();
        let choice = format!("{}{}{}", name, " ".repeat(limit), formatted_date).to_string();
        choices.push(choice);
    }

    let prompt = "Please select the world you would like to use: ";
    let choice = console::present_choices(prompt.to_string(), choices);
    world_folders[choice].clone()
}

// Delete a world folder
pub fn delete_world<T>(world_folder: T)
where
    T: AsRef<Path>,
{
    let world_folder = world_folder.as_ref();
    if !is_minecraft_save_folder(world_folder) {
        console::write_line(&Color::Red, true, "Invalid world folder, deletion cancelled");
        return;
    }

    let size = get_size(world_folder).unwrap();
    if size > 5_000_000_000 {
        console::write_line(&Color::Red, true, "World is too large to delete automatically for safety reasons. Please delete manually.");
        return;
    }

    std::fs::remove_dir_all(world_folder).unwrap();
}