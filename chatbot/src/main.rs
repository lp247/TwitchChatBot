extern crate websocket;

mod connect;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Start");
    let mut proxy = connect::TwitchChatConnector::new("captaincallback");
    proxy.initialize().await;
    proxy.send_message("Hello, World!");
    for message in proxy.incoming_messages() {
        println!("{}: {}", message.user, message.text);
    }
    println!("Finished");
    Ok(())
}
