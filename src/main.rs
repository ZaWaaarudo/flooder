use async_std::{
    net::TcpStream,
    task,
    prelude::*,
    io::BufWriter,
};
use futures::future;
use std::fs;
use yaml_rust::YamlLoader;

fn main() {
    let config = fs::read_to_string("config.yaml").unwrap();
    let doc = YamlLoader::load_from_str(config.as_str()).unwrap().first().unwrap().clone();

    let address = doc["address"].as_str().unwrap();
    let port = doc["port"].as_i64().unwrap();
    let connections = doc["connections"].as_i64().unwrap();
    let messages = doc["messages"].as_i64().unwrap();
    let message = doc["message"].as_str().unwrap();

    let mut clients = Vec::with_capacity(connections as usize);

    for id in 0..connections {
        clients.push(client(address, port, messages, message, id));
    }

    let _ = task::block_on(async {
        let _ = future::join_all(clients).await;
    });

    println!("Flooding done");
    
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
}

async fn client(address: &str, port: i64, messages: i64, message: &str, id: i64) {
    let stream: TcpStream = match TcpStream::connect(format!("{}:{}", address, port)).await {
        Ok(stream) => stream,
        Err(err) => {
            println!("client {} error {}", id, err);
            return;
        },
    };

    let mut writer = BufWriter::with_capacity(message.len() + 1, &stream);

    for msg_id in 0..messages {
        if let Err(err) = writer.write_all(message.as_bytes()).await {
            println!("client {} message {} error {}", id, msg_id, err);
            break;
        }
        if let Err(err) = writer.write(b"\n").await {
            println!("client {} message {} error {}", id, msg_id, err);
            break;
        }
        if let Err(err) = writer.flush().await {
            println!("client {} message {} error {}", id, msg_id, err);
        }
    }
}