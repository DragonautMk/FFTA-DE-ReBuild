//! Compare complete bank plans with a retained manifest using source ranges alone.
use super::{parse, read_bounded, Manifest, MAX_MANIFEST_FILE_BYTES};
use gba_core::{ffta_rom::digest, rom_data::SCHEMA};
use serde::{ser::SerializeSeq, Deserialize, Serialize, Serializer};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
    path::Path,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Plan {
    schema: u32,
    source_rom_sha256: String,
    source_rom_size: u32,
    profile: String,
    banks: Vec<Bank>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bank {
    id: String,
    ranges: Vec<Range>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Range {
    offset: u32,
    length: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Span {
    start: u32,
    end: u32,
}

impl Span {
    fn checked(offset: u32, length: u32, size: u32, label: &str) -> Result<Self, String> {
        let end = offset
            .checked_add(length)
            .filter(|&end| length > 0 && end <= size)
            .ok_or_else(|| {
                format!("Invalid source range for {label}: offset {offset}, length {length}")
            })?;
        Ok(Self { start: offset, end })
    }
}

fn union(mut spans: Vec<Span>) -> Vec<Span> {
    spans.sort_unstable();
    let mut merged: Vec<Span> = Vec::new();
    for span in spans {
        match merged.last_mut() {
            Some(last) if span.start <= last.end => last.end = last.end.max(span.end),
            _ => merged.push(span),
        }
    }
    merged
}

#[derive(Serialize)]
struct Segment<'a> {
    start: u32,
    end: u32,
    owners: Vec<&'a str>,
}

fn append_segment<'a>(segments: &mut Vec<Segment<'a>>, segment: Segment<'a>) {
    match segments.last_mut() {
        Some(last) if last.end == segment.start && last.owners == segment.owners => {
            last.end = segment.end;
        }
        _ => segments.push(segment),
    }
}

fn source_index(manifest: &Manifest) -> Result<Vec<Segment<'_>>, String> {
    let mut events = Vec::new();
    let mut ids = BTreeSet::new();
    for resource in &manifest.resources {
        let id = resource.id.as_str();
        if id.is_empty() || !ids.insert(id) {
            return Err(format!("Empty or duplicate manifest resource ID: {id:?}"));
        }
        let spans = resource
            .sources
            .iter()
            .map(|source| Span::checked(source.offset, source.length, manifest.source_rom_size, id))
            .collect::<Result<Vec<_>, _>>()?;
        // Coalesce duplicate/overlapping roles before counting independent owners.
        for span in union(spans) {
            events.push((span.start, id, true));
            events.push((span.end, id, false));
        }
    }
    events.sort_unstable();
    let mut previous = 0;
    let mut active = BTreeSet::new();
    let mut index = Vec::new();
    for batch in events.chunk_by(|a, b| a.0 == b.0) {
        let position = batch[0].0;
        if !active.is_empty() {
            append_segment(
                &mut index,
                Segment {
                    start: previous,
                    end: position,
                    owners: active.iter().copied().collect(),
                },
            );
        }
        for &(_, id, start) in batch {
            if start {
                active.insert(id);
            } else {
                active.remove(id);
            }
        }
        previous = position;
    }
    Ok(index)
}

fn project<'a>(spans: &[Span], index: &[Segment<'a>]) -> Vec<Segment<'a>> {
    let mut segments = Vec::new();
    for span in spans {
        let mut cursor = span.start;
        let first = index.partition_point(|segment| segment.end <= span.start);
        for segment in index[first..]
            .iter()
            .take_while(|segment| segment.start < span.end)
        {
            if cursor < segment.start {
                segments.push(Segment {
                    start: cursor,
                    end: segment.start,
                    owners: Vec::new(),
                });
            }
            let end = span.end.min(segment.end);
            segments.push(Segment {
                start: span.start.max(segment.start),
                end,
                owners: segment.owners.clone(),
            });
            cursor = end;
        }
        if cursor < span.end {
            segments.push(Segment {
                start: cursor,
                end: span.end,
                owners: Vec::new(),
            });
        }
    }
    segments
}

#[derive(Default, Serialize)]
struct Counts {
    planned_bytes: u64,
    owned_bytes: u64,
    unowned_bytes: u64,
    shared_bytes: u64,
}

