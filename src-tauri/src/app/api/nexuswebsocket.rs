use std::fs::File;
use std::io::{Read, Write};
use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tungstenite::Message;
use tungstenite::protocol::CloseFrame;
use tungstenite::protocol::frame::coding::CloseCode;
use url::Url;
use futures_util::{SinkExt, StreamExt};
use crate::app::util::app_path;

#[derive(Serialize, Deserialize, Debug)]
pub struct WebsocketDataWrapper {
    success: bool,
    data: ConnectionData,
    error: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConnectionData {
    connection_token: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiKeyWrapper {
    success: bool,
    data: ApiKey,
    error: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiKey {
    api_key: String
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WebsocketData {
    id: String,
    token: Option<String>,
    protocol: i32
}

fn create_uuid() -> String {
    let mut rng = thread_rng();
    format!(
        "{:08x}-{:04x}-4{:03x}-{}{:03x}-{:012x}",
        rng.gen::<u32>(),
        rng.gen::<u16>(),
        rng.gen::<u16>() & 0x0fff, // Ensure the top 4 bits are 0000
        match rng.gen::<u8>() & 0b1100 { // Variant must be 10xx
            0b1000 => '8',
            0b1100 => '9',
            0b0100 => 'a',
            _ => 'b',
        },
        rng.gen::<u16>() & 0x0fff,
        rng.gen::<u32>()
    )
}

fn load_binary() -> WebsocketData {
    if app_path("connection.stp").exists() {
        let mut file = File::open(app_path("connection.stp")).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let data : WebsocketData = bincode::deserialize(&buffer[..]).unwrap();

        return data
    }

    let data = WebsocketData {
        id: create_uuid(),
        token: None,
        protocol: 2
    };
    data
}

pub fn load_key() -> String {
    if app_path("key.stp").exists() {
        let mut file = File::open(app_path("key.stp")).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let data : String = bincode::deserialize(&buffer[..]).unwrap();

        return data
    }

    "".to_string()
}

fn save_binary(data: &WebsocketData) {
    let encoded: Vec<u8> = bincode::serialize(&data).unwrap();
    let mut file = File::create(app_path("connection.stp")).unwrap();
    file.write_all(&encoded).unwrap();
}

fn save_key(key: String) {
    let encoded: Vec<u8> = bincode::serialize(&key).unwrap();
    let mut file = File::create(app_path("key.stp")).unwrap();
    file.write_all(&encoded).unwrap();
}


pub async fn connect_user(handle: tauri::AppHandle) {
    let url = Url::parse("wss://sso.nexusmods.com").unwrap();
    let (mut websocket, response) = connect_async(url).await.expect("Failed to connect");

    let data = load_binary();
    let json_string = serde_json::to_string(&data).unwrap();
    let sent_data = data.to_owned();

    let msg = Message::Text(json_string);
    if let Err(e) = websocket.send(msg).await {
        eprintln!("Failed to send message: {}", e);
    }

    let mut nexus_link: String = "https://www.nexusmods.com/sso?id=".to_owned();
    nexus_link.push_str(data.id.to_owned().to_string().as_str());

    tauri::WindowBuilder::new(
        &handle,
        "NexusMod",
        tauri::WindowUrl::External(Url::parse(nexus_link.as_str()).unwrap())
    ).title("Configure").build().unwrap();

    receive_messages(&mut websocket, sent_data).await;
}

async fn receive_messages(websocket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>, mut data: WebsocketData) {
    while let Some(message) = websocket.next().await {
        match message {
            Ok(msg) => {
                let msg_text = msg.to_text().unwrap();
                if msg_text.contains("connection_token") {
                    let wrapper: WebsocketDataWrapper = serde_json::from_str(msg.to_text().unwrap()).unwrap();
                    data.token = Some(wrapper.data.connection_token);
                    save_binary(&data);
                }
                else if msg_text.contains("api_key") {
                    let wrapper: ApiKeyWrapper = serde_json::from_str(msg.to_text().unwrap()).unwrap();
                    save_key(wrapper.data.api_key);

                    let close_frame = Some(CloseFrame {
                        code: CloseCode::Normal,
                        reason: "Normal Closure".into(),
                    });

                    websocket.send(Message::Close(close_frame)).await.expect("Couldn't close wws");
                }
                else {
                    println!("Received close frame from server.");
                    break;
                }
            },
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                break;
            },
        }
    }
}