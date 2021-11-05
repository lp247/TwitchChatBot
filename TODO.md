- [X] Move code to separate modules
- [ ] Test Ping-Pong mechanism
(i would make an in memory structure, that receives "events" and returns commands, you can implement the full logic in there, and let the comands do the IO, and provide the events)
- [X] Implement modified MessageIterator
- [X] Add user information to incoming_messages
- [ ] Implementing chat commands

- [ ] Simplify and fully automate the process for getting the access token
- [ ] Command options
- [ ] Create commands on the fly
- [ ] Remove newlines after messages




fn parse_user_message(user_message: String, user_name: String) -> UserMessageType {
    enum State { Starting, ParsingCommand }
    use State::*;

    let mut state = Starting;
    let mut command = String::with_capacity(raw_message.len());

    for (i, codepoint) in raw_message.char_indices() {
        match state {
            Starting => if codepoint == '!' { state = ParsingCommand } else {
                return UserMessageType::BasicMessage(MessageInfo{user: user_name, text: user_message});
            },
            ParsingCommand => match codepoint { // PRIVMSG #captaincallback :backseating backseating
                ' ' => match command {
                    "help" => UserMessageType::Command({user: user_name, command: Command::Help}),
                    _ => UserMessageType::BasicMessage({user: user_name, text: user_message}),
                }
                _ => command.push(codepoint),
            }
        }
    }
}







