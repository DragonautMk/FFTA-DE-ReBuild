//! Readonly, explicitly assumed plaintext CodeBreaker inspection. Never executes.
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum Operation {
    Write {
        address: u32,
        width: u8,
        value: u16,
    },
    Modify {
        address: u32,
        width: u8,
        operation: &'static str,
        value: u16,
    },
    Slide {
        address: u32,
        width: u8,
        value: u16,
        count: u16,
        address_increment: u16,
        value_increment: u16,
    },
    Conditional {
        address: u32,
        width: u8,
        comparison: &'static str,
        value: u16,
        next: Box<Operation>,
    },
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct Plan {
    format: &'static str,
    assumption: &'static str,
    endianness: &'static str,
    executable: bool,
    operations: Vec<Operation>,
}

/// A bounded static footprint. Conditions are represented as reads; writes
/// include both branches' potential effects, even when a condition is false.
#[derive(Debug, serde::Serialize)]
pub(crate) struct Footprint {
    writes: Vec<(u32, u32)>,
    reads: Vec<(u32, u32)>,
    max_writes: usize,
}

impl Footprint {
    pub(crate) fn max_writes(&self) -> usize { self.max_writes }

    /// Reject write/write and write/read overlap, including conditional reads.
    pub(crate) fn conflicts(&self, other: &Self) -> bool {
        fn overlaps(a: &[(u32, u32)], b: &[(u32, u32)]) -> bool {
            a.iter().any(|&(start, end)| b.iter().any(|&(other_start, other_end)| start <= other_end && other_start <= end))
        }
        overlaps(&self.writes, &other.writes)
            || overlaps(&self.writes, &other.reads)
            || overlaps(&self.reads, &other.writes)
    }
}

fn ram_range(address: u32, width: u8) -> bool {
    let Some(end) = address.checked_add(u32::from(width).saturating_sub(1)) else {
        return false;
    };
    matches!(width, 1 | 2)
        && address % u32::from(width) == 0
        && ((0x02000000..=0x0203ffff).contains(&address) && end <= 0x0203ffff
            || (0x03000000..=0x03007fff).contains(&address) && end <= 0x03007fff)
}

fn range_list(bytes: std::collections::BTreeSet<u32>) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::new();
    for byte in bytes {
        if let Some(last) = out.last_mut() {
            if last.1.checked_add(1) == Some(byte) {
                last.1 = byte;
                continue;
            }
        }
        out.push((byte, byte));
    }
    out
}

impl Plan {
    /// No bus access happens until every potential operation has passed.
    pub(crate) fn footprint(&self) -> Result<Footprint, String> {
        use std::collections::BTreeSet;
        let (mut reads, mut writes, mut count) = (BTreeSet::new(), BTreeSet::new(), 0usize);
        fn checked(
            op: &Operation,
            reads: &mut BTreeSet<u32>,
            writes: &mut BTreeSet<u32>,
            count: &mut usize,
            conditional: bool,
        ) -> Result<(), String> {
            let mut write = |address: u32, width: u8| -> Result<(), String> {
                if !ram_range(address, width) {
                    return Err("Only aligned canonical EWRAM/IWRAM writes are supported".into());
                }
                *count += 1;
                if *count > 256 {
                    return Err("One-shot plan exceeds 256 writes".into());
                }
                for byte in address..address + u32::from(width) {
                    writes.insert(byte);
                }
                Ok(())
            };
            match op {
                Operation::Write {
                    address,
                    width,
                    value,
                } => {
                    if *width == 1 && *value > 0xff {
                        return Err("Byte value exceeds 8 bits".into());
                    }
                    write(*address, *width)?;
                }
                Operation::Modify {
                    address,
                    width,
                    operation,
                    ..
                } => {
                    if *width != 2 || !matches!(*operation, "or" | "and" | "add") {
                        return Err("Unsupported modification".into());
                    }
                    write(*address, *width)?;
                    for b in *address..*address + 2 {
                        reads.insert(b);
                    }
                }
                Operation::Slide {
                    address,
                    width,
                    count: repeat,
                    address_increment,
                    ..
                } => {
                    if *width != 2 || *repeat == 0 || *repeat > 256 || address_increment % 2 != 0 {
                        return Err("Unsupported slide".into());
                    }
                    for i in 0..*repeat {
                        let a = address
                            .checked_add(u32::from(i) * u32::from(*address_increment))
                            .ok_or("Slide overflow")?;
                        write(a, *width)?;
                    }
                }
                Operation::Conditional {
                    address,
                    width,
                    comparison,
                    next,
                    ..
                } => {
                    if conditional
                        || *width != 2
                        || !matches!(*comparison, "equal" | "not-equal")
                        || !(ram_range(*address, 2) || *address == 0x04000130)
                    {
                        return Err("Unsupported condition read".into());
                    }
                    for b in *address..*address + 2 {
                        reads.insert(b);
                    }
                    checked(next, reads, writes, count, true)?;
                }
            }
            Ok(())
        }
        for op in &self.operations {
            checked(op, &mut reads, &mut writes, &mut count, false)?;
        }
        if count == 0 {
            return Err("One-shot plan has no writes".into());
        }
        Ok(Footprint {
            writes: range_list(writes),
            reads: range_list(reads),
            max_writes: count,
        })
    }

