# Twitch Chat Bot
This is a chat bot for twitch streams.

## Run
### Run with Cargo
To simply run this chat bot change to directory `chatbot` and run `cargo run`. This also requires some dependencies (e.g. libssl) to be installed. [Configuration options](#configuration-options) must be provided as environment variables. These can also be defined within the `chatbot/.env` file.

### Run with Docker
#### Build image
To build the chat bot image run `docker build -t chatbot .` 

#### Create container
To create the chatbot container run `docker run -it --rm --name chatbot-app -p 3030:3030 chatbot` in the project's root directory. [Configuration options](#configuration-options) must be provided as environment variables which can be provided to the docker container via the `-e` option. Additionally, these can also be defined within the `chatbot/.env` file.

### Configuration options
- TWITCH_CHANNEL: The twitch channel name to connect to (lowercase version of the name of the streamer)
- TWITCH_CHAT_USER: The name of the user to be used by the chat bot.
- TWITCH_AUTH_CLIENT_ID: The client ID of the user to be used by the chat bot.
- TWITCH_AUTH_CLIENT_SECRET: The client secret of the user to be used by the chat bot.

## Commands
### !help
Returns a list of supported commands.

### !info
Returns some basic information about this chat bot.

### !newcommand <command_name> <Text to return>
Create a dynamic command which returns a simple text.

### !removecommand <command_name>
Removes a dynamic command.
