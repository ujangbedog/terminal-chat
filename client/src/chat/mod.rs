use crate::connection::connect_to_server;
use crate::ui::{display_welcome, get_username};
use futures::{SinkExt, StreamExt};
use shared::utils;
use std::io;
use tokio::sync::mpsc;

/// run the main chat client
pub async fn run_chat_client() -> Result<(), Box<dyn std::error::Error>> {
    // get username from user
    let username = get_username()?;

    // connect to server
    let mut lines = connect_to_server().await?;
    
    // send username
    lines.send(username.clone()).await?;
    
    display_welcome(&username);

    // create channels for communication between tasks
    let (input_tx, mut input_rx) = mpsc::unbounded_channel::<String>();
    
    // spawn task to handle user input
    let input_tx_clone = input_tx.clone();
    tokio::spawn(async move {
        loop {
            let input_result = tokio::task::spawn_blocking(move || {
                let mut line = String::new();
                match io::stdin().read_line(&mut line) {
                    Ok(_) => Ok(line),
                    Err(e) => Err(e),
                }
            }).await;
            
            match input_result {
                Ok(Ok(line)) => {
                    let message = line.trim().to_string();
                    if !message.is_empty() {
                        if message == "/quit" || message == "/exit" {
                            break;
                        }
                        if let Err(_) = input_tx_clone.send(message) {
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
    });

    // main loop
    loop {
        tokio::select! {
            // handle messages from server
            line_result = lines.next() => {
                match line_result {
                    Some(Ok(line)) => {
                        match utils::deserialize_message(&line) {
                            Ok(message) => {
                                println!("{}", message);
                            }
                            Err(_) => {
                                println!("*** Received invalid message format");
                            }
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("Error reading from server: {}", e);
                        break;
                    }
                    None => {
                        println!("*** Server disconnected");
                        break;
                    }
                }
            }
            
            // handle user input
            input = input_rx.recv() => {
                match input {
                    Some(message) => {
                        if utils::is_valid_message_content(&message) {
                            // show our own message with our username
                            println!("{}: {}", username, message);
                            
                            if let Err(e) = lines.send(message).await {
                                eprintln!("Error sending message: {}", e);
                                break;
                            }
                        } else {
                            println!("*** Message is empty or too long");
                        }
                    }
                    None => break,
                }
            }
        }
    }

    println!("*** Disconnected from server");
    Ok(())
}