    /// Apply only after a complete footprint check; no fallible operation is
    /// performed after the first bus write. Caller owns pause/session guards.
    pub(crate) fn apply_once<B: gba_core::Bus>(&self, bus: &mut B) -> Result<usize, String> {
        self.footprint()?;
        fn run<B: gba_core::Bus>(op: &Operation, bus: &mut B, count: &mut usize) {
            match op {
                Operation::Write {
                    address,
                    width,
                    value,
                } => {
                    if *width == 1 {
                        bus.write8(*address, *value as u8)
                    } else {
                        bus.write16(*address, *value)
                    };
                    *count += 1;
                }
                Operation::Modify {
                    address,
                    operation,
                    value,
                    ..
                } => {
                    let current = bus.read16(*address);
                    let next = match *operation {
                        "or" => current | value,
                        "and" => current & value,
                        "add" => current.wrapping_add(*value),
                        _ => unreachable!(),
                    };
                    bus.write16(*address, next);
                    *count += 1;
                }
                Operation::Slide {
                    address,
                    value,
                    count: repeat,
                    address_increment,
                    value_increment,
                    ..
                } => {
                    for i in 0..*repeat {
                        bus.write16(
                            *address + u32::from(i) * u32::from(*address_increment),
                            value.wrapping_add(i.wrapping_mul(*value_increment)),
                        );
                        *count += 1;
                    }
                }
                Operation::Conditional {
                    address,
                    comparison,
                    value,
                    next,
                    ..
                } => {
                    let current = bus.read16(*address);
                    let matched = if *comparison == "equal" {
                        current == *value
                    } else {
                        current != *value
                    };
                    if matched {
                        run(next, bus, count);
                    }
                }
            }
        }
        let mut count = 0;
        for op in &self.operations {
            run(op, bus, &mut count);
        }
        Ok(count)
    }
}

fn line(value: &str, index: usize) -> Result<(u32, u16), String> {
    let mut fields = value.split_ascii_whitespace();
    let (a, b) = (fields.next(), fields.next());
    let (Some(a), Some(b)) = (a, b) else {
        return Err(format!("Invalid CodeBreaker line {}", index + 1));
    };
    if fields.next().is_some()
        || a.len() != 8
        || b.len() != 4
        || !a.bytes().chain(b.bytes()).all(|c| c.is_ascii_hexdigit())
    {
        return Err(format!("Invalid CodeBreaker line {}", index + 1));
    }
    Ok((
        u32::from_str_radix(a, 16).unwrap(),
        u16::from_str_radix(b, 16).unwrap(),
    ))
}

/// The GBA CodeBreaker seed controls a 48-bit permutation and two XOR masks.
/// The caller supplies it explicitly: ciphertext can also start with a 9.
struct Cipher {
    permutation: [usize; 48],
    key: u16,
    first_word: u32,
    first_half: u16,
    second_word: u32,
    second_half: u16,
}

