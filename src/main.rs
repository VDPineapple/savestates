use std::path::PathBuf;

use savestates::worlds;
use savestates::dotfile;
use savestates::console;
use savestates::tas::{self, Tas};

use crossterm::style::Color;

fn main() {
    dotfile::create_dotfile_ifndef();
    
    let mut tas: Tas = tas::choose_tas();
    // Store the latest savestate loaded, so that we can
    // delete it if the user loads before creating a new one
    let mut latest_loaded_savestate: Option<PathBuf> = None;
    
    loop {
        let choices = vec![
            "Create a new savestate".to_string(),
            "Load a savestate".to_string(),
            "Delete a savestate".to_string(),
            "Choose another TAS file".to_string(),
            "Exit".to_string(),
        ];
        let choice = console::present_choices("Choose an action".to_string(), choices);
        match choice {
            0 => {
                let world: PathBuf = worlds::choose_world(tas.minecraft_folder.clone());
                let nickname = console::get_input("Enter a nickname for the savestate: ");
                tas.create_savestate(world, nickname);

            }
            1 => {
                match tas.choose_savestate() {
                    Some(savestate) => {
                        let new_world = tas.load_savestate(&savestate);
                        if latest_loaded_savestate.is_some() {
                            let confirmation = console::confirm("Do you want to delete the previously loaded savestate?".to_string(), "y");
                            if confirmation {
                                console::write_line(&Color::Yellow, true, &format!("Deleting the previously loaded savestate world {}", latest_loaded_savestate.clone().unwrap().file_name().unwrap().to_string_lossy()));
                                worlds::delete_world(latest_loaded_savestate.unwrap());
                            } else {
                                console::write_line(&Color::Yellow, true, "Previous savestate not deleted");
                            }
                        }
                        latest_loaded_savestate = Some(new_world.clone());
                        console::write_line(&Color::Green, true, &format!("Savestate {} loaded successfully", savestate.file_name().unwrap().to_string_lossy()));
                    }
                    None => {}
                }
            }
            2 => {
                let mut savestates = tas.get_savestates();
                if savestates.is_empty() {
                    console::write_line(&Color::Red, true, "No savestates found");
                    continue;
                }

                let savestate_names = Tas::format_names(&savestates);
                let savestate_choice = console::present_choices("Choose a savestate to delete".to_string(), savestate_names);
                let savestate = &savestates[savestate_choice];
                let confirmation = console::confirm(format!("Are you sure you want to delete the savestate {}?", savestate.file_name().unwrap().to_string_lossy()), "delete");
                if confirmation {
                    tas.delete_savestate(savestate);
                } else {
                    console::write_line(&Color::Yellow, true, "Savestate deletion cancelled");
                }
            }
            3 => {
                tas = tas::choose_tas();
            }
            4 => {
                break;
            }
            _ => {
                console::write_line(&Color::Red, true, "Invalid choice");
            }
        }
    }
}
