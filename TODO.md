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
- [X] docker setup
- [X] tests for ReceiveEvent
- [X] Test user management (with command access rights)
- [X] Discord command (!discord; and maybe also timed repetition)

Tests:
- [ ] What do we do with pings when no events are received? They should still be answered!

New Feature:
- [ ] Timed repetitions
- [ ] Respect chat message rate limits
- [ ] Spam protection (machine learning?)
- [ ] Polls
- [ ] Management UI
- [ ] Viewer Statistics
- [ ] Chat logs

Setup:
- [ ] Simplify and fully automate the process for getting the access token

Something to think about:
- [ ] Can display names really be only upper case versions of the user name? If not, what else should be considered?
- [ ] Maybe put all the looping logic of the main.rs file into a separate event_loop.rs file
- [ ] Proper error handling in all the program

Bugs:
- [ ] Bot is not responding to multiple fast sent commands
- [ ] You can slap yourself!!!
- [ ] slapping is case sensitive
