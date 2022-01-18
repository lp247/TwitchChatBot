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
- [X] Timed repetitions
- [X] Simplify and fully automate the process for getting the access token
- [X] Retrying in connections (mainly in authentication)
- [X] Refresh access tokens with refresh tokens
- [X] What do we do with pings when no events are received? They should still be answered!
- [X] LOGGING (flexi_logger)

Tests:

New Feature:
- [ ] Persistence (file or sqlite with rusqlite)
- [ ] Need a way to keep track of names of new commands and repeatings => create CLI
- [ ] Respect chat message rate limits
- [ ] Spam protection (machine learning?)
- [ ] Polls
- [ ] Management UI
- [ ] Viewer Statistics
- [ ] Chat logs

Setup:

Something to think about:
- [ ] Can display names really be only upper case versions of the user name? If not, what else should be considered?
- [ ] Maybe put all the looping logic of the main.rs file into a separate event_loop.rs file
- [ ] Proper error handling in all the program
- [ ] Remove all hard coded string messages (discord, info, ...)

Bugs:
- [ ] Bot is not responding to multiple fast sent commands
- [ ] You can slap yourself!!!
- [ ] slapping is case sensitive
- [ ] "Reader thread stopped with error MessageReceiveFailed("NoDataAvailable")" after some time
