//! Bounded EUR coordinate paths. Heights, buffer selection and movement remain guest work.
use super::{script_archive, script_commands, SourceRange};
use crate::ffta_rom;
use serde::{Deserialize, Serialize};

pub const SOURCE_OFFSET: u32 = 0xd68fd4;
pub const SOURCE_END: u32 = 0xd69bc4;
pub const PATH_COUNT: usize = 210;
pub const POINT_COUNT: usize = 856;
pub const SOURCE_BYTES: usize = 3056;

/// Original low16 index arithmetic, not a bound on live guest IDs.
pub fn index_source_offset(raw_id: u32) -> u32 {
    SOURCE_OFFSET + u32::from(raw_id as u16) * 4
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Point {
    pub x: u8,
    pub y: u8,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoordinatePath {
    pub id: u16,
    pub source_offset: u32,
    pub points: Vec<Point>,
    /// Exact source alignment after this list, separately preserved from its coordinates.
    pub alignment_after: Vec<u8>,
}
impl CoordinatePath {
    pub fn byte(&self, relative: usize) -> Option<u8> {
        let count = u16::try_from(self.points.len()).ok()?;
        if relative < 2 {
            return Some(count.to_le_bytes()[relative]);
        }
        let relative = relative - 2;
        if let Some(point) = self.points.get(relative / 2) {
            return Some(if relative % 2 == 0 { point.x } else { point.y });
        }
        self.alignment_after
            .get(relative.checked_sub(self.points.len() * 2)?)
            .copied()
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptLink {
    pub instruction_offset: u32,
    pub script_opcode: u8,
    pub path_id: u16,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptCoordinates {
    pub source_offset: u32,
    pub paths: Vec<CoordinatePath>,
    pub script_links: Vec<ScriptLink>,
}
impl ScriptCoordinates {
    pub fn byte_at(&self, at: u32) -> Option<u8> {
        let relative = at.checked_sub(self.source_offset)? as usize;
        if relative < self.paths.len().checked_mul(4)? {
            return Some(
                self.paths
                    .get(relative / 4)?
                    .source_offset
                    .checked_sub(self.source_offset)?
                    .to_le_bytes()[relative % 4],
            );
        }
        let index = self
            .paths
            .partition_point(|p| p.source_offset <= at)
            .checked_sub(1)?;
        let path = self.paths.get(index)?;
        path.byte(at.checked_sub(path.source_offset)? as usize)
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.source_offset != SOURCE_OFFSET || self.paths.len() != PATH_COUNT {
            return Err("Coordinate directory source/count mismatch".into());
        }
        let mut next = SOURCE_OFFSET + PATH_COUNT as u32 * 4;
        let mut points = 0;
        let mut padding = 0;
        for (id, path) in self.paths.iter().enumerate() {
            let count =
                u16::try_from(path.points.len()).map_err(|_| "Coordinate count overflow")?;
            if usize::from(path.id) != id || path.source_offset != next || count == 0 {
                return Err("Coordinate path ID/order/extent mismatch".into());
            }
            next = next
                .checked_add(2 + u32::from(count) * 2)
                .ok_or("Coordinate extent overflow")?;
            let align = (4 - next % 4) % 4;
            if path.alignment_after.len() != align as usize
                || path.alignment_after.iter().any(|&v| v != 0xff)
            {
                return Err("Coordinate alignment mismatch".into());
            }
            next = next
                .checked_add(align)
                .ok_or("Coordinate alignment overflow")?;
            if next > SOURCE_END {
                return Err("Coordinate path exceeds source".into());
            }
            points += path.points.len();
            padding += path.alignment_after.len();
        }
        if next != SOURCE_END || points != POINT_COUNT || padding != 84 {
            return Err("Coordinate physical source coverage mismatch".into());
        }
        if self.script_links.len() != 68
            || self
                .script_links
                .windows(2)
                .any(|v| v[0].instruction_offset >= v[1].instruction_offset)
            || self.script_links.iter().any(|v| {
                !matches!(v.script_opcode, 0x44 | 0x45) || usize::from(v.path_id) >= PATH_COUNT
            })
        {
            return Err("Coordinate script-link shape mismatch".into());
        }
        Ok(())
    }
    pub fn validate_dependencies(
        &self,
        archive: &script_archive::ScriptArchive,
        commands: &script_commands::ScriptCommands,
    ) -> Result<(), String> {
        self.validate()?;
        archive.validate()?;
        commands.validate()?;
        for (opcode, advance, callback) in [(0x44, 5, 0x127090), (0x45, 4, 0x1270b8)] {
            let command = &commands.commands[opcode];
            if command.advance != advance || command.callback.rom_offset != callback {
                return Err("Coordinate producer command dependency mismatch".into());
            }
        }
        if self.script_links != script_links(archive)? {
            return Err("Coordinate producer archive links mismatch".into());
        }
        Ok(())
    }
    pub fn source_spans(&self) -> Result<Vec<(u32, Vec<u8>, String)>, String> {
        self.validate()?;
        let bytes = (SOURCE_OFFSET..SOURCE_END)
            .map(|at| self.byte_at(at).ok_or("Coordinate source gap"))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(vec![(
            SOURCE_OFFSET,
            bytes,
            "engine/script-coordinate-paths/index-count-xy-alignment".into(),
        )])
    }
    pub fn verify_source(&self, rom: &[u8]) -> Result<(), String> {
        if self != &Catalog::from_rom(rom)?.decoded {
            return Err("Coordinate paths differ from authenticated source".into());
        }
        Ok(())
    }
}

fn script_links(archive: &script_archive::ScriptArchive) -> Result<Vec<ScriptLink>, String> {
    let mut links = Vec::new();
    for program in &archive.programs {
        for ins in &program.instructions {
            let p = &ins.parameters;
            let path_id = match ins.command {
                // Original byte3 remains uninterpreted in the archive; the callback does not read it.
                0x44 if p.len() == 4 => u16::from(p[1]),
                0x45 if p.len() == 3 => u16::from_le_bytes([p[0], p[1]]),
                0x44 | 0x45 => return Err("Coordinate script argument shape mismatch".into()),
                _ => continue,
            };
            links.push(ScriptLink {
                instruction_offset: ins.source_offset,
                script_opcode: ins.command,
                path_id,
            });
        }
    }
    Ok(links)
}

pub struct Catalog {
    pub decoded: ScriptCoordinates,
    pub sources: Vec<SourceRange>,
}
impl Catalog {
    pub fn from_rom(rom: &[u8]) -> Result<Self, String> {
        if rom.len() != 0x1000000 || ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256 {
            return Err("Coordinate paths require exact European source ROM".into());
        }
        verify_contract(rom)?;
        let mut paths = Vec::new();
        for id in 0..PATH_COUNT {
            let index = SOURCE_OFFSET as usize + id * 4;
            let relative = u32::from_le_bytes(rom[index..index + 4].try_into().unwrap());
            let at = SOURCE_OFFSET
                .checked_add(relative)
                .ok_or("Coordinate target overflow")?;
            if at < SOURCE_OFFSET + PATH_COUNT as u32 * 4 || at >= SOURCE_END - 1 {
                return Err("Coordinate target outside source".into());
            }
            let count = u16::from_le_bytes(rom[at as usize..at as usize + 2].try_into().unwrap());
            let stop = at + 2 + u32::from(count) * 2;
            let next = stop
                .checked_add((4 - stop % 4) % 4)
                .ok_or("Coordinate record overflow")?;
            if next > SOURCE_END {
                return Err("Coordinate record exceeds source".into());
            }
            let points = rom[at as usize + 2..stop as usize]
                .chunks_exact(2)
                .map(|p| Point { x: p[0], y: p[1] })
                .collect();
            paths.push(CoordinatePath {
                id: id as u16,
                source_offset: at,
                points,
                alignment_after: rom[stop as usize..next as usize].to_vec(),
            });
        }
        let archive = script_archive::Catalog::from_rom(rom)?.decoded;
        let commands = script_commands::Catalog::from_rom(rom)?.decoded;
        let decoded = ScriptCoordinates {
            source_offset: SOURCE_OFFSET,
            paths,
            script_links: script_links(&archive)?,
        };
        decoded.validate_dependencies(&archive, &commands)?;
        let mut sources = Vec::new();
        for (offset, bytes, role) in decoded.source_spans()? {
            if bytes != rom[offset as usize..offset as usize + bytes.len()] {
                return Err("Coordinate source roundtrip mismatch".into());
            }
            sources.push(SourceRange {
                offset,
                length: bytes.len() as u32,
                sha256: ffta_rom::digest(&bytes),
                role,
            });
        }
        Ok(Self { decoded, sources })
    }
}
fn verify_contract(rom: &[u8]) -> Result<(), String> {
    for &(offset, value) in CONTRACT_HALFWORDS {
        if u16::from_le_bytes(
            rom[offset as usize..offset as usize + 2]
                .try_into()
                .unwrap(),
        ) != value
        {
            return Err("Coordinate original reader/producer contract mismatch".into());
        }
    }
    for &(offset, value) in CONTRACT_WORDS {
        if u32::from_le_bytes(
            rom[offset as usize..offset as usize + 4]
                .try_into()
                .unwrap(),
        ) != value
        {
            return Err("Coordinate root/successor/live-map contract mismatch".into());
        }
    }
    Ok(())
}

const CONTRACT_HALFWORDS: &[(u32, u16)] = &[
    (0x128914, 0xb5f0),
    (0x128916, 0x464f),
    (0x128918, 0x4646),
    (0x12891a, 0xb4c0),
    (0x12891c, 0x0400),
    (0x12891e, 0x0609),
    (0x128920, 0x0e09),
    (0x128922, 0x4a1c),
    (0x128924, 0x0b80),
    (0x128926, 0x1880),
    (0x128928, 0x6800),
    (0x12892a, 0x1885),
    (0x12892c, 0x1cae),
    (0x12892e, 0x1c08),
    (0x128930, 0xf7fc),
    (0x128932, 0xff1a),
    (0x128934, 0x4680),
    (0x128936, 0x4642),
    (0x128938, 0x3280),
    (0x12893a, 0x7829),
    (0x12893c, 0x7868),
    (0x12893e, 0x0200),
    (0x128940, 0x4308),
    (0x128942, 0x6010),
    (0x128944, 0x4644),
    (0x128946, 0x2700),
    (0x128948, 0x7829),
    (0x12894a, 0x7868),
    (0x12894c, 0x0200),
    (0x12894e, 0x4308),
    (0x128950, 0x4287),
    (0x128952, 0xda15),
    (0x128954, 0x2000),
    (0x128956, 0x4681),
    (0x128958, 0x7830),
    (0x12895a, 0x7020),
    (0x12895c, 0x7870),
    (0x12895e, 0x7060),
    (0x128960, 0x7820),
    (0x128962, 0x7861),
    (0x128964, 0xf6f5),
    (0x128966, 0xfa10),
    (0x128968, 0x70a0),
    (0x12896a, 0x4648),
    (0x12896c, 0x70e0),
    (0x12896e, 0x3701),
    (0x128970, 0x3602),
    (0x128972, 0x3404),
    (0x128974, 0x7829),
    (0x128976, 0x7868),
    (0x128978, 0x0200),
    (0x12897a, 0x4308),
    (0x12897c, 0x4287),
    (0x12897e, 0xdbeb),
    (0x128980, 0x4640),
    (0x128982, 0x3080),
    (0x128984, 0x7800),
    (0x128986, 0xbc18),
    (0x128988, 0x4698),
    (0x12898a, 0x46a1),
    (0x12898c, 0xbcf0),
    (0x12898e, 0xbc02),
    (0x128990, 0x4708),
    (0x125768, 0x0600),
    (0x12576a, 0x0e00),
    (0x12576c, 0x4a03),
    (0x12576e, 0x0141),
    (0x125770, 0x1809),
    (0x125772, 0x0089),
    (0x125774, 0x3158),
    (0x125776, 0x6810),
    (0x125778, 0x1840),
    (0x12577a, 0x4770),
    (0x127090, 0xb510),
    (0x127092, 0xb081),
    (0x127094, 0x784c),
    (0x127096, 0x788a),
    (0x127098, 0x7909),
    (0x12709a, 0x230f),
    (0x12709c, 0x400b),
    (0x12709e, 0x0909),
    (0x1270a0, 0x9100),
    (0x1270a2, 0x1c21),
    (0x1270a4, 0xf004),
    (0x1270a6, 0xfb14),
    (0x1270a8, 0x4802),
    (0x1270aa, 0xb001),
    (0x1270ac, 0xbc10),
    (0x1270ae, 0xbc02),
    (0x1270b0, 0x4708),
    (0x1270b8, 0xb500),
    (0x1270ba, 0x7848),
    (0x1270bc, 0x788a),
    (0x1270be, 0x0212),
    (0x1270c0, 0x4310),
    (0x1270c2, 0x78c9),
    (0x1270c4, 0xf001),
    (0x1270c6, 0xfc26),
    (0x1270c8, 0x2088),
    (0x1270ca, 0x0040),
    (0x1270cc, 0xbc02),
    (0x1270ce, 0x4708),
    (0x12b6d0, 0xb530),
    (0x12b6d2, 0xb087),
    (0x12b6d4, 0x1c05),
    (0x12b6d6, 0x9c0a),
    (0x12b6d8, 0x0609),
    (0x12b6da, 0x0e09),
    (0x12b6dc, 0x0612),
    (0x12b6de, 0x0e12),
    (0x12b6e0, 0x061b),
    (0x12b6e2, 0x0e1b),
    (0x12b6e4, 0x0624),
    (0x12b6e6, 0x0e24),
    (0x12b6e8, 0x2000),
    (0x12b6ea, 0x71a8),
    (0x12b6ec, 0x9501),
    (0x12b6ee, 0x9402),
    (0x12b6f0, 0x4668),
    (0x12b6f2, 0x8281),
    (0x12b6f4, 0x82c2),
    (0x12b6f6, 0x8303),
    (0x12b6f8, 0x4804),
    (0x12b6fa, 0x68ab),
    (0x12b6fc, 0x2110),
    (0x12b6fe, 0x466a),
    (0x12b700, 0xf7f9),
    (0x12b702, 0xffcc),
    (0x12b704, 0xb007),
    (0x12b706, 0xbc30),
    (0x12b708, 0xbc01),
    (0x12b70a, 0x4700),
    (0x12acf8, 0x6870),
    (0x12acfa, 0x6028),
    (0x12acfc, 0x7d30),
    (0x12acfe, 0xf7fe),
    (0x12ad00, 0xf9cd),
    (0x12ad02, 0x1c04),
    (0x12ad04, 0xf76f),
    (0x12ad06, 0xfbc8),
    (0x12ad08, 0x1c21),
    (0x12ad0a, 0xf771),
    (0x12ad0c, 0xffb5),
    (0x12ad0e, 0x6068),
    (0x12ad10, 0x68b0),
    (0x12ad12, 0x73e8),
    (0x12ad14, 0x8af0),
    (0x12ad16, 0x210f),
    (0x12ad18, 0xf7fd),
    (0x12ad1a, 0xfdfc),
    (0x12ad1c, 0x7328),
    (0x12ad1e, 0x7b28),
    (0x1dd88, 0x0600),
    (0x1dd8a, 0x0609),
    (0x1dd8c, 0x0e09),
    (0x1dd8e, 0x4b05),
    (0x1dd90, 0x7a1a),
    (0x1dd92, 0x434a),
    (0x1dd94, 0x0052),
    (0x1dd96, 0x6859),
    (0x1dd98, 0x1889),
    (0x1dd9a, 0x0dc0),
    (0x1dd9c, 0x1809),
    (0x1dd9e, 0x7808),
    (0x1dda0, 0x4770),
    (0xa1f8, 0xb510),
    (0xa1fa, 0x2400),
    (0xa1fc, 0x4b0d),
    (0xa1fe, 0x7801),
    (0xa200, 0x0049),
    (0xa202, 0x18c9),
    (0xa204, 0x880a),
    (0xa206, 0x18d2),
    (0xa208, 0x7841),
    (0xa20a, 0x0049),
    (0xa20c, 0x1889),
    (0xa20e, 0x8809),
    (0xa210, 0x1852),
    (0xa212, 0x6042),
    (0xa214, 0x8044),
    (0xa216, 0xf000),
    (0xa218, 0xf813),
    (0xa21a, 0x0600),
    (0xa21c, 0x0e00),
    (0xa21e, 0x2801),
    (0xa220, 0xd103),
    (0xa222, 0x4805),
    (0xa224, 0x4905),
    (0xa226, 0x1840),
    (0xa228, 0x7804),
    (0xa22a, 0x1c20),
    (0xa22c, 0xbc10),
    (0xa22e, 0xbc02),
    (0xa230, 0x4708),
];

const CONTRACT_WORDS: &[(u32, u32)] = &[
    (0x128994, 0x08d68fd4),
    (0x12577c, 0x020159d4),
    (0x12b70c, 0x0812acb5),
    (0x1dda4, 0x02007f10),
    (0xa234, 0x08d69bc4),
];
