use super::{
    auth::AccessTokenDispenser,
    receive::{ConnectorEvent, ReceiveEvent},
    send::SendTask,
};
use crate::{
    app_config::AppConfig,
    connect::{
        connector::twitch_chat::{auth::TwitchAccessTokenDispenser, send::get_login_tasks},
        error::ConnectorError,
        ChatBotEvent,
    },
};
use futures::{
    channel::mpsc, future::select, lock::Mutex, pin_mut, Sink, SinkExt, Stream, StreamExt,
};
use std::sync::Arc;
use tokio_tungstenite::{
    connect_async,
    tungstenite::{self, Message},
};

async fn handle_input(
    receiver: Arc<Mutex<impl Stream<Item = Result<Message, tungstenite::Error>> + Unpin>>,
    chatbot_sender: Arc<Mutex<impl Sink<ChatBotEvent, Error = mpsc::SendError> + Unpin>>,
    mut internal_sender: impl Sink<ConnectorEvent, Error = mpsc::SendError> + Unpin,
) -> Result<(), ConnectorError> {
    let receiver = Arc::clone(&receiver);
    let receiver = &mut *receiver.lock().await;
    while let Some(message) = receiver.next().await {
        match message {
            Ok(Message::Text(text_message)) => {
                log::debug!("<- {}", text_message.trim());
                let received_events = text_message
                    .lines()
                    .filter_map(ReceiveEvent::parse_from_message);
                for event in received_events {
                    match event {
                        ReceiveEvent::ChatBotEvent(chat_bot_event) => {
                            chatbot_sender.lock().await.send(chat_bot_event).await?;
                        }
                        ReceiveEvent::ConnectorEvent(connector_event) => {
                            internal_sender.send(connector_event).await?
                        }
                    }
                }
            }
            _ => (),
        }
    }
    log::warn!("Input handler stopped");
    Ok(())
}

async fn handle_output(
    sender: Arc<Mutex<impl Sink<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin>>,
    chatbot_receiver: Arc<Mutex<impl Stream<Item = String> + Unpin>>,
    mut internal_receiver: impl Stream<Item = ConnectorEvent> + Unpin,
) -> Result<(), ConnectorError> {
    let internal_output_fut = async {
        while let Some(event) = internal_receiver.next().await {
            let sender = Arc::clone(&sender);
            match event {
                ConnectorEvent::Ping => {
                    let text = SendTask::Pong.to_string();
                    log::debug!("-> {}", text);
                    sender.lock().await.send(Message::Text(text)).await?;
                }
            }
        }
        log::warn!("Internal output handler stopped");
        Ok::<(), ConnectorError>(())
    };
    let chatbot_output_fut = async {
        while let Some(message) = chatbot_receiver.lock().await.next().await {
            let sender = Arc::clone(&sender);
            let text = SendTask::PrivateMessage("captaincallback".to_owned(), message).to_string();
            log::debug!("-> {}", text);
            sender.lock().await.send(Message::Text(text)).await?;
        }
        log::warn!("Chatbot output handler stopped");
        Ok::<(), ConnectorError>(())
    };
    pin_mut!(internal_output_fut, chatbot_output_fut);
    select(internal_output_fut, chatbot_output_fut).await;
    log::warn!("Output handler stopped");
    Ok(())
}

async fn connect_to_twitch_chat_with_connection<'a>(
    from_chatbot: Arc<Mutex<impl Stream<Item = String> + Unpin>>,
    to_chatbot: Arc<Mutex<impl Sink<ChatBotEvent, Error = mpsc::SendError> + Unpin>>,
    from_websocket: Arc<Mutex<impl Stream<Item = Result<Message, tungstenite::Error>> + Unpin>>,
    to_websocket: Arc<Mutex<impl Sink<Message, Error = tungstenite::Error> + Unpin>>,
    app_config: &'a AppConfig,
    access_token_dispenser: &'a mut impl AccessTokenDispenser,
) -> Result<(), ConnectorError> {
    let access_token: String = access_token_dispenser
        .get()
        .await
        .expect("Could not get valid access token")
        .to_owned();

    let (internal_tx, internal_rx) = mpsc::unbounded::<ConnectorEvent>();

    let output_fut = handle_output(Arc::clone(&to_websocket), from_chatbot, internal_rx);
    let input_fut = handle_input(from_websocket, to_chatbot, internal_tx);

    let login_tasks = get_login_tasks(
        &access_token,
        app_config.bot_user_name(),
        app_config.channel_name(),
    );
    for task in login_tasks {
        to_websocket
            .lock()
            .await
            .send(Message::Text(task.to_string()))
            .await?;
    }

    pin_mut!(input_fut, output_fut);
    select(input_fut, output_fut).await;
    Ok(())
}

