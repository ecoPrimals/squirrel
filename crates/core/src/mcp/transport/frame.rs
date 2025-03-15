use bytes::{BytesMut, BufMut, Buf};
use tokio::io::{AsyncRead, AsyncWrite};
use serde::{Serialize, Deserialize};
use crate::mcp::error::{MCPError, Result};
use crate::mcp::types::MCPMessage;

const FRAME_HEADER_SIZE: usize = 8; // 4 bytes for magic + 4 bytes for payload length
const FRAME_MAGIC: u32 = 0x4D435000; // "MCP\0"

#[derive(Debug)]
pub struct Frame {
    pub payload: BytesMut,
}

impl Frame {
    pub fn new(payload: BytesMut) -> Self {
        Self { payload }
    }

    pub fn check_frame(buf: &[u8]) -> Option<usize> {
        if buf.len() < FRAME_HEADER_SIZE {
            return None;
        }

        let mut header = &buf[..FRAME_HEADER_SIZE];
        let magic = header.get_u32();
        if magic != FRAME_MAGIC {
            return None;
        }

        let payload_len = header.get_u32() as usize;
        let frame_len = FRAME_HEADER_SIZE + payload_len;

        if buf.len() < frame_len {
            return None;
        }

        Some(frame_len)
    }

    pub fn parse(buf: &mut BytesMut) -> Result<Option<Frame>> {
        if let Some(frame_len) = Frame::check_frame(buf) {
            let frame_data = buf.split_to(frame_len);
            let mut payload = frame_data.freeze().slice(FRAME_HEADER_SIZE..);
            Ok(Some(Frame::new(BytesMut::from(&payload[..]))))
        } else {
            Ok(None)
        }
    }

    pub fn serialize(&self) -> BytesMut {
        let mut buf = BytesMut::with_capacity(FRAME_HEADER_SIZE + self.payload.len());
        buf.put_u32(FRAME_MAGIC);
        buf.put_u32(self.payload.len() as u32);
        buf.extend_from_slice(&self.payload);
        buf
    }
}

pub struct MessageCodec;

impl MessageCodec {
    pub fn new() -> Self {
        Self
    }

    pub async fn encode_message(&self, message: &MCPMessage) -> Result<Frame> {
        let payload = serde_json::to_vec(message)
            .map_err(|e| MCPError::SerdeJson(e))?;
        Ok(Frame::new(BytesMut::from(&payload[..])))
    }

    pub async fn decode_message(&self, frame: Frame) -> Result<MCPMessage> {
        serde_json::from_slice(&frame.payload)
            .map_err(|e| MCPError::SerdeJson(e))
    }
}

pub struct FrameReader<R> {
    reader: R,
    buffer: BytesMut,
}

impl<R: AsyncRead + Unpin> FrameReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: BytesMut::with_capacity(8 * 1024),
        }
    }

    pub async fn read_frame(&mut self) -> Result<Option<Frame>> {
        use tokio::io::AsyncReadExt;

        loop {
            if let Some(frame) = Frame::parse(&mut self.buffer)? {
                return Ok(Some(frame));
            }

            if 0 == self.reader.read_buf(&mut self.buffer).await? {
                if self.buffer.is_empty() {
                    return Ok(None);
                } else {
                    return Err(MCPError::InvalidMessage("connection reset by peer".into()));
                }
            }
        }
    }
}

pub struct FrameWriter<W> {
    writer: W,
}

impl<W: AsyncWrite + Unpin> FrameWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer }
    }

    pub async fn write_frame(&mut self, frame: Frame) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        
        let buf = frame.serialize();
        self.writer.write_all(&buf).await?;
        self.writer.flush().await?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use crate::mcp::types::{MessageType, ProtocolVersion, SecurityLevel, MessageMetadata};

    #[tokio::test]
    async fn test_message_codec() {
        let codec = MessageCodec::new();
        
        let message = MCPMessage::new(
            MessageType::Command,
            ProtocolVersion::new(1, 0),
            SecurityLevel::None,
            vec![1, 2, 3],
        );

        let frame = codec.encode_message(&message).await.unwrap();
        let decoded = codec.decode_message(frame).await.unwrap();

        assert_eq!(decoded.message_type, message.message_type);
        assert_eq!(decoded.protocol_version, message.protocol_version);
        assert_eq!(decoded.payload, message.payload);
    }

    #[test]
    fn test_frame_serialization() {
        let payload = BytesMut::from(&b"test payload"[..]);
        let frame = Frame::new(payload.clone());
        
        let serialized = frame.serialize();
        let mut parsed_frame = Frame::parse(&mut serialized.clone()).unwrap().unwrap();
        
        assert_eq!(&parsed_frame.payload[..], &payload[..]);
    }
} 