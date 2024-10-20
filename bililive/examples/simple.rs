use futures::{SinkExt, StreamExt};
use log::info;
use serde::Serialize;
use serde_json::Value;

use bililive::{ConfigBuilder, Operation, Packet, Protocol, RetryConfig};

#[derive(Serialize)]
struct ExportType {
    uid: String,
    username: String,
    message: String,
    avatar: String,
}

/**
println!("Face: {}", face);
println!("Name: {}", name);
println!("Userhash: {}", userhash);
println!("Content: {}", content);
*/

async fn run() {
    pretty_env_logger::init();

    let config = if let Ok(token) = std::env::var("BILIBILI_TOKEN") {
        ConfigBuilder::new()
            .sess_token(token.as_str())
            .by_uid(3546729368520811)
            .await
            .unwrap()
            .fetch_conf()
            .await
            .unwrap()
            .build()
    } else {
        ConfigBuilder::new()
            .by_uid(90873)
            .await
            .unwrap()
            .fetch_conf()
            .await
            .unwrap()
            .build()
    };

    info!("room_id: {}", config.room_id());
    info!("buvid: {}", config.buvid());
    info!("uid: {}", config.uid());
    info!("token: {}", config.token());
    info!("servers: {:#?}", config.servers());

    #[cfg(feature = "tokio")]
    let mut stream =
        bililive::connect::tokio::connect_with_retry(config.clone(), RetryConfig::default())
            .await
            .unwrap();
    #[cfg(feature = "async-std")]
    let mut stream =
        bililive::connect::async_std::connect_with_retry(config, RetryConfig::default())
            .await
            .unwrap();

    while let Some(e) = stream.next().await {
        match e {
            Ok(packet) => {
                if packet.op() == Operation::RoomEnterResponse {
                    info!("Replying room enter response with heartbeat");
                    stream
                        .send(Packet::new(
                            Operation::HeartBeat,
                            Protocol::Json,
                            "{}".as_bytes(),
                        ))
                        .await
                        .expect("can send heartbeat");
                }

                if let Ok(json) = packet.json::<Value>() {
                    info!("json: {:?}", json);

                    // TODO: Just a very fast hack for me to get a JSON object to save
                    if let Some(cmd) = json.get("cmd") {
                        if cmd == "DANMU_MSG" {
                            if let Some(info) = json.get("info").and_then(|v| v.as_array()) {
                                if let Some(user_info) = info.get(0).and_then(|v| v.as_array()) {
                                    if let Some(user) =
                                        user_info.get(15).and_then(|v| v.get("user"))
                                    {
                                        if let Some(base) = user.get("base") {
                                            let face = base
                                                .get("face")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            let name = base
                                                .get("name")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            let (userhash, content) = user_info
                                                .get(15)
                                                .and_then(|v| v.get("extra"))
                                                .and_then(|v| v.as_str())
                                                .and_then(|v| serde_json::from_str::<Value>(v).ok())
                                                .and_then(|v| Some((v.get("user_hash")?.clone(), v.get("content")?.clone())))
                                                .and_then(|(v1, v2)| Some((v1.as_str()?.to_string(), v2.as_str()?.to_string())))
                                                .unwrap_or((String::new(), String::new()));

                                            println!(
                                                "{}",
                                                serde_json::to_string(&ExportType {
                                                    uid: userhash,
                                                    username: name.to_string(),
                                                    message: content,
                                                    avatar: face.to_string()
                                                })
                                                .unwrap()
                                            );
                                            // println!("Face: {}", face);
                                            // println!("Name: {}", name);
                                            // println!("Userhash: {}", userhash);
                                            // println!("Content: {}", content);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                // info!("err: {:?}", e);
            }
        }
    }
}

fn main() {
    #[cfg(feature = "tokio")]
    {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();
        return runtime.block_on(run());
    }
    #[cfg(feature = "async-std")]
    return async_std::task::block_on(run());
}
