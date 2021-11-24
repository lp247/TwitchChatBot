use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum CommandType {
    Help,
    Info,
    Slap,
}

#[derive(Debug)]
pub struct Command {
    pub commmand_type: CommandType,
    pub options: Vec<String>,
    pub user_name: String,
    pub tags: HashMap<String, String>,
}

impl Command {
    pub fn new(text: &str, user_name: &str, tags: HashMap<String, String>) -> Option<Self> {
        if !text.starts_with('!') {
            None
        } else {
            let mut words = text.split(' ');
            match &words.next()?[1..] {
                "help" => Some(Self {
                    commmand_type: CommandType::Help,
                    options: words.map(String::from).collect(),
                    user_name: user_name.to_owned(),
                    tags,
                }),
                "info" => Some(Self {
                    commmand_type: CommandType::Info,
                    options: words.map(String::from).collect(),
                    user_name: user_name.to_owned(),
                    tags,
                }),
                "slap" => Some(Self {
                    commmand_type: CommandType::Slap,
                    options: words.map(String::from).collect(),
                    user_name: user_name.to_owned(),
                    tags,
                }),
                _ => None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Command, CommandType};
    use std::collections::HashMap;

    #[test]
    fn parsing_help_command_in_command_parser_without_options() {
        let raw_command = "!help";
        let expected_command_type = CommandType::Help;
        let expected_tags: HashMap<String, String> = HashMap::from([
            ("tag1".to_owned(), "something".to_owned()),
            ("tag2".to_owned(), "".to_owned()),
        ]);
        let parsed = Command::new(raw_command, "testuser", expected_tags);
        assert!(parsed.is_some());
        let unwrapped_parsed = parsed.unwrap();
        assert_eq!(unwrapped_parsed.commmand_type, expected_command_type);
        assert_eq!(unwrapped_parsed.user_name, "testuser");
        assert_eq!(unwrapped_parsed.options, Vec::<String>::new());
    }

    #[test]
    fn parsing_command_in_command_parser_with_options() {
        let raw_command = "!help option1 option2";
        let expected_command_type = CommandType::Help;
        let expected_options = vec!["option1".to_owned(), "option2".to_owned()];
        let expected_tags: HashMap<String, String> = HashMap::from([
            ("tag1".to_owned(), "something".to_owned()),
            ("tag2".to_owned(), "".to_owned()),
        ]);
        let parsed = Command::new(raw_command, "testuser", expected_tags);
        assert!(parsed.is_some());
        let unwrapped_parsed = parsed.unwrap();
        assert_eq!(unwrapped_parsed.commmand_type, expected_command_type);
        assert_eq!(unwrapped_parsed.user_name, "testuser");
        assert_eq!(unwrapped_parsed.options, expected_options);
    }

    #[test]
    fn parsing_slap_command() {
        let raw_command = "!slap captaincallback";
        let expected_command_type = CommandType::Slap;
        let expected_tags = HashMap::<String, String>::new();
        let parsed = Command::new(raw_command, "carkhy", expected_tags);
        assert!(parsed.is_some());
        let unwrapped_parsed = parsed.unwrap();
        assert_eq!(unwrapped_parsed.commmand_type, expected_command_type);
        assert_eq!(unwrapped_parsed.user_name, "carkhy");
        assert_eq!(unwrapped_parsed.options, vec!["captaincallback"]);
    }
}
