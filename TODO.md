Done:
- [X] Move code to separate modules
- [X] Implement modified MessageIterator
- [X] Add user information to incoming_messages
- [X] Implementing chat commands
- [X] Remove newlines after messages
- [X] Move tests from TwitchChatMessage

Tests:
- [ ] Test Ping-Pong mechanism, What do we do with pings when no events are received? They should still be answered!
- [ ] Full command parsing tests

New Feature:
- [P] Slap command | Split incoming messages by newlines
- [ ] Command options
- [ ] Create commands on the fly
- [ ] Viewerlist handler

Setup:
- [ ] Simplify and fully automate the process for getting the access token
- [ ] Add channel as option
- [ ] .env file
- [ ] docker setup

Miscellaneous:
- [ ] Refcell still needed?

Bugs:
- [ ] Bot is not responding to multiple fast sent commands
