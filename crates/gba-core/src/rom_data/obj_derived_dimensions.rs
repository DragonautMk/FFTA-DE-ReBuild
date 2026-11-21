//! EUR shape/size lookup views derived from the authenticated OBJ dimensions.
use super::{ObjDimensions, SourceRange};
use crate::ffta_rom;

pub const TABLE_OFFSETS: [u32; 2] = [0x14d2fc, 0x14d32c];
pub const TABLE_BYTES: usize = 48;
const TABLE_HASHES: [&str; 2] = [
    "fae8e894f5ec0923c841505e953e961d1dff9720f95db4d6dd5f0c1b08f02935",
    "0ea1537e0ab6656f6da02fbc795b52a0f97bfab0346886491b37103dc181d2ec",
];
const ROLES: [&str; 2] = ["shape-size-tile-exponents", "shape-size-pixel-dimensions"];

/// Each record contains two byte fields and two authenticated zero bytes.
/// The guest's reserved shape remains an ordinary physical read, not a new cap.
pub fn projections(dimensions: &ObjDimensions) -> [[u8; TABLE_BYTES]; 2] {
    let mut tables = [[0; TABLE_BYTES]; 2];
    for (i, size) in dimensions.entries.iter().enumerate() {
        tables[0][i * 4] = size.width.trailing_zeros().saturating_sub(3) as u8;
        tables[0][i * 4 + 1] = size.height.trailing_zeros().saturating_sub(3) as u8;
        tables[1][i * 4] = size.width as u8;
        tables[1][i * 4 + 1] = size.height as u8;
    }
    tables
}

/// Authenticate both views and the original byte readers and root literals.
pub fn source_ranges(rom: &[u8], dimensions: &ObjDimensions) -> Result<Vec<SourceRange>, String> {
    dimensions.validate()?;
    if ffta_rom::digest(rom) != ffta_rom::EUROPE_SHA256
        || rom.len() != 0x0100_0000
        || ffta_rom::digest(&rom[0x1fb2a..0x1fb4c])
            != "e1c37834bf0435d60c43f5c090c336044316943256c6d752116e3b27f175b650"
        || ffta_rom::digest(&rom[0x1fb7c..0x1fb84])
            != "11618dcc1781549f8c463827aaa866e96ca36287d63399d53d8222bfc90f231c"
    {
        return Err("OBJ derived-dimension source or original reader mismatch".into());
    }
    let projections = projections(dimensions);
    let mut sources = Vec::with_capacity(TABLE_OFFSETS.len());
    for (i, offset) in TABLE_OFFSETS.into_iter().enumerate() {
        let bytes = &rom[offset as usize..offset as usize + TABLE_BYTES];
        if bytes != projections[i] || ffta_rom::digest(bytes) != TABLE_HASHES[i] {
            return Err("OBJ derived dimensions differ from authenticated source".into());
        }
        sources.push(SourceRange {
            offset,
            length: TABLE_BYTES as u32,
            sha256: TABLE_HASHES[i].into(),
            role: ROLES[i].into(),
        });
    }
    Ok(sources)
}
