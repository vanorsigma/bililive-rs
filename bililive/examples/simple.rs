use futures::{SinkExt, StreamExt};
use log::info;
use serde_json::Value;

use bililive::{ConfigBuilder, Operation, Packet, Protocol, RetryConfig};

async fn run() {
    pretty_env_logger::init();

    let config = if let Ok(token) = std::env::var("BILIBILI_TOKEN") {
        ConfigBuilder::new()
            .sess_token(token.as_str())
            .by_uid(3537122061453707)
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
                info!("raw: {:?}", packet);
                if packet.op() == Operation::RoomEnterResponse {
                    info!("Replying room enter response with heartbeat");
                    stream.send(Packet::new(Operation::HeartBeat, Protocol::Json, "{}".as_bytes())).await.expect("can send heartbeat");
                }

                if let Ok(json) = packet.json::<Value>() {
                    info!("json: {:?}", json);
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
