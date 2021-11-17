Done:
- [X] Move code to separate modules
- [X] Implement modified MessageIterator
- [X] Add user information to incoming_messages
- [X] Implementing chat commands
- [X] Remove newlines after messages
- [X] Move tests from TwitchChatMessage
- [X] Full command parsing tests

Tests:
- [ ] Test Ping-Pong mechanism, What do we do with pings when no events are received? They should still be answered!

New Feature:
- [P] Slap command | Split incoming messages by newlines
- [ ] Command options
- [ ] Viewerlist handler
- [ ] Create commands on the fly

Setup:
- [ ] Simplify and fully automate the process for getting the access token
- [ ] Add channel as option
- [ ] .env file
- [ ] docker setup

Something to think about:
- [ ] Refcell still needed?
- [ ] Enable sentences with spaces as single option for some commands

Bugs:
- [ ] Bot is not responding to multiple fast sent commands
