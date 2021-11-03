extern crate websocket;

mod proxies;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Start");
    let mut proxy = proxies::TwitchChatProxy::new("captaincallback");
    proxy.initialize().await;
    proxy.send_message("Hello, World!");
    for message in proxy.incoming_messages() {
        println!("User {} wrote message {}", message.user, message.text);
    };
    println!("Finished");
    Ok(())
}
