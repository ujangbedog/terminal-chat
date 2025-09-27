use crate::state::SharedState;
use futures::{SinkExt, StreamExt};
use shared::{utils, Message};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::{Framed, LinesCodec};
use tracing::{error, info};
use uuid::Uuid;

/// handle a single client connection
pub async fn handle_client(
    client_id: Uuid,
    stream: TcpStream,
    state: Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = stream.peer_addr()?;
    let mut lines = Framed::new(stream, LinesCodec::new());
    
    // create channel for this client
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    
    // add client to shared state
    {
        let mut state_guard = state.lock().await;
        state_guard.add_client(client_id, tx, addr);
    }

    // send welcome message
    let welcome_msg = Message::System {
        content: "Welcome to the chat! Please send your username to join.".to_string(),
    };
    let welcome_str = utils::serialize_message(&welcome_msg)?;
    lines.send(welcome_str).await?;

    // split the framed stream into sink and stream
    let (mut sink, mut stream) = lines.split();
    
    let outgoing_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if let Err(e) = sink.send(message).await {
                error!("Failed to send message to client: {}", e);
                break;
            }
        }
    });

    // handle incoming messages
    let mut username_set = false;
    while let Some(line_result) = stream.next().await {
        match line_result {
            Ok(line) => {
                if let Err(e) = handle_client_message(
                    client_id,
                    &line,
                    &mut username_set,
                    &state,
                ).await {
                    error!("Error handling client message: {}", e);
                    break;
                }
            }
            Err(e) => {
                error!("Error reading from client {}: {}", client_id, e);
                break;
            }
        }
    }

    // cleanup when client disconnects
    outgoing_task.abort();
    let username = {
        let mut state_guard = state.lock().await;
        state_guard.remove_client(&client_id)
    };

    // notify other clients about disconnection
    if let Some(username) = username {
        println!("[LEAVE] User '{}' left the chat", username);
        info!("User {} left the chat", username);
        let leave_msg = Message::Leave { username };
        let state_guard = state.lock().await;
        state_guard.broadcast(Some(client_id), &leave_msg).await;
    }

    Ok(())
}

/// handle a message from a client
async fn handle_client_message(
    client_id: Uuid,
    line: &str,
    username_set: &mut bool,
    state: &Arc<Mutex<SharedState>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if !*username_set {
        // first message should be the username
        let username = line.trim().to_string();
        
        if !utils::is_valid_username(&username) {
            let error_msg = Message::System {
                content: "Invalid username. Use only alphanumeric characters, underscore, or dash (max 32 chars)".to_string(),
            };
            let error_str = utils::serialize_message(&error_msg)?;
            
            let state_guard = state.lock().await;
            if let Some(client_info) = state_guard.get_client(&client_id) {
                let _ = client_info.sender.send(error_str);
            }
            return Ok(());
        }

        // try to set username
        let result = {
            let mut state_guard = state.lock().await;
            state_guard.set_username(&client_id, username.clone())
        };

        match result {
            Ok(()) => {
                *username_set = true;
                
                // send confirmation and user list
                let join_msg = Message::Join { username: username.clone() };
                let usernames = {
                    let state_guard = state.lock().await;
                    state_guard.get_usernames()
                };
                let user_list_msg = Message::UserList { users: usernames };
                
                // log and broadcast join message to others
                println!("[JOIN] User '{}' joined the chat", username);
                info!("User {} joined the chat", username);
                {
                    let state_guard = state.lock().await;
                    state_guard.broadcast(Some(client_id), &join_msg).await;
                }
                
                // send user list to the new client
                let user_list_str = utils::serialize_message(&user_list_msg)?;
                let state_guard = state.lock().await;
                if let Some(client_info) = state_guard.get_client(&client_id) {
                    let _ = client_info.sender.send(user_list_str);
                }
            }
            Err(e) => {
                let error_msg = Message::System { content: e };
                let error_str = utils::serialize_message(&error_msg)?;
                
                let state_guard = state.lock().await;
                if let Some(client_info) = state_guard.get_client(&client_id) {
                    let _ = client_info.sender.send(error_str);
                }
            }
        }
    } else {
        // handle chat message
        let content = line.trim().to_string();
        
        if !utils::is_valid_message_content(&content) {
            let error_msg = Message::System {
                content: "Message is empty or too long".to_string(),
            };
            let error_str = utils::serialize_message(&error_msg)?;
            
            let state_guard = state.lock().await;
            if let Some(client_info) = state_guard.get_client(&client_id) {
                let _ = client_info.sender.send(error_str);
            }
            return Ok(());
        }

        // get username and broadcast message
        let username = {
            let state_guard = state.lock().await;
            state_guard.get_client(&client_id)
                .and_then(|client| client.username.clone())
        };

        if let Some(username) = username {
            let chat_msg = Message::Chat { 
                username: username.clone(), 
                content: content.clone() 
            };
            
            // log the message relay on server
            println!("[RELAY] {}: {}", username, content);
            info!("Relaying message from {}: {}", username, content);
            
            let state_guard = state.lock().await;
            state_guard.broadcast(Some(client_id), &chat_msg).await;
        }
    }

    Ok(())
}
