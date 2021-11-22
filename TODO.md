Done:
- [X] Move code to separate modules
- [X] Implement modified MessageIterator
- [X] Add user information to incoming_messages
- [X] Implementing chat commands
- [X] Remove newlines after messages
- [X] Move tests from TwitchChatMessage
- [X] Full command parsing tests
- [X] Slap command | Split incoming messages by newlines
- [X] Command options
- [X] Viewerlist handler
- [X] .env file
- [X] Add channel as option

Tests:
- [ ] Test Ping-Pong mechanism, What do we do with pings when no events are received? They should still be answered!

New Feature:
- [ ] User management (with command access rights)
- [ ] Dynamic commands (on the fly)
- [ ] Timed repetitions
- [ ] Respect chat message rate limits
- [ ] Spam protection (machine learning?)
- [ ] Polls
- [ ] Management UI
- [ ] Viewer Statistics
- [ ] Chat logs

Setup:
- [ ] Simplify and fully automate the process for getting the access token
- [ ] docker setup

Something to think about:
- [ ] Refcell still needed?
- [ ] Enable sentences with spaces as single option for some commands

Bugs:
- [ ] Bot is not responding to multiple fast sent commands
- [ ] You can slap yourself!!!
