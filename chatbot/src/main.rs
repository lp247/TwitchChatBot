extern crate websocket;

use reqwest;
use std::{env, thread, time};
use serde::Deserialize;
use websocket::client::ClientBuilder;
use websocket::Message;
use std::convert::Infallible;
use std::net::SocketAddr;
use hyper::{Body, Request, Response, Server};
use hyper::service::{make_service_fn, service_fn};
use futures::join;

const TWITCH_CHAT_SERVER: &'static str = "ws://irc-ws.chat.twitch.tv:80";

// #[derive(Deserialize)]
// struct AuthorizationData {
//     access_token: String,
//     expires_in: u32,
//     token_type: String,
// }

async fn hello_world(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("Hello, World".into()))
}

async fn recv_msgs() {
    let chat_client = ClientBuilder::new(TWITCH_CHAT_SERVER)
        .unwrap()
        .connect_insecure()
        .unwrap();
    let (mut receiver, mut sender) = chat_client.split().unwrap();
    let message = Message::text("Some unknown command");
    sender.send_message(&message);
    for message in receiver.incoming_messages() {
        println!("Recv: {:?}", message.unwrap());
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let code_http_server_addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = Server::bind(&code_http_server_addr).serve(make_svc);

    let recv_msgs_future = recv_msgs();

    // join!(server);
    join!(server, recv_msgs_future);

    Ok(())
}
