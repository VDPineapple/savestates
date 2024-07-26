use std::path::PathBuf;

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor, Attribute, SetAttribute},
};


// Writes text to the console with the specified color and boldness
pub fn write(color: &Color, bold: bool, text: &str) {
    execute!(
        std::io::stdout(),
        if bold { SetAttribute(Attribute::Bold) } else { SetAttribute(Attribute::Reset) },
        SetForegroundColor(*color),
        Print(text),
        ResetColor
    ).unwrap();
}

// Writes text to the console with the specified color and boldness, followed by a newline
pub fn write_line(color: &Color, bold: bool, text: &str) {
    write(color, bold, text);
    execute!(std::io::stdout(), Print("\n")).unwrap();
}

// Gets user string input
pub fn get_input(prompt: &str) -> String {
    write(&Color::Magenta, true, prompt);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

// Gets user integer input from 1-N
pub fn get_int_input(prompt: &str, min: i32, max: i32) -> i32 {
    loop {
        write(&Color::Magenta, true, prompt);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim().parse::<i32>() {
            Ok(i) => {
                if i >= min && i <= max {
                    return i;
                } else {
                    write_line(&Color::Red, true, &format!("Please enter a number between {} and {}", min, max));
                }
            }
            Err(_) => {
                write_line(&Color::Red, true, "Invalid input.");
            }
        }
    }
}

pub trait Displayable {
    fn display_string(&self) -> String;
}

impl Displayable for String {
    fn display_string(&self) -> String {
        self.clone()
    }
}

impl Displayable for PathBuf {
    fn display_string(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}

// Takes a list of strings and waits for user input to select one
// Returns the index of the selected string
pub fn present_choices<T: Displayable>(prompt: String, choices: Vec<T>) -> usize {
    if choices.is_empty() {
        panic!("No choices provided to present_choices");
    } else if choices.len() == 1 {
        return 0;
    }


    write_line(&Color::Magenta, true, &prompt);
    for (i, choice) in choices.iter().enumerate() {
        // Print each choice in sky blue
        write_line(&Color::Cyan, false, &format!("{}. {}", i + 1, choice.display_string()));
    }
    
    get_int_input("Enter the number of your choice: ", 1, choices.len() as i32) as usize - 1
}

// Asks the user to confirm an action, returns true if the user confirms
// by typing a given string
pub fn confirm(prompt: String, confirm_string: &str) -> bool {
    write_line(&Color::Red, true, &prompt);
    let input = get_input(format!("Type the word '{}' to confirm: ", confirm_string).as_str());
    input.to_lowercase().trim() == confirm_string.to_lowercase()
}