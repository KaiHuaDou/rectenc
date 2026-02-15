use rmp_serde;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RectEncError {
    #[error("Video Error: {0}")]
    Video(&'static str),
    #[error("IO Error: {0}")]
    IO(#[from] io::Error),
    #[error("OpenCV Error: {0}")]
    OpenCV(#[from] opencv::Error),
    #[error("MessagePack Encode Error: {0}")]
    RmpEncode(#[from] rmp_serde::encode::Error),
    #[error("MessagePack Decode Error: {0}")]
    RmpDecode(#[from] rmp_serde::decode::Error),
}
