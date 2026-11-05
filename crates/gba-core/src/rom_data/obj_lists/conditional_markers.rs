//! Two conditional OBJ lists passed through a live RAM slot to the original draw.
//! This source model does not publish the lists or constrain guest selection.

use super::{List, Lists};
use crate::{ffta_rom, rom_data::SourceRange};
use serde::{Deserialize, Serialize};

pub const ROOTS: [(u32, &str); 2] = [
    (
        0x395172,
        "a9463596dde9c444781f52a46494aaca5029b82ca738575b50ef58ce776ce68d",
    ),
    (
        0x39517a,
        "2ef110040f298c4249ef44034012bf4768c7d710e91345f4506b851b6353b18f",
    ),
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConditionalMarkers {
    pub lists: Vec<List>,
}

impl ConditionalMarkers {
    pub fn validate(&self) -> Result<(), String> {
        if self.lists.len() != ROOTS.len() {
            return Err("Conditional OBJ markers require two lists".into());
        }
        for (list, (offset, _)) in self.lists.iter().zip(ROOTS) {
            list.validate()?;
            if list.source_offset != offset || list.objects.len() != 1 || list.objects[0].shape >= 3
            {
                return Err("Conditional OBJ marker identity, count or shape mismatch".into());
            }
        }
        Lists {
            lists: self.lists.clone(),
        }
        .validate()
    }

    /// The original writer tests only the low byte of its second argument.
    pub fn select(&self, selector: u32) -> Option<&List> {
        let index = usize::from(selector as u8 == 0);
        self.lists.get(index)
    }

    pub fn byte_at(&self, offset: u32) -> Option<u8> {
        self.lists
            .iter()
            .find_map(|list| list.source_byte(offset.checked_sub(list.source_offset)? as usize))
    }

    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, &'static str)>, String> {
        self.validate()?;
        self.lists
            .iter()
            .map(|list| Ok((list.source_offset, list.source_bytes()?, "static-obj-list")))
            .collect()
    }

    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        self.validate()?;
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Conditional OBJ markers differ from authenticated EUR source".into());
        }
        Ok(())
    }
}

pub struct Catalog {
    pub decoded: ConditionalMarkers,
    pub sources: Vec<SourceRange>,
}

fn half(rom: &[u8], offset: usize) -> Result<u16, String> {
    let bytes = rom
        .get(offset..offset + 2)
        .ok_or("Truncated OBJ marker instruction")?;
    Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
}

