//! Metadata plan for the bounded European task-345 tilemap directory.
use super::{
    bank_ownership::{Bank, Plan, Range},
    read_bounded,
};
use gba_core::{
    ffta_rom,
    rom_data::tilemap_transfer_directory::{
        TilemapTransferDirectory, COUNT_DIRECTORY, DIRECTORY_LITERALS, POINTER_DIRECTORY, ROM_BASE,
        ROM_SIZE, ROW_BYTES,
    },
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
    path::Path,
};

const ROLES: [&str; 6] = [
    "list-pointers",
    "list-counts",
    "rows",
    "descriptors",
    "wrappers",
    "lz77-inputs",
];

pub(super) fn plan(rom: &[u8]) -> Result<Plan, String> {
    let directory = TilemapTransferDirectory::from_rom(rom)?;
    let inputs: BTreeMap<_, _> = directory
        .lz77_input_ranges(rom)?
        .into_iter()
        .map(|input| {
            (
                input.start,
                Range {
                    offset: input.start,
                    length: input.end - input.start,
                },
            )
        })
        .collect();
    let descriptors: BTreeMap<_, _> = directory
        .descriptors
        .iter()
        .map(|descriptor| (ROM_BASE + descriptor.source_offset, descriptor))
        .collect();
    let row_ranges: BTreeMap<_, _> = directory
        .rows
        .iter()
        .map(|row| {
            let descriptor = descriptors[&row.descriptor_pointer];
            let sources = [
                Range {
                    offset: row.source_offset,
                    length: ROW_BYTES,
                },
                Range {
                    offset: descriptor.source_offset,
                    length: 8,
                },
                Range {
                    offset: descriptor.wrapper.source_offset,
                    length: 4,
                },
                inputs[&(descriptor.wrapper.source_offset + 4)],
            ];
            (row.source_offset, sources)
        })
        .collect();
    let mut banks = Vec::new();
    banks.push(Bank {
        id: "directory-addresses".into(),
        ranges: vec![Range {
            offset: DIRECTORY_LITERALS,
            length: 8,
        }],
    });
    let mut by_role: [BTreeSet<Range>; ROLES.len()] = std::array::from_fn(|_| BTreeSet::new());
    for (index, slot) in directory.slots.iter().enumerate() {
        let pointer = Range {
            offset: POINTER_DIRECTORY + 4 * index as u32,
            length: 4,
        };
        let count = Range {
            offset: COUNT_DIRECTORY + index as u32,
            length: 1,
        };
        by_role[0].insert(pointer);
        by_role[1].insert(count);
        let mut sources = vec![pointer, count];
        for row in slot.row_offsets()? {
            let linked = row_ranges[&row];
            sources.extend(linked);
            for (ranges, span) in by_role[2..].iter_mut().zip(linked) {
                ranges.insert(span);
            }
        }
        banks.push(Bank {
            id: format!("slot-{index}"),
            ranges: sources,
        });
    }
    banks.extend(ROLES.into_iter().zip(by_role).map(|(role, spans)| Bank {
        id: role.into(),
        ranges: spans.into_iter().collect(),
    }));
    Ok(Plan {
        schema: 1,
        source_rom_sha256: ffta_rom::EUROPE_SHA256.into(),
        source_rom_size: ROM_SIZE,
        profile: "europe".into(),
        banks,
    })
}

pub(super) fn run(rom_path: &Path) -> Result<(), String> {
    let rom = read_bounded(rom_path, u64::from(ROM_SIZE))?;
    let plan = plan(&rom)?;
    let mut output = io::stdout().lock();
    serde_json::to_writer_pretty(&mut output, &plan)
        .map_err(|error| format!("Cannot write tilemap bank plan: {error}"))?;
    writeln!(output).map_err(|error| format!("Cannot finish tilemap bank plan: {error}"))
}