fn random(state: &mut u32) -> u32 {
    let mut next = || {
        *state = state.wrapping_mul(0x41c64e6d).wrapping_add(0x3039);
        *state
    };
    let a = next();
    let b = next();
    let c = next();
    (a << 14 & 0xc0000000) | (b >> 1 & 0x3fff8000) | (c >> 16 & 0x7fff)
}

impl Cipher {
    fn new(seed_word: u32, seed_value: u16) -> Result<Self, String> {
        if seed_word >> 28 != 9 {
            return Err("CodeBreaker seed must start with 9".into());
        }
        let mut state = 0x1111 ^ u32::from(seed_value & 0xff);
        let mut permutation = std::array::from_fn(|i| i);
        for _ in 0..80 {
            let x = random(&mut state) as usize % 48;
            let y = random(&mut state) as usize % 48;
            permutation.swap(x, y);
        }
        state = 0x4efad1c3;
        for _ in 0..((seed_word >> 24) & 15) {
            state = random(&mut state);
        }
        let second_word = random(&mut state);
        let second_half = random(&mut state) as u16;
        state = 0xf254 ^ u32::from(seed_value >> 8);
        for _ in 0..(seed_value >> 8) {
            state = random(&mut state);
        }
        let first_word = random(&mut state);
        let first_half = random(&mut state) as u16;
        Ok(Self {
            permutation,
            key: seed_word as u16,
            first_word,
            first_half,
            second_word,
            second_half,
        })
    }

    fn decrypt(&self, word: u32, half: u16) -> (u32, u16) {
        let mut bytes = [0u8; 6];
        bytes[..4].copy_from_slice(&word.to_le_bytes());
        bytes[4..].copy_from_slice(&half.to_le_bytes());
        let order = [3, 2, 1, 0, 5, 4];
        for i in (0..48).rev() {
            let j = self.permutation[i];
            let (x, y) = (order[i / 8], order[j / 8]);
            let difference = ((bytes[x] >> (i % 8)) ^ (bytes[y] >> (j % 8))) & 1;
            bytes[x] ^= difference << (i % 8);
            bytes[y] ^= difference << (j % 8);
        }
        let w = u32::from_le_bytes(bytes[..4].try_into().unwrap()) ^ self.first_word;
        bytes[..4].copy_from_slice(&w.to_le_bytes());
        let h = u16::from_le_bytes(bytes[4..].try_into().unwrap()) ^ self.first_half;
        bytes[4..].copy_from_slice(&h.to_le_bytes());
        let lo = u32::from(self.key & 0xff);
        let hi = u32::from(self.key >> 8);
        let middle = u32::from(u16::from_le_bytes([bytes[1], bytes[2]]));
        let result_word = (u32::from(bytes[3]).wrapping_mul(0x01010000)
            + u32::from(bytes[0]) * 0x100
            + u32::from(bytes[5]))
            ^ (middle * 0x10001)
            ^ self.second_word
            ^ (lo * 0x01010101)
            ^ (hi << 24);
        let result_half = (u32::from(bytes[5]) + (u32::from(bytes[0] ^ bytes[4]) << 8))
            ^ (lo * 0x101)
            ^ u32::from(self.second_half);
        (result_word, result_half as u16)
    }
}

