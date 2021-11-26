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
- [X] Dynamic commands (on the fly)

Tests:
- [ ] Test Ping-Pong mechanism, What do we do with pings when no events are received? They should still be answered!
- [ ] tests for TwitchChatInternalEvent
- [ ] Test user management (with command access rights)

New Feature:
- [ ] Timed repetitions
- [ ] Respect chat message rate limits
- [ ] Spam protection (machine learning?)
- [ ] Polls
- [ ] Management UI
- [ ] Viewer Statistics
- [ ] Chat logs
- [ ] Discord command (!discord; and maybe also timed repetition)

Setup:
- [ ] Simplify and fully automate the process for getting the access token
- [ ] docker setup

Something to think about:
- [ ] Refcell still needed?
- [ ] Enable sentences with spaces as single option for some commands
- [ ] Can display names really be only upper case versions of the user name? If not, what else should be considered?

Bugs:
- [ ] Bot is not responding to multiple fast sent commands
- [ ] You can slap yourself!!!
- [ ] slapping is case sensitive
