use bytes::Bytes;
use serde::{de::Error, Deserialize, Serialize};

#[derive(Debug, Clone)]
pub enum Message {
    Text(String),
    Binary(Bytes),
    Ping(Bytes),
    Pong(Bytes),
    Close(Option<CloseFrame>),
}

#[derive(Debug, Clone)]
pub struct CloseFrame {
    pub code: u16,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Text,
    Binary,
    Ping,
    Pong,
    Close,
}

impl Message {
    pub fn text(text: impl Into<String>) -> Self {
        Message::Text(text.into())
    }

    pub fn binary(data: impl Into<Bytes>) -> Self {
        Message::Binary(data.into())
    }

    pub fn ping(data: impl Into<Bytes>) -> Self {
        Message::Ping(data.into())
    }

    pub fn pong(data: impl Into<Bytes>) -> Self {
        Message::Pong(data.into())
    }

    pub fn close() -> Self {
        Message::Close(None)
    }

    pub fn close_with_reason(code: u16, reason: impl Into<String>) -> Self {
        Message::Close(Some(CloseFrame {
            code,
            reason: reason.into(),
        }))
    }

    pub fn message_type(&self) -> MessageType {
        match self {
            Message::Text(_) => MessageType::Text,
            Message::Binary(_) => MessageType::Binary,
            Message::Ping(_) => MessageType::Ping,
            Message::Pong(_) => MessageType::Pong,
            Message::Close(_) => MessageType::Close,
        }
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Message::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn as_binary(&self) -> Option<&Bytes> {
        match self {
            Message::Binary(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub fn into_text(self) -> Option<String> {
        match self {
            Message::Text(text) => Some(text),
            _ => None,
        }
    }

    pub fn into_binary(self) -> Option<Bytes> {
        match self {
            Message::Binary(bytes) => Some(bytes),
            _ => None,
        }
    }

    pub fn is_text(&self) -> bool {
        matches!(self, Message::Text(_))
    }

    pub fn is_binary(&self) -> bool {
        matches!(self, Message::Binary(_))
    }

    pub fn is_close(&self) -> bool {
        matches!(self, Message::Close(_))
    }

    pub fn json<T: Serialize>(value: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_string(value)?;
        Ok(Message::Text(json))
    }

    pub fn parse_json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        match self {
            Message::Text(text) => serde_json::from_str(text),
            _ => Err(serde_json::Error::custom("Message is not text")),
        }
    }
}

impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Message::Text(text) => write!(f, "Text: {}", text),
            Message::Binary(bytes) => write!(f, "Binary: {} bytes", bytes.len()),
            Message::Ping(bytes) => write!(f, "Ping: {} bytes", bytes.len()),
            Message::Pong(bytes) => write!(f, "Pong: {} bytes", bytes.len()),
            Message::Close(frame) => match frame {
                Some(frame) => write!(f, "Close: {} - {}", frame.code, frame.reason),
                None => write!(f, "Close"),
            },
        }
    }
}

impl From<String> for Message {
    fn from(text: String) -> Self {
        Message::Text(text)
    }
}

impl From<&str> for Message {
    fn from(text: &str) -> Self {
        Message::Text(text.to_string())
    }
}

impl From<Bytes> for Message {
    fn from(bytes: Bytes) -> Self {
        Message::Binary(bytes)
    }
}

impl From<Vec<u8>> for Message {
    fn from(bytes: Vec<u8>) -> Self {
        Message::Binary(Bytes::from(bytes))
    }
}
