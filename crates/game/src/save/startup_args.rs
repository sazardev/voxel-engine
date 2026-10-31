//! The world options on the command line, until the menus can ask the player (`ui/` is planned).

use thiserror::Error;

use super::select::WorldChoice;

/// What the player asked for when starting the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupRequest {
    pub choice: WorldChoice,
    /// Print the world list and quit instead of playing.
    pub list_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ArgsError {
    #[error("--world needs a name, for example: --world \"My world\"")]
    MissingWorldName,
    #[error("unknown option {0}; options are --world <name> and --list-worlds")]
    Unknown(String),
}

/// Reads the arguments after the program name. No arguments means: continue the last world.
///
/// # Errors
/// Returns [`ArgsError`] for an option this game does not have or `--world` without a name.
pub fn parse_startup_args(
    args: impl IntoIterator<Item = String>,
) -> Result<StartupRequest, ArgsError> {
    let mut request = StartupRequest {
        choice: WorldChoice::MostRecent,
        list_only: false,
    };
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--world" => {
                let name = args.next().ok_or(ArgsError::MissingWorldName)?;
                request.choice = WorldChoice::Named(name);
            }
            "--list-worlds" => request.list_only = true,
            _ => return Err(ArgsError::Unknown(arg)),
        }
    }
    Ok(request)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<StartupRequest, ArgsError> {
        parse_startup_args(args.iter().map(|arg| (*arg).to_owned()))
    }

    #[test]
    fn no_arguments_continue_the_last_world() {
        assert_eq!(
            parse(&[]),
            Ok(StartupRequest {
                choice: WorldChoice::MostRecent,
                list_only: false
            })
        );
    }

    #[test]
    fn world_names_the_world_to_play() {
        assert_eq!(
            parse(&["--world", "My world"]).map(|request| request.choice),
            Ok(WorldChoice::Named("My world".to_owned()))
        );
    }

    #[test]
    fn list_worlds_asks_for_the_list_only() {
        assert!(parse(&["--list-worlds"]).expect("parses").list_only);
    }

    #[test]
    fn world_without_a_name_is_an_error() {
        assert_eq!(parse(&["--world"]), Err(ArgsError::MissingWorldName));
    }

    #[test]
    fn an_unknown_option_is_named_in_the_error() {
        assert_eq!(
            parse(&["--fullscreen", "x"]),
            Err(ArgsError::Unknown("--fullscreen".to_owned()))
        );
    }
}