fn word(rom: &[u8], offset: usize) -> Result<u32, String> {
    let bytes = rom
        .get(offset..offset + 4)
        .ok_or("Truncated OBJ marker literal")?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

fn check_call(rom: &[u8], pc: usize, target: usize) -> Result<(), String> {
    let hi = half(rom, pc)?;
    let lo = half(rom, pc + 2)?;
    let displacement =
        ((((u32::from(hi & 2047)) << 12) | (u32::from(lo & 2047) << 1)) as i32) << 9 >> 9;
    if hi & 0xf800 != 0xf000
        || lo & 0xf800 != 0xf800
        || (pc as u32)
            .wrapping_add(4)
            .wrapping_add_signed(displacement)
            != target as u32
    {
        return Err("Conditional OBJ marker branch mismatch".into());
    }
    Ok(())
}

impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Conditional OBJ markers require the exact EUR ROM".into());
        }
        // The writer stores one of two literals at context + 47c + 8*index + 4;
        // the consumer loads that slot and forwards it to the original draw.
        for (pc, value) in [
            (0x2f374, 0x0609),
            (0x2f376, 0x4b05),
            (0x2f378, 0x00e0),
            (0x2f37a, 0x4a05),
            (0x2f37c, 0x1880),
            (0x2f37e, 0x681a),
            (0x2f380, 0x1812),
            (0x2f382, 0x2900),
            (0x2f384, 0xd008),
            (0x2f386, 0x4803),
            (0x2f388, 0xe007),
            (0x2f398, 0x4808),
            (0x2f39a, 0x6050),
            (0x2f3ee, 0x4641),
            (0x2f3f0, 0x6808),
            (0x2f3f2, 0x1980),
            (0x2f3f4, 0x6843),
            (0x2f3f6, 0x2b00),
            (0x2f410, 0x1c18),
            (0x2f792, 0x2101),
            (0x30228, 0x2100),
            (0x30380, 0x2101),
        ] {
            if half(rom, pc)? != value {
                return Err("Conditional OBJ marker source instruction mismatch".into());
            }
        }
        for (offset, value) in [
            (0x2f38c, 0x0200f444),
            (0x2f390, 0x47c),
            (0x2f394, 0x08395172),
            (0x2f3bc, 0x0839517a),
            (0x2f434, 0x0200f444),
            (0x2f440, 0x47c),
        ] {
            if word(rom, offset)? != value {
                return Err("Conditional OBJ marker pointer mismatch".into());
            }
        }
        for (pc, target) in [
            (0x2f794, 0x2f370),
            (0x3022a, 0x2f370),
            (0x30382, 0x2f370),
            (0x2f414, 0x1d28),
            (0x1db8, 0x1500),
        ] {
            check_call(rom, pc, target)?;
        }
        let mut lists = Vec::new();
        let mut sources = Vec::new();
        for (offset, expected) in ROOTS {
            let bytes = rom
                .get(offset as usize..offset as usize + 8)
                .ok_or("Truncated conditional OBJ marker")?;
            if ffta_rom::digest(bytes) != expected {
                return Err("Conditional OBJ marker source identity mismatch".into());
            }
            lists.push(List::decode(offset, bytes)?);
            sources.push(SourceRange {
                offset,
                length: 8,
                sha256: expected.into(),
                role: "static-obj-list".into(),
            });
        }
        let decoded = ConditionalMarkers { lists };
        decoded.validate()?;
        Ok(Self { decoded, sources })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Bus, Machine};

    fn sample() -> ConditionalMarkers {
        ConditionalMarkers {
            lists: ROOTS
                .into_iter()
                .map(|(offset, _)| List::decode(offset, &[1, 0, 0, 0, 0, 0, 0, 0]).unwrap())
                .collect(),
        }
    }

    #[test]
    fn selection_boundaries_and_shape_guards() {
        let data = sample();
        data.validate().unwrap();
        for (selector, offset) in [
            (0, ROOTS[1].0),
            (1, ROOTS[0].0),
            (255, ROOTS[0].0),
            (256, ROOTS[1].0),
            (257, ROOTS[0].0),
        ] {
            assert_eq!(data.select(selector).unwrap().source_offset, offset);
        }
        assert_eq!(data.byte_at(ROOTS[0].0 - 1), None);
        assert_eq!(data.byte_at(ROOTS[0].0 + 8), Some(1));
        assert_eq!(data.byte_at(ROOTS[1].0 + 8), None);
        let mut invalid = data.clone();
        invalid.lists.swap(0, 1);
        assert!(invalid.validate().is_err());
        invalid = data.clone();
        invalid.lists[0].objects[0].shape = 3;
        assert!(invalid.validate().is_err());
        invalid = data.clone();
        invalid.lists[1].objects.clear();
        assert!(invalid.validate().is_err());
        assert!(data.verify_source(&[]).is_err());
    }

    #[test]
    #[ignore = "requires the private exact EUR ROM"]
    fn exact_eur_sources_and_roundtrip() {
        let rom = std::fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
        let catalog = Catalog::from_rom(&rom).unwrap();
        let data = &catalog.decoded;
        data.verify_source(&rom).unwrap();
        for (list, source) in data.lists.iter().zip(&catalog.sources) {
            assert_eq!(
                list.source_bytes().unwrap(),
                rom[source.offset as usize..source.offset as usize + 8]
            );
        }
        let bytes = serde_json::to_vec(data).unwrap();
        assert_eq!(
            serde_json::from_slice::<ConditionalMarkers>(&bytes).unwrap(),
            *data
        );
        let mut changed = data.clone();
        changed.lists[0].objects[0].tile ^= 1;
        changed.validate().unwrap();
        assert!(changed.verify_source(&rom).is_err());

        // Execute the original writer on an isolated machine. It writes a
        // pointer and coordinates to a RAM slot; no gameplay save is loaded.
        const CONTEXT: u32 = 0x0202_0000;
        const SLOT: u32 = CONTEXT + 0x47c + 8;
        const STACK: u32 = 0x0300_7c00;
        const RETURN: u32 = 0x0800_0300;
        for selector in [0, 1, 255, 256, 257, u32::MAX] {
            let mut machine = Machine::new(rom.clone());
            machine.bus.write32(0x0200_f444, CONTEXT);
            machine.bus.write16(CONTEXT, 31);
            machine.bus.write16(CONTEXT + 2, 10);
            let mut expected = machine.bus.ewram.clone();
            let slot = (SLOT - 0x0200_0000) as usize;
            expected[slot..slot + 2].copy_from_slice(&242u16.to_le_bytes());
            expected[slot + 2..slot + 4].copy_from_slice(&112u16.to_le_bytes());
            let pointer = 0x0800_0000 + data.select(selector).unwrap().source_offset;
            expected[slot + 4..slot + 8].copy_from_slice(&pointer.to_le_bytes());
            let mut iwram = machine.bus.iwram.clone();
            machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            machine.cpu.regs[0] = 1;
            machine.cpu.regs[1] = selector;
            machine.cpu.regs[13] = STACK;
            machine.cpu.regs[14] = RETURN | 1;
            machine.cpu.regs[15] = 0x0802_f370;
            machine.cpu.cpsr = 0xbf;
            let callee = machine.cpu.regs[4..12].to_vec();
            let mut returned = false;
            for _ in 0..200 {
                machine.bus.frame_ready = false;
                machine.step();
                if machine.cpu.regs[15] == RETURN {
                    returned = true;
                    break;
                }
            }
            assert!(returned, "original marker writer did not return");
            assert_eq!(machine.cpu.regs[13], STACK);
            assert_eq!(&machine.cpu.regs[4..12], callee.as_slice());
            assert_eq!(machine.bus.ewram, expected, "writer EWRAM footprint");
            // The original prologue uses this stack window; all other IWRAM
            // must remain unchanged.
            iwram[0x7b00..0x7c00].fill(0);
            machine.bus.iwram[0x7b00..0x7c00].fill(0);
            assert_eq!(machine.bus.iwram, iwram, "writer IWRAM footprint");
        }
    }

    #[test]
    #[ignore = "requires the private exact EUR ROM"]
    fn original_slot_consumer_draws_the_selected_markers() {
        let rom = std::fs::read(std::env::var_os("FFTA_STATE_TEST_ROM").unwrap()).unwrap();
        let catalog = Catalog::from_rom(&rom).unwrap();
        let dimensions = crate::rom_data::ObjDimensions::decode(&rom[0x14ced8..0x14cf08])
            .expect("original object dimensions");
        const CONTEXT: u32 = 0x0202_0000;
        const STACK: u32 = 0x0300_7c00;
        const RETURN: u32 = 0x0800_0300;
        const CURSOR: usize = 0x2c50;
        const BUFFER: usize = 0x2c58;
        // Exercise both source lists, a clipped item, and the three original
        // guards, without fabricating an ordinary game state.
        for (selector, x, y, enabled, count, pointer_present) in [
            (0, 100i16, 80i16, true, 1, true),
            (1, 100, 80, true, 1, true),
            (1, -1024, 80, true, 1, true),
            (0, 100, 80, false, 1, true),
            (1, 100, 80, true, 0, true),
            (0, 100, 80, true, 1, false),
        ] {
            let list = catalog.decoded.select(selector).unwrap();
            let mut machine = Machine::new(rom.clone());
            machine.bus.write32(0x0200_f444, CONTEXT);
            machine.bus.write8(CONTEXT + 0xe, u8::from(enabled));
            machine.bus.write8(0x0200_3c32, count);
            machine.bus.write16(CONTEXT + 0x47c, x as u16);
            machine.bus.write16(CONTEXT + 0x47e, y as u16);
            machine.bus.write32(
                CONTEXT + 0x480,
                if pointer_present {
                    0x0800_0000 + list.source_offset
                } else {
                    0
                },
            );
            machine.bus.iwram[BUFFER..BUFFER + 0x180].fill(0x5a);
            let ewram = machine.bus.ewram.clone();
            let mut expected_iwram = machine.bus.iwram.clone();
            let mut drawn = false;
            if enabled && count != 0 && pointer_present {
                let object = &list.objects[0];
                let dim = dimensions.get(object.shape, object.size).unwrap();
                let x = i32::from(x) + i32::from(object.x);
                let y = i32::from(y) + i32::from(object.y);
                if !(x < 0 && x + i32::from(dim.width) < 0
                    || x > 239 && x + i32::from(dim.width) > 239
                    || y < 0 && y + i32::from(dim.height) < 0
                    || y > 159 && y + i32::from(dim.height) > 159)
                {
                    let [a0, a1, a2] = object.attributes();
                    let attributes = [
                        (a0 & 0xff00) | (y as u16 & 255),
                        (a1 & !511) | (x as u16 & 511),
                        a2,
                    ];
                    for (index, value) in attributes.into_iter().enumerate() {
                        let offset = BUFFER + index * 2;
                        expected_iwram[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
                    }
                    expected_iwram[CURSOR..CURSOR + 4].copy_from_slice(&1u32.to_le_bytes());
                    drawn = true;
                }
            }
            let video = (
                machine.bus.vram.clone(),
                machine.bus.oam.clone(),
                machine.bus.palette.clone(),
            );
            machine.cpu.regs = std::array::from_fn(|i| 0x1100_0000 + i as u32);
            machine.cpu.regs[13] = STACK;
            machine.cpu.regs[14] = RETURN | 1;
            machine.cpu.regs[15] = 0x0802_f3c0;
            machine.cpu.cpsr = 0xbf;
            let callee = machine.cpu.regs[4..12].to_vec();
            let mut returned = false;
            for _ in 0..20_000 {
                machine.bus.frame_ready = false;
                machine.step();
                if machine.cpu.regs[15] == RETURN {
                    returned = true;
                    break;
                }
            }
            assert!(returned, "original slot consumer did not return");
            assert_eq!(machine.cpu.regs[13], STACK);
            assert_eq!(&machine.cpu.regs[4..12], callee.as_slice());
            assert_eq!(machine.bus.ewram, ewram, "consumer EWRAM footprint");
            expected_iwram[0x7a00..0x7c00].fill(0);
            machine.bus.iwram[0x7a00..0x7c00].fill(0);
            assert_eq!(machine.bus.iwram, expected_iwram, "consumer IWRAM/output");
            assert_eq!(
                (machine.bus.vram, machine.bus.oam, machine.bus.palette),
                video
            );
            assert_eq!(drawn, enabled && count != 0 && pointer_present && x >= 0);
        }
    }
}
