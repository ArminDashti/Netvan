use anyhow::Result;
use netvan_collectors::CollectorEngine;
use netvan_core::db::Database;
use netvan_core::ipc::{decode_frame, encode_message, RpcEnvelope, RpcReply, RpcResponse};
use netvan_core::paths::PIPE_NAME;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tracing::{error, info, warn};

#[cfg(windows)]
mod win_pipe {
    use super::*;
    use tokio::net::windows::named_pipe::ServerOptions;

    pub async fn serve(engine: Arc<CollectorEngine>) -> Result<()> {
        info!("listening on named pipe {PIPE_NAME}");
        loop {
            let server = ServerOptions::new()
                .first_pipe_instance(false)
                .create(PIPE_NAME)?;
            server.connect().await?;
            let eng = engine.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_client(server, eng).await {
                    warn!("pipe client error: {e:#}");
                }
            });
        }
    }

    async fn handle_client(
        mut pipe: tokio::net::windows::named_pipe::NamedPipeServer,
        engine: Arc<CollectorEngine>,
    ) -> Result<()> {
        let mut buf = Vec::new();
        let mut tmp = [0u8; 8192];
        loop {
            let n = match pipe.read(&mut tmp).await {
                Ok(0) => break,
                Ok(n) => n,
                Err(e) => {
                    error!("pipe read: {e}");
                    break;
                }
            };
            buf.extend_from_slice(&tmp[..n]);
            while let Some((consumed, frame)) = decode_frame(&buf) {
                let reply = match serde_json::from_slice::<RpcEnvelope>(frame) {
                    Ok(env) => {
                        let response = engine.handle(env.request).await;
                        RpcReply {
                            id: env.id,
                            response,
                        }
                    }
                    Err(e) => RpcReply {
                        id: 0,
                        response: RpcResponse::Error {
                            message: format!("bad request: {e}"),
                        },
                    },
                };
                let encoded = encode_message(&reply)?;
                pipe.write_all(&encoded).await?;
                buf.drain(..consumed);
            }
        }
        Ok(())
    }
}

#[cfg(not(windows))]
mod win_pipe {
    use super::*;
    pub async fn serve(_engine: Arc<CollectorEngine>) -> Result<()> {
        anyhow::bail!("named pipe server is Windows-only");
    }
}

pub async fn run() -> Result<()> {
    netvan_core::paths::ensure_data_dir()?;
    let db = Database::open_default()?;
    let engine = CollectorEngine::new(db)?;
    engine.clone().start_background().await;
    info!("Netvan service running");
    win_pipe::serve(engine).await
}
