//! Original fixed-source calls and the preserved OBJ property-word alias.

use super::{call, half, BankPointer, Graphic};
use crate::{ffta_rom, rom_data::unit_obj_ffff_reader_fields};

// Each pair is the literal load and the original call of the custom decoder.
// The four literal cells already owned in e312 are left with their owners.
const ROOTS: &[(u32, u32)] = &[
    (0x023c00, 0x023c04),
    (0x02a66a, 0x02a66c),
    (0x03068c, 0x030690),
    (0x032896, 0x03289a),
    (0x0340e4, 0x0340e8),
    (0x03e604, 0x03e608),
    (0x04fc1e, 0x04fc22),
    (0x04fd00, 0x04fd04),
    (0x04fd74, 0x04fd76),
    (0x05e750, 0x05e754),
    (0x089f28, 0x089f2c),
    (0x0954d4, 0x0954fe),
    (0x12edfa, 0x12edfe),
    (0x12ee28, 0x12ee2c),
    (0x12f2d2, 0x12f2d4),
    (0x13bfa2, 0x13bfa6),
    (0x13fe7e, 0x13fe82),
    (0x143c92, 0x143c96),
    (0x143dfe, 0x143e02),
    (0x143e32, 0x143e36),
    (0x143ef0, 0x143ef4),
    (0x143f08, 0x143f0c),
    (0x143fbe, 0x143fc2),
    (0x144072, 0x144076),
    (0x144126, 0x14412a),
    (0x144408, 0x14440c),
];
const R7_READER_SHA256: &str = "d2885952a3973112f75782f503d3adfc6000192590bea5cec27a303d851c4b4b";
const BANK_WITH_PROPERTY_ALIAS: u32 = 0x3b1108;
const PROPERTY_START: u32 = unit_obj_ffff_reader_fields::PROPERTY_OFFSET;
const PROPERTY_END: u32 = PROPERTY_START + 2;

pub(super) fn pointers(rom: &[u8]) -> Result<Vec<BankPointer>, String> {
    ROOTS
        .iter()
        .map(|&(reader, site)| {
            let load = half(rom, reader)?;
            let literal = ((reader + 4) & !3) + 4 * u32::from(load & 255);
            // Most callers load r1 directly. One keeps the address in r7
            // across heap setup and moves it to r1 just before the decoder.
            let valid = if site == 0x0954fe {
                ffta_rom::digest(&rom[reader as usize..site as usize + 4]) == R7_READER_SHA256
            } else {
                let last = half(rom, site - 2)?;
                load & 0xff00 == 0x4900
                    && (site == reader + 2
                        || site == reader + 4
                            && (last & 0xffc7 == 0x1c00 || last & 0xff87 == 0x4600))
            };
            if !valid {
                return Err("Custom graphics literal argument reader mismatch".into());
            }
            call(rom, site, 0x54a0)?;
            let at = literal as usize;
            let target = u32::from_le_bytes(
                rom.get(at..at + 4)
                    .ok_or("Truncated custom graphics source literal")?
                    .try_into()
                    .unwrap(),
            );
            Ok(BankPointer {
                source_offset: literal,
                bank_offset: target
                    .checked_sub(0x08000000)
                    .ok_or("Invalid custom graphics source literal")?,
            })
        })
        .collect()
}

pub(in crate::rom_data) fn bank_parts(
    bank: &Graphic,
) -> Result<impl Iterator<Item = (u32, &[u8])>, String> {
    let encoded = bank.encoded_source.as_slice();
    let parts = if bank.source_offset == BANK_WITH_PROPERTY_ALIAS {
        let index = (PROPERTY_START - bank.source_offset) as usize;
        let before = encoded
            .get(..index)
            .ok_or("Custom graphics property alias before stream end")?;
        let after = encoded
            .get(index + 2..)
            .filter(|bytes| !bytes.is_empty())
            .ok_or("Custom graphics property alias after stream end")?;
        [
            Some((bank.source_offset, before)),
            Some((PROPERTY_END, after)),
        ]
    } else {
        [Some((bank.source_offset, encoded)), None]
    };
    Ok(parts.into_iter().flatten())
}
