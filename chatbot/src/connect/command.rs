use std::collections::HashMap;

#[derive(Debug)]
pub struct Command {
    pub name: String,
    pub options: Vec<String>,
    pub user_name: String,
    pub tags: HashMap<String, String>,
}

impl Command {
    pub fn new(text: &str, user_name: &str, tags: HashMap<String, String>) -> Option<Self> {
        if text.is_empty() {
            None
        } else {
            let mut words = text.split(' ');
            words.next().map(|name| Self {
                name: name.to_owned(),
                options: words.map(String::from).collect(),
                user_name: user_name.to_owned(),
                tags,
            })
        }
    }

    pub fn is_from_a_mod(&self) -> bool {
        // this is ugly because i have no time today,
        // we really should have a proper list of badges in our Command struct
        // then again it's maybe useless to always do that parsing as most commands will
        // not need to be protected
        if let Some(badges) = self.tags.get("badges") {
            badges.contains("broadcaster") || badges.contains("moderator")
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Command;
    use std::collections::HashMap;

    #[test]
    fn parsing_help_command_in_command_parser_without_options() {
        let raw_command = "help";
        let expected_command = "help";
        let expected_tags: HashMap<String, String> = HashMap::from([
            ("tag1".to_owned(), "something".to_owned()),
            ("tag2".to_owned(), "".to_owned()),
        ]);
        let parsed = Command::new(raw_command, "testuser", expected_tags).unwrap();
        assert_eq!(parsed.name, expected_command);
        assert_eq!(parsed.user_name, "testuser");
        assert_eq!(parsed.options, Vec::<String>::new());
    }

    #[test]
    fn parsing_command_in_command_parser_with_options() {
        let raw_command = "help option1 option2";
        let expected_command_type = "help";
        let expected_options = vec!["option1".to_owned(), "option2".to_owned()];
        let expected_tags: HashMap<String, String> = HashMap::from([
            ("tag1".to_owned(), "something".to_owned()),
            ("tag2".to_owned(), "".to_owned()),
        ]);
        let parsed = Command::new(raw_command, "testuser", expected_tags).unwrap();
        assert_eq!(parsed.name, expected_command_type);
        assert_eq!(parsed.user_name, "testuser");
        assert_eq!(parsed.options, expected_options);
    }

    #[test]
    fn parsing_slap_command() {
        let raw_command = "slap captaincallback";
        let expected_command = "slap";
        let expected_tags = HashMap::<String, String>::new();
        let parsed = Command::new(raw_command, "carkhy", expected_tags).unwrap();
        assert_eq!(parsed.name, expected_command);
        assert_eq!(parsed.user_name, "carkhy");
        assert_eq!(parsed.options, vec!["captaincallback"]);
    }

    #[test]
    fn parsing_empty_command() {
        let raw_command = "";
        println!(
            "{:#?}",
            Command::new(raw_command, "carkhy", HashMap::default())
        );
        assert!(matches!(
            Command::new(raw_command, "carkhy", HashMap::default()),
            None
        ));
    }

    #[test]
    fn ensure_is_from_a_mod_works() {
        let raw = "foo";
        let tags = HashMap::default();
        let parsed = Command::new(raw, "user", tags).unwrap();
        assert!(!parsed.is_from_a_mod());

        let mut tags = HashMap::default();
        tags.insert("badges".to_string(), "someotherbadge/2,broadcaster/1".to_string());
        let parsed = Command::new(raw, "user", tags).unwrap();
        assert!(parsed.is_from_a_mod());

        let mut tags = HashMap::default();
        tags.insert("badges".to_string(), "someotherbadge/2,moderator/1".to_string());
        let parsed = Command::new(raw, "user", tags).unwrap();
        assert!(parsed.is_from_a_mod());
    }
}
