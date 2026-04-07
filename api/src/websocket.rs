//! WebSocket 支持

use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::Response,
};
use serde::{Deserialize, Serialize};
use tracing::info;

/// WebSocket 消息
#[derive(Debug, Serialize, Deserialize)]
pub enum WsMessage {
    /// 订阅新区块
    SubscribeBlocks,
    /// 订阅新交易
    SubscribeTransactions,
    /// 新区块通知
    NewBlock { height: u64, hash: String },
    /// 新交易通知
    NewTransaction { hash: String },
    /// 心跳
    Ping,
    /// 心跳响应
    Pong,
}

/// WebSocket 处理器
pub async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

/// 处理 WebSocket 连接
async fn handle_socket(mut socket: WebSocket) {
    info!("WebSocket 连接建立");

    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(text) => {
                    // 尝试解析消息
                    if let Ok(ws_msg) = serde_json::from_str::<WsMessage>(&text) {
                        let response = match ws_msg {
                            WsMessage::Ping => WsMessage::Pong,
                            WsMessage::SubscribeBlocks => {
                                info!("客户端订阅区块");
                                WsMessage::Pong
                            }
                            WsMessage::SubscribeTransactions => {
                                info!("客户端订阅交易");
                                WsMessage::Pong
                            }
                            _ => WsMessage::Pong,
                        };
                        
                        let json = serde_json::to_string(&response).unwrap();
                        if socket.send(Message::Text(json)).await.is_err() {
                            break;
                        }
                    }
                }
                Message::Ping(data) => {
                    if socket.send(Message::Pong(data)).await.is_err() {
                        break;
                    }
                }
                Message::Close(_) => {
                    info!("WebSocket 连接关闭");
                    break;
                }
                _ => {}
            }
        } else {
            break;
        }
    }

    info!("WebSocket 连接结束");
}
