mod connection;
mod ui;
mod chat;

use chat::run_chat_client;
use ui::display_header;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("chat_client=info".parse()?),
        )
        .init();

    display_header();
    
    run_chat_client().await?;

    Ok(())
}
