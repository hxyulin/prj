use crate::ProjectStorage;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use skim::{
    prelude::{SkimItemReader, SkimOptionsBuilder},
    Skim,
};
use std::{io::Cursor, str::FromStr};

pub fn prompt(message: String, default: Option<String>) -> String {
    let theme = ColorfulTheme::default();
    let mut options = Input::with_theme(&theme).with_prompt(message);
    if let Some(default) = default {
        options = options.with_initial_text(default);
    }
    options.interact_text().expect("Failed to capture input")
}

pub fn prompt_empty(message: String) -> String {
    let theme = ColorfulTheme::default();
    Input::with_theme(&theme)
        .with_prompt(message)
        .allow_empty(true)
        .interact_text()
        .expect("Failed to capture input")
}

pub fn prompt_enum<T: FromStr>(
    message: String,
    options: &[&str],
    default: Option<String>,
) -> Option<T>
where
    T::Err: std::fmt::Debug,
{
    let theme = ColorfulTheme::default();
    let mut select = Select::with_theme(&theme).with_prompt(message);
    for option in options {
        select = select.item(option);
    }
    if let Some(default) = default {
        let idx = options.iter().position(|&x| x == default.as_str()).unwrap();
        select = select.default(idx);
    }
    select.interact_opt().expect("Failed to capture input").map(|index| FromStr::from_str(options[index]).unwrap())
}

pub fn choose_project_name(storage: &ProjectStorage, multi: bool) -> Option<Vec<String>> {
    let project_names = storage
        .projects
        .iter()
        .map(|project| project.name.clone())
        .collect::<Vec<_>>();

    let item_reader = SkimItemReader::default();
    let input = item_reader.of_bufread(Cursor::new(project_names.join("\n")));

    let options = SkimOptionsBuilder::default()
        .prompt(Some(if multi {
            "Select projects (press TAB to select multiple):"
        } else {
            "Select a project:"
        }))
        .height(Some("100%"))
        .bind(vec!["ctrl-a:select-all", "ctrl-u:clear-query"])
        .multi(multi)
        .build()
        .unwrap();

    let name: Option<Vec<_>> = Skim::run_with(&options, Some(input)).map(|out| {
        out.selected_items
            .iter()
            .map(|item| item.output().into_owned())
            .collect()
    });

    // Clear terminal state
    // 1. Show cursor
    // 2. Reset text attributes
    // 3. Clear screen
    // 4. Move cursor to top-left corner
    print!("\x1B[?25h\x1B[0m\x1B[2J\x1B[H");

    name
}
