use anyhow::Result;
use bytes::BytesMut;
use futures::SinkExt;
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::{Decoder, Encoder, Framed};
use tracing::info;

use crate::{
    cmd::Command, Backend, CommandExcetor, RespDecoder, RespEncoder, RespError, RespFrame,
};

struct RespFrameCodec;

struct RedisRequest {
    frame: RespFrame,
    backend: Backend,
}
struct RedisResponse {
    frame: RespFrame,
}

impl Encoder<RespFrame> for RespFrameCodec {
    type Error = anyhow::Error;
    fn encode(&mut self, item: RespFrame, dst: &mut BytesMut) -> Result<()> {
        let encoded = item.encode();
        dst.extend_from_slice(&encoded);
        Ok(())
    }
}
impl Decoder for RespFrameCodec {
    type Item = RespFrame;
    type Error = anyhow::Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<RespFrame>> {
        match RespFrame::decode(src) {
            Ok(frame) => Ok(Some(frame)),
            Err(RespError::NotComplete) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
}
pub async fn stream_handler(_stream: TcpStream, backend: Backend) -> Result<()> {
    let mut framed = Framed::new(_stream, RespFrameCodec);
    loop {
        match framed.next().await {
            Some(Ok(frame)) => {
                info!("Received frame: {:?}", frame);
                let request = RedisRequest {
                    frame,
                    backend: backend.clone(),
                };
                let response = request_handler(request).await?;
                info!("Sending response: {:?}", response.frame);
                framed.send(response.frame).await?;
            }
            Some(Err(e)) => {
                info!("Error decoding frame: {:?}", e);
                return Err(e);
            }
            None => {
                return Ok(());
            }
        }
    }
}

async fn request_handler(_request: RedisRequest) -> Result<RedisResponse> {
    let (frame, backend) = (_request.frame, _request.backend);
    let cmd = Command::try_from(frame)?;
    info!("Executing command: {:?}", cmd);
    let ret = cmd.execute(&backend);
    Ok(RedisResponse { frame: ret })
}
