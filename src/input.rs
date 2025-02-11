use dialoguer::theme::ColorfulTheme;

pub trait Fail {
    type T;
    fn unwrap_or_fail(self) -> Self::T;
}

impl<T, R> Fail for Result<T, R>
where
    R: std::fmt::Display,
{
    type T = T;
    fn unwrap_or_fail(self) -> Self::T {
        match self {
            Ok(v) => v,
            Err(e) => {
                eprintln!("{}", e);
                std::process::exit(1);
            }
        }
    }
}

pub fn get_cli_choice(prompt: &str, items: Vec<String>) -> usize {
    let choice = dialoguer::Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&items)
        .interact_opt()
        .unwrap();

    choice.ok_or("No choice selected").unwrap_or_fail()
}
