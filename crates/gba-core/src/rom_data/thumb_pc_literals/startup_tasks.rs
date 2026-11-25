//! European startup callback installation, dispatcher and title callback literals.

mod title_callbacks;

use super::{LiteralWord, SourceLayout};
use crate::ffta_rom;

const SLOT_COUNT: u32 = 12;
pub(super) const INSTALLATION_ROOT_COUNT: usize = SLOT_COUNT as usize + 2;
const INSTALLATION_ROOTS_SHA256: &str =
    "f9250dbfbd1c4994c192a32d7451c6e0aa37b44053f3f0808ceb1a1b3e205a1a";
const INSTALLER_SHA256: &str = "67bedcc75c48af9fe05aa21872aa996f6dec04f7f5b6e1f0f4b117c9589cffd0";
const DISPATCHER_ROOTS_SHA256: &str =
    "9ce5b5878698a02d487e06cfe9fae1f011d70d2315c6eda897ff46a85d383950";
const DISPATCHER_SHA256: &str = "8bb7dd983e6d293b079d0397d72ef8b153c240814ff81faa5e9638a9de79a64a";

// The unowned literal words of 080036c4 and its transition helpers, in ROM
// order. The other 19 loads are covered by the earlier literal groups.
const DISPATCHER_LOADS: [(u32, u32, u16); 39] = [
    (0x376c, 0x3712, 0x4816),
    (0x3770, 0x3714, 0x4c16),
    (0x3774, 0x371a, 0x4816),
    (0x3778, 0x371e, 0x4916),
    (0x377c, 0x3730, 0x4812),
    (0x37dc, 0x378a, 0x4814),
    (0x37e0, 0x378e, 0x4814),
    (0x37e4, 0x3792, 0x4914),
    (0x382c, 0x3812, 0x4806),
    (0x3878, 0x3850, 0x4a09),
    (0x387c, 0x3860, 0x4906),
    (0x38a0, 0x3892, 0x4803),
    (0x38a4, 0x3898, 0x4802),
    (0x391c, 0x38ee, 0x4e0b),
    (0x3920, 0x38f0, 0x4c0b),
    (0x3924, 0x38fc, 0x4909),
    (0x3928, 0x38fe, 0x4d0a),
    (0x3944, 0x3930, 0x4a04),
    (0x3948, 0x3936, 0x4a04),
    (0x3974, 0x3968, 0x4a02),
    (0x3978, 0x396c, 0x4902),
    (0x398c, 0x397c, 0x4903),
    (0x3990, 0x3982, 0x4903),
    (0x3994, 0x3984, 0x4a03),
    (0x39a8, 0x3998, 0x4903),
    (0x39ac, 0x399c, 0x4903),
    (0x39b0, 0x399e, 0x4a04),
    (0x39bc, 0x39b4, 0x4801),
    (0x39d0, 0x39c2, 0x4803),
    (0x3a14, 0x39e6, 0x490b),
    (0x3a18, 0x39ec, 0x480a),
    (0x3a1c, 0x39f6, 0x4809),
    (0x3a20, 0x39fa, 0x4909),
    (0x3a24, 0x3a04, 0x4807),
    (0x3a40, 0x3a28, 0x4905),
    (0x3a44, 0x3a2e, 0x4905),
    (0x3a48, 0x3a30, 0x4805),
    (0x3a64, 0x3a50, 0x4a04),
    (0x3a68, 0x3a54, 0x4a04),
];
pub(super) const DISPATCHER_ROOT_COUNT: usize = INSTALLATION_ROOT_COUNT + DISPATCHER_LOADS.len();
const FULL_ROOT_COUNT: usize = DISPATCHER_ROOT_COUNT + title_callbacks::LOADS.len();

fn loads() -> impl Iterator<Item = (u32, u32, u16)> {
    let roots = [(0x05c0, 0x058e, 0x4a0c), (0x3738, 0x36e0, 0x4915)];
    roots
        .into_iter()
        .chain((0..SLOT_COUNT).map(|slot| (0x373c + slot * 4, 0x36e2 + slot * 4, 0x4816)))
        .chain(DISPATCHER_LOADS)
        .chain(title_callbacks::LOADS)
}

pub(super) fn validate(words: &[LiteralWord]) -> Result<(), String> {
    if words.is_empty() {
        return Ok(());
    }
    if !matches!(
        words.len(),
        INSTALLATION_ROOT_COUNT | DISPATCHER_ROOT_COUNT | FULL_ROOT_COUNT
    ) || words
        .iter()
        .zip(loads())
        .any(|(word, (source, reader, _))| {
            word.source_offset != source || word.reader_offset != reader
        })
    {
        return Err("Invalid startup task root sources or readers".into());
    }
    let (installed, remaining) = words.split_at(INSTALLATION_ROOT_COUNT);
    let (dispatcher, title) = remaining.split_at(remaining.len().min(DISPATCHER_LOADS.len()));
    for (group, expected) in [
        (installed, INSTALLATION_ROOTS_SHA256),
        (dispatcher, DISPATCHER_ROOTS_SHA256),
        (title, title_callbacks::ROOTS_SHA256),
    ] {
        if group.is_empty() {
            continue;
        }
        let bytes: Vec<_> = group
            .iter()
            .flat_map(|word| word.value.to_le_bytes())
            .collect();
        if ffta_rom::digest(&bytes) != expected {
            return Err("Startup task root fingerprint mismatch".into());
        }
    }
    Ok(())
}

pub(super) fn extract(rom: &[u8], layout: SourceLayout) -> Result<Vec<LiteralWord>, String> {
    let count = match layout {
        SourceLayout::StartupTaskRoots => INSTALLATION_ROOT_COUNT,
        SourceLayout::StartupTaskDispatcher => DISPATCHER_ROOT_COUNT,
        SourceLayout::TitleTaskCallbacks | SourceLayout::TitleFrontend => FULL_ROOT_COUNT,
        _ => return Err("Invalid startup task layout".into()),
    };
    if ffta_rom::digest(&rom[0x36c4..0x3712]) != INSTALLER_SHA256 {
        return Err("Startup task installer fingerprint mismatch".into());
    }
    if count > INSTALLATION_ROOT_COUNT
        && ffta_rom::digest(&rom[0x3712..0x3a6c]) != DISPATCHER_SHA256
    {
        return Err("Startup dispatcher fingerprint mismatch".into());
    }
    if count == FULL_ROOT_COUNT {
        title_callbacks::verify_windows(rom)?;
    }
    let words = loads()
        .take(count)
        .map(|(source, reader, opcode)| LiteralWord::from_thumb_load(rom, source, reader, opcode))
        .collect::<Result<Vec<_>, String>>()?;
    validate(&words)?;
    Ok(words)
}