pub async fn connect_to_twitch_chat<'a>(
    mut from_chatbot: impl Stream<Item = String> + Unpin,
    mut to_chatbot: impl Sink<ChatBotEvent, Error = mpsc::SendError> + Unpin,
    app_config: &'a AppConfig,
) -> Result<(), ConnectorError> {
    let mut access_token_dispenser = TwitchAccessTokenDispenser::new(app_config)
        .await
        .expect("Could not instantiate Twitch connector");

    loop {
        let (ws_stream, _) = connect_async("ws://irc-ws.chat.twitch.tv:80").await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();
        let to_chatbot = Arc::new(Mutex::new(&mut to_chatbot));
        let from_chatbot = Arc::new(Mutex::new(&mut from_chatbot));
        let to_websocket = Arc::new(Mutex::new(&mut ws_write));
        let from_websocket = Arc::new(Mutex::new(&mut ws_read));
        connect_to_twitch_chat_with_connection(
            from_chatbot,
            to_chatbot,
            from_websocket,
            to_websocket,
            app_config,
            &mut access_token_dispenser,
        )
        .await?;
        log::warn!("Connection dropped, connection will be re-established");
    }
}

#[cfg(test)]
mod tests {
    use std::pin::Pin;
    use std::task::Context;
    use std::task::Poll;

    use async_trait::async_trait;
    use futures::future::Either;
    use futures::StreamExt;

    use super::*;

    struct MockAccessTokenDispenser(String);

    #[async_trait]
    impl AccessTokenDispenser for MockAccessTokenDispenser {
        async fn get(&mut self) -> Result<&str, ConnectorError> {
            Ok(self.0.as_str())
        }
    }

    struct DummyWSInput {
        inputs: Vec<Message>,
        pointer: usize,
    }

    impl DummyWSInput {
        fn new(inputs: Vec<Message>) -> DummyWSInput {
            Self { inputs, pointer: 0 }
        }
    }

    impl Stream for DummyWSInput {
        type Item = Result<Message, tokio_tungstenite::tungstenite::Error>;

        fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let ret = if self.pointer < self.inputs.len() {
                Poll::Ready(self.inputs.get(self.pointer).map(|x| x.clone()).map(Ok))
            } else {
                Poll::Pending
            };
            self.pointer += 1;
            ret
        }
    }

    #[tokio::test]
    async fn sends_login_data_at_the_beginning() {
        let app_config = AppConfig {
            bot_user_name: "botname".to_owned(),
            channel_name: "channelname".to_owned(),
            twitch_client_id: "client_id".to_owned(),
            twitch_client_secret: "client_secret".to_owned(),
        };
        let (mut to_cb_tx, _) = mpsc::unbounded::<ChatBotEvent>();
        let (_from_cb_tx, mut from_cb_rx) = mpsc::unbounded::<String>();
        let mut from_ws_rx = DummyWSInput::new(Vec::default());
        let (to_ws_tx, to_ws_rx) = mpsc::unbounded::<Message>();
        let mut ws_out_tx_mod =
            to_ws_tx.sink_map_err(|_| tokio_tungstenite::tungstenite::Error::ConnectionClosed);
        let from_cb_rx = Arc::new(Mutex::new(&mut from_cb_rx));
        let to_cb_tx = Arc::new(Mutex::new(&mut to_cb_tx));
        let from_ws_rx = Arc::new(Mutex::new(&mut from_ws_rx));
        let ws_out_tx_mod = Arc::new(Mutex::new(&mut ws_out_tx_mod));
        let mut dispenser = MockAccessTokenDispenser("ACCESS_TOKEN".to_owned());
        let conn_fut = connect_to_twitch_chat_with_connection(
            from_cb_rx,
            to_cb_tx,
            from_ws_rx,
            ws_out_tx_mod,
            &app_config,
            &mut dispenser,
        );
        let sent_msgs_fut = to_ws_rx.take(6).collect::<Vec<Message>>();
        pin_mut!(conn_fut, sent_msgs_fut);
        if let Either::Right(sent_msgs) = select(conn_fut, sent_msgs_fut).await {
            if let Some(Message::Text(first)) = sent_msgs.0.get(0) {
                assert_eq!(first, "PASS oauth:ACCESS_TOKEN")
            } else {
                unreachable!();
            }
            if let Some(Message::Text(first)) = sent_msgs.0.get(1) {
                assert_eq!(first, "NICK botname")
            } else {
                unreachable!();
            }
            if let Some(Message::Text(first)) = sent_msgs.0.get(2) {
                assert_eq!(first, "JOIN #channelname")
            } else {
                unreachable!();
            }
            if let Some(Message::Text(first)) = sent_msgs.0.get(3) {
                assert_eq!(first, "CAP REQ :twitch.tv/membership")
            } else {
                unreachable!();
            }
            if let Some(Message::Text(first)) = sent_msgs.0.get(4) {
                assert_eq!(first, "CAP REQ :twitch.tv/tags")
            } else {
                unreachable!();
            }
            if let Some(Message::Text(first)) = sent_msgs.0.get(5) {
                assert_eq!(first, "PRIVMSG #channelname :Hello, world!");
            } else {
                unreachable!();
            }
        }
    }

    #[tokio::test]
    async fn responds_with_pong_to_ping() {
        let app_config = AppConfig {
            bot_user_name: "botname".to_owned(),
            channel_name: "channelname".to_owned(),
            twitch_client_id: "client_id".to_owned(),
            twitch_client_secret: "client_secret".to_owned(),
        };
        let (mut to_cb_tx, _) = mpsc::unbounded::<ChatBotEvent>();
        let (_from_cb_tx, mut from_cb_rx) = mpsc::unbounded::<String>();
        let mut from_ws_rx =
            DummyWSInput::new(vec![Message::Text("PING :tmi.twitch.tv".to_owned())]);
        let (to_ws_tx, to_ws_rx) = mpsc::unbounded::<Message>();
        let mut ws_out_tx_mod =
            to_ws_tx.sink_map_err(|_| tokio_tungstenite::tungstenite::Error::ConnectionClosed);
        let from_cb_rx = Arc::new(Mutex::new(&mut from_cb_rx));
        let to_cb_tx = Arc::new(Mutex::new(&mut to_cb_tx));
        let from_ws_rx = Arc::new(Mutex::new(&mut from_ws_rx));
        let ws_out_tx_mod = Arc::new(Mutex::new(&mut ws_out_tx_mod));
        let mut dispenser = MockAccessTokenDispenser("ACCESS_TOKEN".to_owned());
        let conn_fut = connect_to_twitch_chat_with_connection(
            from_cb_rx,
            to_cb_tx,
            from_ws_rx,
            ws_out_tx_mod,
            &app_config,
            &mut dispenser,
        );
        let sent_msgs_fut = to_ws_rx.take(7).collect::<Vec<Message>>();
        pin_mut!(conn_fut, sent_msgs_fut);
        if let Either::Right(sent_msgs) = select(conn_fut, sent_msgs_fut).await {
            log::warn!("{:?}", sent_msgs.0);
            if let Some(Message::Text(pong)) = sent_msgs.0.get(6) {
                assert_eq!(pong, "PONG :tmi.twitch.tv")
            } else {
                unreachable!();
            }
        }
    }
}
