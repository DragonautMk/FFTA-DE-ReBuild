//! Shared BE-size LZ/RLE codec used by original BG graphics and alternate palettes.
//! Callers supply independent allocation and input guards; no guest work is skipped.
#[derive(Debug, PartialEq, Eq)]
pub struct Block {
    pub bytes: Vec<u8>,
    /// Exact header/token bytes, excluding trailing alignment or other assets.
    pub consumed: usize,
    /// Literal payloads can be served from the decoded banks while preserving
    /// every original token/header byte and guest decompression instruction.
    pub literal_spans: Vec<LiteralSpan>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiteralSpan {
    pub encoded_offset: usize,
    pub output_offset: usize,
    pub length: usize,
}

/// Decode a BE-size custom stream, distinct from BIOS LZ77. Both allocation and
/// input consumption are bounded. An entire token must fit the declared output:
/// the original consumer can overrun that size, which is invalid offline data.
pub fn decode(input: &[u8], output_budget: usize, encoded_budget: usize) -> Result<Block, String> {
    let input = &input[..input.len().min(encoded_budget)];
    let header = input.get(..4).ok_or("Truncated custom LZ header")?;
    let size = u32::from_be_bytes(header.try_into().unwrap()) as usize;
    if size == 0 || size > output_budget {
        return Err("Invalid custom LZ output size/budget".into());
    }
    let mut bytes = Vec::with_capacity(size);
    let mut position = 4;
    let mut literal_spans = Vec::new();
    while bytes.len() < size {
        let token = take(input, &mut position)? as usize;
        let (length, distance, fill) = if token & 0x80 != 0 {
            (
                ((token >> 3) & 15) + 3,
                Some(((token & 7) << 8 | take(input, &mut position)? as usize) + 1),
                0,
            )
        } else if token & 0x40 != 0 {
            let length = (token & 63) + 1;
            fits(bytes.len(), length, size)?;
            let literal = input
                .get(position..position + length)
                .ok_or("Truncated custom LZ literal")?;
            literal_spans.push(LiteralSpan {
                encoded_offset: position,
                output_offset: bytes.len(),
                length,
            });
            bytes.extend_from_slice(literal);
            position += length;
            continue;
        } else if token & 0x20 != 0 {
            ((token & 31) + 2, None, 0)
        } else if token & 0x10 != 0 {
            let next = take(input, &mut position)? as usize;
            (
                ((token & 15) | ((next >> 2) & 48)) + 4,
                Some(((next & 63) << 8 | take(input, &mut position)? as usize) + 1),
                0,
            )
        } else {
            match token {
                0 => {
                    let length = take(input, &mut position)? as usize + 5;
                    let high = take(input, &mut position)? as usize;
                    (
                        length,
                        Some((high << 8 | take(input, &mut position)? as usize) + 1),
                        0,
                    )
                }
                1 | 2 => (
                    take(input, &mut position)? as usize + 3,
                    None,
                    if token == 1 { 255 } else { 0 },
                ),
                // Original dispatch consumes 03..0f without producing output.
                // They are not used in the verified base streams; preserve the
                // observed control behavior rather than inventing a token body.
                _ => continue,
            }
        };
        fits(bytes.len(), length, size)?;
        if let Some(distance) = distance {
            if distance > bytes.len() {
                return Err("Custom LZ reference precedes output history".into());
            }
            for _ in 0..length {
                bytes.push(bytes[bytes.len() - distance]);
            }
        } else {
            bytes.resize(bytes.len() + length, fill);
        }
    }
    Ok(Block {
        bytes,
        consumed: position,
        literal_spans,
    })
}

fn take(input: &[u8], position: &mut usize) -> Result<u8, String> {
    let byte = *input
        .get(*position)
        .ok_or("Truncated or overlong custom LZ stream")?;
    *position += 1;
    Ok(byte)
}

fn fits(written: usize, length: usize, size: usize) -> Result<(), String> {
    if length > size - written {
        Err("Custom LZ token exceeds declared output".into())
    } else {
        Ok(())
    }
}
