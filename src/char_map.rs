pub enum ParseCustomCharMapError {
    ParseError,
}

pub fn parse_custom_char_map(input: &str) -> Result<Vec<u8>, ParseCustomCharMapError> {
    // best case capacity
    let mut chars: Vec<u8> = Vec::with_capacity(input.len() / 2);
    let mut chars_i = 0;
    for (i, c) in input.chars().enumerate() {
        if i % 2 == 0 {
            let c_str = c.to_string();
            let c_bytes = c_str.as_bytes();
            if c_bytes.len() != 1 {
                return Err(ParseCustomCharMapError::ParseError);
            }
            chars.insert(chars_i, c_bytes[0]);
            chars_i += 1
        }
    }

    Ok(chars)
}
