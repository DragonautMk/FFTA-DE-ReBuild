//! ROM -> .lex translator integrated into the installer.
//!
//! Entry point: `translate()`. It currently returns an empty, valid wasm module
//! as a placeholder. The recompiler library (the `recomp` crate) will be called
//! here to emit the actual wasm.

use sha1::{Digest, Sha1};

pub const MAGIC: &[u8; 4] = b"LEX1";
pub const FORMAT_VERSION: u16 = 1;

pub struct LexHeader {
    pub region: [u8; 4],
    pub rom_sha1: [u8; 20],
}

/// Translates the ROM into a WebAssembly module.
/// Calls `progress(fraction 0..1, text)` to update the UI.
pub fn translate(rom: &[u8], progress: &mut dyn FnMut(f32, &str)) -> Result<Vec<u8>, String> {
    progress(0.15, "Analyzing ROM...");
    if rom.len() < 0xC0 {
        return Err("ROM is too small.".into());
    }

    // TODO: replace with the actual recompiler call, for example:
    //   let wasm = recomp::translate_to_wasm(rom, |p, s| progress(0.15 + 0.7 * p, s))?;
    progress(0.5, "Translating code...");
    let wasm = placeholder_wasm();

    progress(0.85, "Packaging...");
    Ok(wasm)
}

/// Minimal valid wasm module: magic "\0asm" + version 1, with no sections.
fn placeholder_wasm() -> Vec<u8> {
    vec![0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]
}

/// .lex format:
///   "LEX1" | version u16 LE | region [4] | ROM sha1 [20] | len u64 LE | wasm payload
/// The payload never includes the ROM, only the translated code.
pub fn build_lex(rom: &[u8], payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 40);
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
    out.extend_from_slice(&rom[0xAC..0xB0]);
    out.extend_from_slice(&Sha1::digest(rom));
    out.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    out.extend_from_slice(payload);
    out
}

/// Reads for the app (exe/apk): validates the header and returns the wasm.
#[allow(dead_code)]
pub fn read_lex(bytes: &[u8]) -> Result<(LexHeader, &[u8]), String> {
    if bytes.len() < 38 || &bytes[0..4] != MAGIC {
        return Err("Invalid .lex file.".into());
    }
    let ver = u16::from_le_bytes([bytes[4], bytes[5]]);
    if ver != FORMAT_VERSION {
        return Err(format!("Unsupported .lex version: {ver}"));
    }
    let mut region = [0u8; 4];
    region.copy_from_slice(&bytes[6..10]);
    let mut rom_sha1 = [0u8; 20];
    rom_sha1.copy_from_slice(&bytes[10..30]);
    let len = u64::from_le_bytes(bytes[30..38].try_into().unwrap()) as usize;
    let payload = bytes.get(38..38 + len).ok_or("Truncated .lex file.")?;
    Ok((LexHeader { region, rom_sha1 }, payload))
}
