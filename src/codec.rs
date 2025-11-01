use anyhow::Result;
use bincode::{Decode, Encode};

pub fn encode<T: Encode>(msg: &T) -> Result<Vec<u8>> {
    Ok(bincode::encode_to_vec(msg, bincode::config::standard())?)
}

pub fn decode<T: Decode<()>>(bytes: &[u8]) -> Result<T> {
    let (msg, _) = bincode::decode_from_slice(bytes, bincode::config::standard())?;
    Ok(msg)
}
