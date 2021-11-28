use super::user_info::UserInfo;

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub options: Vec<String>,
    pub user: UserInfo,
}

impl Command {
    pub fn new(text: &str, user: UserInfo) -> Option<Self> {
        if text == "!" {
            return None;
        }
        let mut words = text.split(' ');
        words.next().map(|name| Self {
            name: name[1..].to_owned(),
            options: words.map(String::from).collect(),
            user,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn command_test_helper(
        raw_command: &str,
        expected_comand_name: &str,
        expected_command_options: Vec<String>,
    ) {
        let mock_user_info: UserInfo = UserInfo {
            name: String::from(""),
            badges: HashSet::default(),
        };
        let expected_command = Command {
            name: expected_comand_name.to_owned(),
            options: expected_command_options,
            user: UserInfo {
                name: "".to_owned(),
                badges: HashSet::default(),
            },
        };
        if let Some(command) = Command::new(raw_command, mock_user_info) {
            assert_eq!(command, expected_command);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn parsing_help_command() {
        command_test_helper("!help", "help", Vec::default());
    }

    #[test]
    fn parsing_slap_command() {
        command_test_helper(
            "!slap captaincallback",
            "slap",
            vec!["captaincallback".to_owned()],
        )
    }

    #[test]
    fn parsing_command_with_options() {
        command_test_helper(
            "!command option1 option2",
            "command",
            vec!["option1".to_owned(), "option2".to_owned()],
        );
    }

    #[test]
    fn parsing_empty_command() {
        let mock_user_info: UserInfo = UserInfo {
            name: String::from(""),
            badges: HashSet::default(),
        };
        assert!(Command::new("!", mock_user_info).is_none());
    }
}