fn count(segments: &[Segment<'_>]) -> Counts {
    let mut counts = Counts::default();
    for segment in segments {
        let length = u64::from(segment.end - segment.start);
        counts.planned_bytes += length;
        if segment.owners.is_empty() {
            counts.unowned_bytes += length;
        } else {
            counts.owned_bytes += length;
            if segment.owners.len() > 1 {
                counts.shared_bytes += length;
            }
        }
    }
    counts
}

#[derive(Serialize)]
struct BankReport<'a> {
    id: &'a str,
    counts: Counts,
    segments: Vec<Segment<'a>>,
}

struct Banks<'a> {
    ranges: BTreeMap<&'a str, Vec<Span>>,
    index: Vec<Segment<'a>>,
}

impl Serialize for Banks<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut sequence = serializer.serialize_seq(Some(self.ranges.len()))?;
        // Repeated bank views can have many spans in common. Emit one bank at a time.
        for (&id, ranges) in &self.ranges {
            let segments = project(ranges, &self.index);
            sequence.serialize_element(&BankReport {
                id,
                counts: count(&segments),
                segments,
            })?;
        }
        sequence.end()
    }
}

#[derive(Serialize)]
struct Report<'a> {
    schema: u32,
    scope: &'static str,
    profile: &'a str,
    source_rom_sha256: &'a str,
    source_rom_size: u32,
    extractor_revision: u32,
    decoder_revision: u32,
    manifest_sha256: String,
    plan_sha256: String,
    distinct: Counts,
    banks: Banks<'a>,
}

fn audit<'a>(manifest: &'a Manifest, plan: &'a Plan) -> Result<(Counts, Banks<'a>), String> {
    if manifest.schema != SCHEMA || plan.schema != 1 {
        return Err("Unsupported ownership audit manifest or plan schema".into());
    }
    let rom_hash = &manifest.source_rom_sha256;
    if rom_hash.len() != 64
        || !rom_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        || manifest.profile.is_empty()
        || manifest.source_rom_size == 0
        || &plan.source_rom_sha256 != rom_hash
        || plan.source_rom_size != manifest.source_rom_size
        || plan.profile != manifest.profile
    {
        return Err("Manifest and bank plan must identify the same source ROM and profile".into());
    }
    if plan.banks.is_empty() {
        return Err("Ownership audit requires at least one bank".into());
    }
    let index = source_index(manifest)?;
    let mut ranges = BTreeMap::new();
    for bank in &plan.banks {
        let id = bank.id.as_str();
        if id.is_empty() || bank.ranges.is_empty() {
            return Err(format!("Bank ID and ranges must be present: {id:?}"));
        }
        let spans = bank
            .ranges
            .iter()
            .map(|range| Span::checked(range.offset, range.length, plan.source_rom_size, id))
            .collect::<Result<Vec<_>, _>>()?;
        if ranges.insert(id, union(spans)).is_some() {
            return Err(format!("Duplicate bank ID: {id:?}"));
        }
    }
    let all_spans = union(ranges.values().flatten().copied().collect());
    let totals = count(&project(&all_spans, &index));
    Ok((totals, Banks { ranges, index }))
}

pub(super) fn run(manifest_path: &Path, plan_path: &Path) -> Result<(), String> {
    let manifest_bytes = read_bounded(manifest_path, MAX_MANIFEST_FILE_BYTES)?;
    let plan_bytes = read_bounded(plan_path, MAX_MANIFEST_FILE_BYTES)?;
    let manifest: Manifest = parse(&manifest_bytes, "ownership audit manifest")?;
    let plan: Plan = serde_json::from_slice(&plan_bytes)
        .map_err(|error| format!("Invalid ownership audit plan: {error}"))?;
    let (distinct, banks) = audit(&manifest, &plan)?;
    let report = Report {
        schema: 1,
        scope: "manifest-source-ownership",
        profile: &manifest.profile,
        source_rom_sha256: &manifest.source_rom_sha256,
        source_rom_size: manifest.source_rom_size,
        extractor_revision: manifest.extractor_revision,
        decoder_revision: manifest.decoder_revision,
        manifest_sha256: digest(&manifest_bytes),
        plan_sha256: digest(&plan_bytes),
        distinct,
        banks,
    };
    let mut output = io::stdout().lock();
    serde_json::to_writer_pretty(&mut output, &report)
        .map_err(|error| format!("Cannot write ownership audit: {error}"))?;
    writeln!(output).map_err(|error| format!("Cannot finish ownership audit: {error}"))
}