/// Interpret every supplied entry line as ciphertext under the explicit seed.
/// Seed changes and master behavior are deliberately not inferred or executed.
pub(crate) fn inspect_seeded(lines: &[String], seed: &str) -> Result<Plan, String> {
    if lines.is_empty() || lines.len() > 1024 {
        return Err("A block needs 1..1024 lines".into());
    }
    if seed.len() != 13 || seed.as_bytes()[8] != b':' {
        return Err("CodeBreaker seed must use 9XXXXXXX:XXXX".into());
    }
    let (seed_word, seed_half) = line(&seed.replace(':', " "), 0)?;
    let cipher = Cipher::new(seed_word, seed_half)?;
    let plaintext = lines
        .iter()
        .enumerate()
        .map(|(i, text)| {
            let (word, half) = line(text, i)?;
            let (word, half) = cipher.decrypt(word, half);
            Ok(format!("{word:08X} {half:04X}"))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut plan = inspect(&plaintext)?;
    plan.format = "codebreaker-advance-seeded-subset";
    plan.assumption = "Caller supplies a seed and assumes every entry line is ciphertext under it; source device, master and ROM behavior are unverified";
    Ok(plan)
}

fn master_crc(rom: &[u8]) -> u16 {
    let mut crc = 0xffffu16;
    for i in 0..65536 {
        let mut x = u16::from(rom.get(i).copied().unwrap_or(0xff)) ^ (crc >> 8);
        x ^= x >> 4;
        crc = crc.wrapping_shl(8) ^ x.wrapping_mul(0x1001) ^ x.wrapping_mul(0x20);
    }
    crc
}

/// Decode one explicitly selected source's seed/game-ID/hook master.
/// This is a description only: the native backend does not install this hook.
pub(crate) fn inspect_source(
    lines: &[String],
    master: &[String],
    rom: Option<&[u8]>,
) -> Result<(Plan, serde_json::Value), String> {
    if master.len() != 3 {
        return Err("Selected source master needs exactly seed, game ID and hook lines".into());
    }
    let (seed_word, seed_half) = line(&master[0], 0)?;
    let cipher = Cipher::new(seed_word, seed_half)?;
    let (w, h) = line(&master[1], 1)?;
    let (game_id, flags) = cipher.decrypt(w, h);
    let (w, h) = line(&master[2], 2)?;
    let (hook, slot) = cipher.decrypt(w, h);
    if game_id >> 16 != 0 || flags > 15 || hook >> 28 != 1 || hook & 0x0e000001 != 0 || slot > 7 {
        return Err("Selected source master is not a supported seed/game-ID/hook block".into());
    }
    let expected = game_id as u16;
    let checksum_enabled = flags & 8 != 0;
    let rom_info = rom.map(|rom| {
        let actual = master_crc(rom);
        let sha256 = gba_core::ffta_rom::digest(rom);
        serde_json::json!({"sha256":sha256,"bytes":rom.len(),"first_64k_crc16":actual,
            "checksum_state":if !checksum_enabled {"not-required"} else if actual==expected {"match"} else {"mismatch"},
            "scope":"CRC and SHA-256 calculated from supplied bytes; not proof of device hook or cheat behavior"})
    });
    let seed = format!("{seed_word:08X}:{seed_half:04X}");
    let mut plan = inspect_seeded(lines, &seed)?;
    plan.format = "codebreaker-advance-source-bound-subset";
    plan.assumption = "Caller selected the source and linked master; seed and entry encryption are source assumptions, not verified device behavior";
    let info = serde_json::json!({"seed_sha256":gba_core::ffta_rom::digest(seed.as_bytes()),
        "game_id":{"checksum":expected,"checksum_enabled":checksum_enabled,"disable_interrupts":flags&2!=0,"flags":flags},
        "hook":{"rom_offset":hook&0x01ffffff,"address":0x08000000u32+(hook&0x01ffffff),"mode":"thumb",
            "handler_slot":slot,"handler_address":0x08000100u32+u32::from(slot)*0x00400000},
        "rom":rom_info,"executable":false,
        "scope":"Decoded source master metadata; no native hook or code is installed"});
    Ok((plan, info))
}

fn aligned(address: u32, width: u8, index: usize) -> Result<(), String> {
    if address % u32::from(width) != 0
        || address
            .checked_add(u32::from(width) - 1)
            .is_none_or(|end| end > 0x0fff_ffff)
    {
        return Err(format!(
            "Unaligned or out-of-range address at line {}",
            index + 1
        ));
    }
    Ok(())
}

fn action(lines: &[String], index: &mut usize) -> Result<Operation, String> {
    let start = *index;
    let (word, value) = line(&lines[start], start)?;
    *index += 1;
    let address = word & 0x0fff_ffff;
    let opcode = word >> 28;
    let op = match opcode {
        3 => {
            aligned(address, 1, start)?;
            if value > 0xff {
                return Err(format!("Byte value exceeds 8 bits at line {}", start + 1));
            }
            Operation::Write {
                address,
                width: 1,
                value,
            }
        }
        8 => {
            aligned(address, 2, start)?;
            Operation::Write {
                address,
                width: 2,
                value,
            }
        }
        2 | 6 | 14 => {
            aligned(address, 2, start)?;
            Operation::Modify {
                address,
                width: 2,
                operation: match opcode {
                    2 => "or",
                    6 => "and",
                    _ => "add",
                },
                value,
            }
        }
        4 => {
            aligned(address, 2, start)?;
            let continuation = lines
                .get(*index)
                .ok_or_else(|| format!("Missing slide continuation after line {}", start + 1))?;
            let (second, address_increment) = line(continuation, *index)?;
            *index += 1;
            let count = second as u16;
            let value_increment = (second >> 16) as u16;
            if count == 0 || count > 1024 || address_increment % 2 != 0 {
                return Err(format!(
                    "Unsupported slide count or alignment at line {}",
                    start + 1
                ));
            }
            let last = address
                .checked_add(u32::from(count - 1) * u32::from(address_increment))
                .ok_or_else(|| format!("Slide address overflow at line {}", start + 1))?;
            aligned(last, 2, start)?;
            Operation::Slide {
                address,
                width: 2,
                value,
                count,
                address_increment,
                value_increment,
            }
        }
        _ => return Err(format!("Unsupported opcode at line {}", start + 1)),
    };
    Ok(op)
}

pub(crate) fn inspect(lines: &[String]) -> Result<Plan, String> {
    if lines.is_empty() || lines.len() > 1024 {
        return Err("A block needs 1..1024 lines".into());
    }
    let mut operations = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let (word, value) = line(&lines[index], index)?;
        let opcode = word >> 28;
        if opcode == 7 || opcode == 10 {
            let start = index;
            let address = word & 0x0fff_ffff;
            aligned(address, 2, start)?;
            index += 1;
            if index == lines.len() {
                return Err(format!(
                    "Condition lacks its next operation at line {}",
                    start + 1
                ));
            }
            let next = action(lines, &mut index)?;
            operations.push(Operation::Conditional {
                address,
                width: 2,
                comparison: if opcode == 7 { "equal" } else { "not-equal" },
                value,
                next: Box::new(next),
            });
        } else {
            operations.push(action(lines, &mut index)?);
        }
    }
    Ok(Plan { format: "codebreaker-advance-plaintext-subset", assumption: "Caller assumes every line is plaintext; device, encryption, master and source dependencies are not verified", endianness: "little", executable: false, operations })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn decode(lines: &[&str]) -> Result<Plan, String> {
        inspect(&lines.iter().map(|v| (*v).into()).collect::<Vec<_>>())
    }
    #[test]
    fn seeded_cipher_matches_public_collision_and_independent_emulator_vectors() {
        for (sw, sh, word, half, expected) in [
            (0x9abcdef0, 0x1234, 0x909ee49b, 0x10cb, (0x72024c10, 0x9eb7)),
            (0x90000000, 0x0000, 0x00000000, 0x0000, (0x2b37a221, 0x7163)),
            (0x9fffffff, 0xffff, 0xffffffff, 0xffff, (0xbe1c52c9, 0x15bd)),
            (0x90000001, 0x0100, 0x12345678, 0xabcd, (0xccbcc2b4, 0xdb60)),
        ] {
            assert_eq!(Cipher::new(sw, sh).unwrap().decrypt(word, half), expected);
        }
        assert!(Cipher::new(0x80000000, 0).is_err());
    }
    #[test]
    fn seeded_inspection_decrypts_whole_continuations_and_rejects_atomically() {
        let lines = vec![
            "BB28F300 B8C0".into(),
            "8C8303D6 5E13".into(),
            "2C940CDC 920B".into(),
        ];
        let plan = inspect_seeded(&lines, "9ABCDEF0:1234").unwrap();
        assert!(!plan.executable);
        assert_eq!(
            plan.operations[0],
            Operation::Write {
                address: 0x02000001,
                width: 1,
                value: 255
            }
        );
        assert_eq!(
            plan.operations[1],
            Operation::Slide {
                address: 0x02000004,
                width: 2,
                value: 16,
                count: 4,
                address_increment: 2,
                value_increment: 3
            }
        );
        assert!(inspect_seeded(&lines[..2], "9ABCDEF0:1234").is_err());
        assert!(inspect_seeded(
            &["BB28F300 B8C0".into(), "4CD38AFF 7753".into()],
            "9ABCDEF0:1234"
        )
        .is_err());
        for seed in [
            "8ABCDEF0:1234",
            "9ABCDEF0::1234",
            "9ABCDEF0 1234",
            "9ABCDEF0:123Z",
        ] {
            assert!(inspect_seeded(&lines, seed).is_err());
        }
    }
    #[test]
    fn selected_master_decodes_metadata_and_checks_supplied_rom_bytes() {
        let lines = vec!["BB28F300 B8C0".into()];
        let master = vec![
            "9ABCDEF0 1234".into(),
            "B131BCEF 0691".into(),
            "26960CD0 140B".into(),
        ];
        let rom = (0..65536).map(|n| n as u8).collect::<Vec<_>>();
        let (plan, info) = inspect_source(&lines, &master, Some(&rom)).unwrap();
        assert!(!plan.executable);
        assert_eq!(info["game_id"]["checksum"], 0xffe7);
        assert_eq!(info["game_id"]["checksum_enabled"], true);
        assert_eq!(info["hook"]["rom_offset"], 0x00100000);
        assert_eq!(info["hook"]["handler_slot"], 7);
        assert_eq!(info["hook"]["handler_address"], 0x09c00100u32);
        assert_eq!(info["rom"]["checksum_state"], "match");
        assert_eq!(
            inspect_source(&lines, &master, Some(&[])).unwrap().1["rom"]["checksum_state"],
            "mismatch"
        );
        assert_eq!(master_crc(&[]), 0);
        assert!(inspect_source(&lines, &master[..2], None).is_err());
        let mut invalid = master.clone();
        invalid[2] = "BB28F300 B8C0".into();
        assert!(inspect_source(&lines, &invalid, None).is_err());
    }
    #[test]
    fn one_shot_preflights_every_access_and_uses_width_correct_bus_operations() {
        use gba_core::Bus;
        let plan = decode(&[
            "74000130 03FE",
            "82000002 1111",
            "42000004 0010",
            "00010003 0002",
            "E2000002 0001",
        ])
        .unwrap();
        let footprint = plan.footprint().unwrap();
        assert_eq!(footprint.max_writes, 5);
        assert!(footprint.reads.contains(&(0x04000130, 0x04000131)));
        let mut machine = gba_core::Machine::new(vec![]);
        machine.bus.keys = 0x3fe;
        let clock = machine.bus.clock;
        let frame = machine.bus.frames;
        assert_eq!(plan.apply_once(&mut machine.bus).unwrap(), 5);
        assert_eq!(machine.bus.read16(0x02000002), 0x1112);
        assert_eq!(machine.bus.read16(0x02000004), 0x10);
        assert_eq!(machine.bus.read16(0x02000006), 0x11);
        assert_eq!(machine.bus.read16(0x02000008), 0x12);
        assert_eq!((machine.bus.clock, machine.bus.frames), (clock, frame));
        let mut other = gba_core::Machine::new(vec![]);
        other.bus.keys = 0x3ff;
        assert_eq!(plan.apply_once(&mut other.bus).unwrap(), 4);
        assert_eq!(other.bus.read16(0x02000002), 1);
        let invalid = decode(&["82000000 1234", "84000130 5678"]).unwrap();
        let before = other.save_state().unwrap();
        assert!(invalid.apply_once(&mut other.bus).is_err());
        assert_eq!(other.save_state().unwrap(), before);
        assert!(decode(&["72000000 0000", "88000000 1234"])
            .unwrap()
            .footprint()
            .is_err());
        assert!(decode(&["42000000 0000", "00000101 0002"])
            .unwrap()
            .footprint()
            .is_err());
    }
    #[test]
    fn typed_writes_modifiers_and_order() {
        let plan = decode(&[
            "32000001 00FF",
            "82000002 1234",
            "22000004 0080",
            "62000006 FFFE",
            "E2000008 0001",
        ])
        .unwrap();
        assert_eq!(
            plan.operations,
            vec![
                Operation::Write {
                    address: 0x02000001,
                    width: 1,
                    value: 255
                },
                Operation::Write {
                    address: 0x02000002,
                    width: 2,
                    value: 0x1234
                },
                Operation::Modify {
                    address: 0x02000004,
                    width: 2,
                    operation: "or",
                    value: 0x80
                },
                Operation::Modify {
                    address: 0x02000006,
                    width: 2,
                    operation: "and",
                    value: 0xfffe
                },
                Operation::Modify {
                    address: 0x02000008,
                    width: 2,
                    operation: "add",
                    value: 1
                }
            ]
        );
        assert!(!plan.executable);
    }
    #[test]
    fn conditions_are_bound_to_one_complete_simple_operation() {
        let plan = decode(&[
            "72000002 1234",
            "32000001 0001",
            "A2000004 0000",
            "82000006 9999",
        ])
        .unwrap();
        assert!(
            matches!(&plan.operations[0],Operation::Conditional{comparison:"equal", next, ..} if matches!(**next,Operation::Write{width:1,..}))
        );
        assert!(matches!(
            &plan.operations[1],
            Operation::Conditional {
                comparison: "not-equal",
                ..
            }
        ));
    }
    #[test]
    fn slide_continuation_is_consumed_and_preserved() {
        let plan = decode(&["42000000 0010", "00030004 0002", "32000009 0001"]).unwrap();
        assert_eq!(
            plan.operations[0],
            Operation::Slide {
                address: 0x02000000,
                width: 2,
                value: 0x10,
                count: 4,
                address_increment: 2,
                value_increment: 3
            }
        );
        assert_eq!(plan.operations.len(), 2);
    }
    #[test]
    fn conditional_slide_guards_the_whole_bounded_operation() {
        use gba_core::Bus;
        let plan = decode(&[
            "72000000 1234",
            "42000020 0010",
            "00030004 0002",
            "32000040 0077",
        ])
        .unwrap();
        assert_eq!(plan.operations.len(), 2);
        assert!(matches!(&plan.operations[0], Operation::Conditional { next, .. }
            if matches!(**next, Operation::Slide { count: 4, .. })));
        assert_eq!(plan.footprint().unwrap().max_writes(), 5);

        let mut machine = gba_core::Machine::new(vec![]);
        assert_eq!(plan.apply_once(&mut machine.bus).unwrap(), 1);
        assert_eq!(machine.bus.read16(0x02000020), 0);
        assert_eq!(machine.bus.read8(0x02000040), 0x77);

        machine.bus.write16(0x02000000, 0x1234);
        assert_eq!(plan.apply_once(&mut machine.bus).unwrap(), 5);
        for i in 0..4 {
            assert_eq!(machine.bus.read16(0x02000020 + i * 2), 0x10 + i as u16 * 3);
        }

        let oversized = decode(&[
            "82000010 1234",
            "72000000 1234",
            "42000020 0000",
            "00000101 0002",
        ])
        .unwrap();
        let before = machine.save_state().unwrap();
        assert!(oversized.apply_once(&mut machine.bus).is_err());
        assert_eq!(machine.save_state().unwrap(), before);
    }
    #[test]
    fn malformed_incomplete_encrypted_unknown_and_ambiguous_blocks_are_atomic_errors() {
        for lines in [
            vec![],
            vec!["32000000 0001", "garbage"],
            vec!["42000000 0001"],
            vec!["72000000 0000"],
            vec!["72000000 0000", "72000002 0000"],
            vec!["92000000 0000"],
            vec!["12000000 0000"],
            vec!["52000000 0001", "00000000 0000"],
            vec!["82000001 0000"],
            vec!["32000000 0100"],
            vec!["42000000 0000", "00000000 0002"],
            vec!["42000000 0000", "00000401 0002"],
            vec!["42000000 0000", "00000002 0001"],
            vec!["4FFFFFFE 0000", "00000002 0002"],
            vec!["３２００００００ 0001"],
        ] {
            assert!(decode(&lines).is_err(), "accepted {lines:?}");
        }
    }
}
