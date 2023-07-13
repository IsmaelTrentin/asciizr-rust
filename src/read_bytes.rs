#[derive(Debug)]
pub enum ReadBytesError {
    NotEnoughBytes,
}

pub fn read_uint16_le(bytes: &[u8], offset: usize) -> Result<u16, ReadBytesError> {
    if bytes.len() < 2 {
        return Err(ReadBytesError::NotEnoughBytes);
    }

    let result = (bytes[offset + 1] as u16) << 8 | (bytes[offset] as u16);

    Ok(result)
}

pub fn read_uint32_le(bytes: &[u8], offset: usize) -> Result<u32, ReadBytesError> {
    if bytes.len() < 4 {
        return Err(ReadBytesError::NotEnoughBytes);
    }

    let result = (bytes[offset + 3] as u32) << 24
        | (bytes[offset + 2] as u32) << 16
        | (bytes[offset + 1] as u32) << 8
        | (bytes[offset] as u32);

    Ok(result)
}
