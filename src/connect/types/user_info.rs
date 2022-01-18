use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct Badge {
    pub name: String,
    pub level: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub struct UserInfo {
    pub name: String,
    pub badges: HashSet<Badge>,
}

impl UserInfo {
    pub fn has_elevated_rights(&self) -> bool {
        self.badges
            .iter()
            .any(|badge| badge.name == "broadcaster" || badge.name == "moderator")
    }
}
