#!/usr/bin/env python3
"""Build a metadata-only ROM coverage atlas. Python 3.11+, standard library.

ROM and provided datasets are read-only; automatic preparation writes only to
the selected cache. No ROM bytes, instructions, text or decoded assets enter the
report. Code candidates, runtime availability and ordinary use remain separate.
"""
import argparse
import collections
import datetime
import hashlib
import json
import os
import pathlib
import re
import sqlite3
import subprocess
import sys
import tempfile
import tomllib

from artifact_coordinator import Lease
from local_artifacts import prune_coverage_cache
from rom_coverage_ranges import union, length, intersection, complement, multiple_owners, checked_span
from rom_coverage_audit import attach_audits, bounded_json, compare_reports

ROOT = pathlib.Path(__file__).resolve().parents[1]


def sha(path):
    with open(path, "rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def retained_seeds(path, evidence, rom):
    if sha(path) != evidence["code_seeds_sha256"]:
        raise ValueError("retained code-seed metadata differs from reviewed evidence")
    metadata = json.loads(path.read_text())
    if metadata["schema"] != 1 or metadata["rom_sha256"] != hashlib.sha256(rom).hexdigest():
        raise ValueError("code-seed metadata/ROM identity mismatch")
    seeds = [int(value, 16) for value in metadata["entries"]]
    if not seeds or seeds != sorted(set(seeds)) or any(
            not 0x08000000 <= (key & ~1) < 0x08000000 + len(rom) for key in seeds):
        raise ValueError("invalid retained ROM entry seeds")
    text = "".join(f"{key:08x}\n" for key in seeds)
    if hashlib.sha256(text.encode()).hexdigest() != metadata["seeds_sha256"]:
        raise ValueError("retained code-seed list checksum mismatch")
    if metadata["translation_sha256"] != evidence["translation_sha256"]:
        raise ValueError("code-seed provenance differs from reviewed translation")
    return seeds, metadata


def code_ranges(rom_path, translation, analyzer, rom, work, seed_path, evidence):
    if translation is not None:
        symbols = subprocess.check_output(["nm", "--defined-only", str(translation)], text=True)
        entries = {(int(addr, 16) | (mode == "t")) for addr, mode in
                   re.findall(r"\bb_([0-9a-fA-F]{8})_([ta])\b", symbols)}
        seeds = sorted(key for key in entries if 0x08000000 <= (key & ~1) < 0x08000000 + len(rom))
        excluded = len(entries) - len(seeds)
        translation_digest = sha(translation)
    else:
        seeds, metadata = retained_seeds(seed_path, evidence, rom)
        excluded = metadata["excluded_non_rom_entries"]
        translation_digest = metadata["translation_sha256"]
    if not seeds:
        raise ValueError("no native ROM entry symbols in translation")
    seed_file = work / "seeds.txt"
    seed_file.write_text("".join(f"{key:08x}\n" for key in seeds))
    lines = subprocess.check_output([str(analyzer.resolve()), str(rom_path.resolve()),
                                     str(seed_file)], text=True).splitlines()
    fnv = 0xcbf29ce484222325
    for byte in rom:
        # Match recomp_core::fnv64, including its existing multiplier.
        fnv = ((fnv ^ byte) * 0x1_0000_01b3) & 0xffffffffffffffff
    expected = f"# rom_bytes={len(rom)} rom_fnv64={fnv:016x} seeds={len(seeds)}"
    if len(lines) < 2 or lines[0] != expected or lines[1] != "start\tend\tmode\tinstructions\tretained_entry":
        raise ValueError("analyzer/ROM identity mismatch")
    spans, by_mode, retained, instruction_visits = [], collections.defaultdict(list), 0, 0
    for line in lines[2:]:
        a, b, mode, count, old = line.split("\t")
        if mode not in ("arm", "thumb") or old not in ("0", "1"):
            raise ValueError("invalid analyzer row")
        span = checked_span(int(a), int(b), len(rom))
        if int(count) * (2 if mode == "thumb" else 4) != span[1] - span[0]:
            raise ValueError("instruction span/width mismatch")
        spans.append(span)
        by_mode[mode].append(span)
        retained += int(old)
        instruction_visits += int(count)
    if retained != len(seeds):
        raise ValueError("retained entry missing from analysis")
    return union(spans), {
        "method": "Current recursive analyzer seeded by retained native ROM block symbols; speculative candidates, not executed-byte coverage or semantic function coverage.",
        "retained_rom_entries": len(seeds), "excluded_non_rom_entries": excluded,
        "discovered_blocks": len(spans), "instruction_visits_with_overlap": instruction_visits,
        "arm_bytes": length(by_mode["arm"]), "thumb_bytes": length(by_mode["thumb"]),
        "arm_thumb_overlap_bytes": length(intersection(by_mode["arm"], by_mode["thumb"])),
        "translation_sha256": translation_digest, "analyzer_executable_sha256": sha(analyzer),
        "entry_source": "native ELF" if translation is not None else "reviewed address metadata",
        "analyzer_source_sha256": sha(ROOT / "crates/recomp-core/src/analyze.rs"),
        "seeds_sha256": sha(seed_file),
    }


def collect_sharing(dataset, manifest):
    """Explicit same-address references, separate from cross-resource ownership.

    Bounded lower bound: BG descriptors, packet indexes, transfer links, and
    same-resource manifest aliases. Identical payloads at different offsets are
    never classified as shared addresses.
    """
    resources = {r["id"]: r for r in manifest["resources"]}
    refs = collections.defaultdict(set)
    spans = {}
    starts = {}
    for rid, r in resources.items():
        by_range = collections.defaultdict(list)
        for source in r["sources"]:
            a, b = source["offset"], source["offset"] + source["length"]
            starts[rid, a] = (a, b)
            by_range[a, b].append(source["role"])
        for span, roles in by_range.items():
            if len(set(roles)) >= 2:
                key = rid, *span
                spans[key] = list(span)
                refs[key].update("manifest:" + role for role in roles)

    def link(rid, start, ref):
        if (rid, start) not in starts:
            raise ValueError(f"unresolved sharing link {rid} at {start:#x}")
        a, b = starts[rid, start]
        key = rid, a, b
        spans[key] = [a, b]
        refs[key].add(ref)

    def load(rid):
        return json.loads((dataset / resources[rid]["path"]).read_text())

    root = resources["graphics.bg-descriptors"]["sources"][0]["offset"]
    for i, desc in enumerate(load("graphics.bg-descriptors")["entries"]):
        for field, rid in (("graphics_source", "graphics.bg-main-graphics"),
                           ("palette_source", "graphics.bg-base-palettes")):
            link(rid, root + desc[field], f"descriptor/{i}/{field}")
    for plane, catalog in load("graphics.bg-packet-programs").items():
        for i, index in enumerate(catalog["descriptor_programs"]):
            if index is not None:
                program = catalog["programs"][index]
                link("graphics.bg-packet-programs", root + program["relative_source"],
                     f"descriptor/{i}/{plane}")
    transfers = load("graphics.bg-transfer-programs")
    for i, row in enumerate(transfers["descriptor_links"]):
        for j, entry in enumerate(row):
            if entry is not None:
                for field, records in (("control", "controls"), ("tiles", "tiles")):
                    source = transfers[records][entry[field]]["relative_source"]
                    link("graphics.bg-transfer-programs", root + source, f"descriptor/{i}/transfer/{j}/{field}")
    if "audio.m4a-directory" in resources:
        audio = load("audio.m4a-directory")
        headers = {h["id"]: h for h in audio["headers"]}
        for song in audio["songs"]:
            link("audio.m4a-directory", headers[song["header_id"]]["source_offset"],
                 f"song/{song['id']}/header")
    if "audio.m4a-tones" in resources:
        audio = load("audio.m4a-tones")
        waves = {0x08000000 + w["source_offset"]: w for w in audio["pcm_waves"]}
        for tone in audio["tones"]:
            wave = waves.get(tone["word04"])
            if wave is not None:
                for start in [wave["source_offset"], wave["source_offset"] + 16]:
                    if ("audio.m4a-tones", start) in starts:
                        link("audio.m4a-tones", start, f"tone/{tone['id']}/wave")
    result = []
    for key in sorted(refs):
        if len(refs[key]) >= 2:
            # Prefer explicit decoded links to duplicate manifest role labels.
            explicit = [r for r in refs[key] if not r.startswith("manifest:")]
            selected = explicit if len(explicit) >= 2 else sorted(refs[key])
            if len(selected) >= 2:
                result.append({"resource": key[0], "range": spans[key],
                               "references": len(selected), "examples": sorted(selected)[:4]})
    return result


def check_baseline(manifest, evidence, digest):
    if digest != evidence["baseline"][0]:
        raise ValueError("ROM differs from reviewed evidence")
    for key in ("extractor_revision", "decoder_revision"):
        if type(manifest[key]) is not int or manifest[key] < 1:
            raise ValueError(f"invalid dataset {key}")


def extractor_generation():
    """Read the frozen source compiled into the coverage image, never evidence."""
    source = (ROOT / "crates/gba-core/src/rom_data.rs").read_text()
    return tuple(int(re.search(rf"pub const {name}_REV: u32 = (\d+);", source)[1])
                 for name in ("EXTRACTOR", "DECODER"))


def sources_sha256(resource):
    return hashlib.sha256(json.dumps(resource["sources"], sort_keys=True,
                                    separators=(",", ":")).encode()).hexdigest()


def resource_evidence(resource, evidence):
    """Carry ordinary proofs forward only for identical content and source map."""
    note = evidence["resources"].get(resource["id"])
    if (note and resource["sha256"] == note["decoded_sha256"]
            and sources_sha256(resource) == note.get("sources_sha256")):
        if not note["references"] or any(not (ROOT / ref).is_file() for ref in note["references"]):
            raise ValueError(f"missing evidence record {resource['id']}")
        return {**note, "evidence_status": "reviewed"}
    return {"ordinary_positive": False, "references": [], "evidence_status": "unreviewed",
            "note": "Resource and sources authenticated in the selected dataset; "
                    "ordinary-use evidence pending review."}


def source_only_spans(extra, rom):
    spans = []
    for source in extra["sources"]:
        a, b = checked_span(source["offset"], source["offset"] + source["length"], len(rom))
        if hashlib.sha256(rom[a:b]).hexdigest() != source["sha256"]:
            raise ValueError(f"source-only authentication failed: {extra['id']}")
        spans.append([a, b])
    if union(spans) != union(extra["ranges"]):
        raise ValueError(f"source-only ranges differ from authenticated spans: {extra['id']}")
    return union(spans)


def prepare_dataset(args):
    if args.dataset is not None:
        return
    evidence = json.loads(pathlib.Path(args.evidence).read_text())
    digest = evidence["baseline"][0]
    extractor_rev, decoder_rev = extractor_generation()
    if sha(args.rom) != digest:
        raise ValueError("ROM differs from reviewed evidence; no dataset was created")
    dataset = pathlib.Path(args.dataset_cache) / digest / f"e{extractor_rev}-d{decoder_rev}"
    if not (dataset / "manifest.toml").is_file():
        if not args.extractor:
            raise ValueError("automatic dataset preparation requires --extractor")
        dataset.parent.mkdir(parents=True, exist_ok=True)
        print(f"Preparing dataset e{extractor_rev}-d{decoder_rev}: {dataset}", flush=True)
        subprocess.run([args.extractor, "rom-data", "extract", args.rom, str(dataset)], check=True)
    else:
        print(f"Retained dataset e{extractor_rev}-d{decoder_rev}: verify and reuse", flush=True)
    manifest = tomllib.loads((dataset / "manifest.toml").read_text())
    if (manifest["extractor_revision"], manifest["decoder_revision"]) != (extractor_rev, decoder_rev):
        raise ValueError("cached/extracted dataset differs from built extractor revision; "
                         "use a clean --dataset-cache directory")
    args.dataset = str(dataset)


def build(args):
    rom_path, dataset = pathlib.Path(args.rom), pathlib.Path(args.dataset)
    rom = rom_path.read_bytes()
    manifest_path = dataset / "manifest.toml"
    manifest = tomllib.loads(manifest_path.read_text())
    evidence = json.loads(pathlib.Path(args.evidence).read_text())
    digest = hashlib.sha256(rom).hexdigest()
    if digest != manifest["source_rom_sha256"] or len(rom) != manifest["source_rom_size"]:
        raise ValueError("manifest/ROM identity mismatch")
    check_baseline(manifest, evidence, digest)
    if args.translation is not None and sha(args.translation) != evidence["translation_sha256"]:
        raise ValueError("translation differs from the retained evidence baseline")
    print(f"Verifying e{manifest['extractor_revision']}-d{manifest['decoder_revision']}: "
          f"{len(manifest['resources'])} resources and their ROM ranges", flush=True)
    records, owners, all_data, ordinary = [], {}, [], []
    for resource in manifest["resources"]:
        rid = resource["id"]
        path = (dataset / resource["path"]).resolve()
        if not path.is_relative_to(dataset.resolve()) or sha(path) != resource["sha256"]:
            raise ValueError(f"invalid decoded resource {rid}")
        spans = []
        for s in resource["sources"]:
            a, b = checked_span(s["offset"], s["offset"] + s["length"], len(rom))
            if hashlib.sha256(rom[a:b]).hexdigest() != s["sha256"]:
                raise ValueError(f"source authentication failed: {rid}")
            spans.append([a, b])
        note = resource_evidence(resource, evidence)
        spans = union(spans)
        owners[rid] = spans
        all_data.extend(spans)
        if note["ordinary_positive"]:
            ordinary.extend(spans)
        records.append({"id": rid, "ranges": spans, "source_bytes": length(spans),
                        "manifest_roles": len(resource["sources"]), "runtime_available": True,
                        "decoded_sha256": resource["sha256"], **note})
    runtime = union(all_data)
    for extra in evidence["source_only"]:
        spans = intersection(source_only_spans(extra, rom), complement(runtime, len(rom)))
        if any(not (ROOT / ref).is_file() for ref in extra["references"]):
            raise ValueError("missing source-only evidence")
        if not spans:
            continue  # Already migrated into authenticated runtime resources.
        all_data.extend(spans)
        extra_id = extra["id"] + "/source-only" if extra["id"] in owners else extra["id"]
        records.append({**extra, "id": extra_id, "ranges": spans, "source_bytes": length(spans),
                        "runtime_available": False, "ordinary_positive": False})
    data = union(all_data)
    print("Static code analysis from retained entry points", flush=True)
    with tempfile.TemporaryDirectory() as work:
        code, code_info = code_ranges(rom_path, pathlib.Path(args.translation) if args.translation else None,
                                      pathlib.Path(args.analyzer), rom, pathlib.Path(work),
                                      pathlib.Path(args.code_seeds), evidence)
    db_path = ROOT / "gamedb.sqlite"
    with sqlite3.connect(db_path.as_uri() + "?mode=ro", uri=True) as db:
        code_info["gamedb_functions_for_rom"] = db.execute(
            "SELECT COUNT(*) FROM functions WHERE sha256 = ?", (digest,)).fetchone()[0]
    code_info["gamedb_sha256"] = sha(db_path)
    shared_records = collect_sharing(dataset, manifest)
    shared = union([r["range"] for r in shared_records])
    owner_overlap = multiple_owners(owners)
    header = [[4, 192]]  # Header branch 0..4 is code, not cartridge metadata.
    unknown = complement(code + data + header, len(rom))
    layers = {"code_candidates": code, "identified_data": data,
              "runtime_available": runtime, "ordinary_resource_extents": union(ordinary),
              "shared_references": shared, "multiple_resource_owners": owner_overlap,
              "header_metadata": header, "unclassified": unknown,
              "code_data_overlap": intersection(code, data)}
    totals = {name: length(spans) for name, spans in layers.items()}
    totals.update({"rom": len(rom), "identified_families": len(records),
                   "runtime_families": len(manifest["resources"]),
                   "ordinary_positive_families": sum(r["ordinary_positive"] for r in records),
                   "ordinary_distinct_read_bytes": None,
                   "source_only_data": length(data) - length(runtime),
                   "identified_union_with_header": length(data + header),
                   "shared_records": len(shared_records)})
    built = extractor_generation()
    source_file = ROOT / "coverage-source.txt"
    report = {"schema": 2, "generated_at": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="microseconds"),
            "evidence_date": evidence["date"], "reviewed_generation": f"e{evidence['baseline'][1]}-d{evidence['baseline'][2]}",
            "source_revision": source_file.read_text().strip() if source_file.exists() else "working tree (direct invocation)",
            "extractor_generation": f"e{built[0]}-d{built[1]}",
            "historical_dataset": built != (manifest["extractor_revision"], manifest["decoder_revision"]),
            "pending_review": [r["id"] for r in records if r.get("evidence_status") == "unreviewed"],
            "profile": manifest["profile"],
            "rom_sha256": digest, "generation": f"e{manifest['extractor_revision']}-d{manifest['decoder_revision']}",
            "manifest_sha256": sha(manifest_path), "evidence_sha256": sha(args.evidence),
            "code": code_info, "totals": totals, "layers": layers,
            "resources": records, "sharing": shared_records,
            "limits": evidence["limits"], "bin_bytes": 65536,
            "largest_unclassified": sorted(unknown, key=lambda r: r[1] - r[0], reverse=True)[:20]}
    attach_audits(report, getattr(args, "audit", []))
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--rom", required=True)
    dataset_group = parser.add_mutually_exclusive_group(required=True)
    dataset_group.add_argument("--dataset", help="existing dataset, read-only")
    dataset_group.add_argument("--dataset-cache", help="prepare/reuse the built extractor's generation in this cache directory")
    parser.add_argument("--cache-lease", help="writable coordination root for an explicit dataset mounted read-only from an evictable cache; no pruning")
    parser.add_argument("--extractor", help="recomp CLI for automatic dataset preparation")
    parser.add_argument("--cache-keep", type=int, default=0,
                        help="after success, keep this many automatic cache generations; 0 disables pruning")
    parser.add_argument("--translation", help="optional original retained ELF; default uses versioned seed metadata")
    parser.add_argument("--code-seeds", default=str(ROOT / "docs/development/rom-coverage/code-seeds.json"))
    parser.add_argument("--analyzer", required=True, help="rom_code_ranges executable")
    parser.add_argument("--evidence", default=str(ROOT / "docs/development/rom-coverage/evidence.json"))
    parser.add_argument("--audit", action="append", default=[], help="DBG06 rom-audit.json, repeatable; metadata only")
    parser.add_argument("--compare", help="previous coverage.json, same ROM; default compares the existing output")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    if args.cache_lease and (args.dataset_cache or not args.dataset):
        parser.error("--cache-lease belongs to an explicit --dataset, not automatic preparation")
    if args.cache_keep < 0:
        parser.error("--cache-keep must not be negative")
    try:
        return generate(args)
    except (OSError, ValueError, KeyError, TypeError, OverflowError, subprocess.CalledProcessError) as error:
        print(f"ROM coverage failed: {error}", file=sys.stderr, flush=True)
        return 1


def explicit_cache_root(dataset):
    if not dataset:
        return None
    path = pathlib.Path(dataset).resolve()
    if re.fullmatch(r"e[0-9]+-d[0-9]+", path.name) and re.fullmatch(r"[0-9a-f]{64}", path.parent.name):
        return str(path.parent.parent)
    return None


def generate(args):
    automatic = getattr(args, "dataset_cache", None)
    cache = automatic or getattr(args, "cache_lease", None) or explicit_cache_root(getattr(args, "dataset", None))
    if cache:
        if automatic:
            pathlib.Path(cache).mkdir(parents=True, exist_ok=True)
        with Lease(cache, "coverage-generate") as lease:
            return _generate(args, lease)
    return _generate(args, None)


def _generate(args, lease):
    prepare_dataset(args)
    report = build(args)
    output = pathlib.Path(args.output)
    output.mkdir(parents=True, exist_ok=True)
    template = pathlib.Path(__file__).with_name("rom_coverage.html").read_text()
    previous = None
    comparison_path = getattr(args, "compare", None)
    if comparison_path or (output / "coverage.json").is_file():
        try:
            previous, old_hash, _ = bounded_json(comparison_path or output / "coverage.json")
            if not isinstance(previous.get("totals"), dict):
                raise ValueError("invalid previous report")
            if "schema" in previous:
                report["comparison"] = compare_reports(report, previous, old_hash)
        except (OSError, ValueError, KeyError, TypeError) as error:
            if comparison_path:
                raise ValueError(f"invalid --compare report: {error}") from error
            previous = None
            report["comparison_note"] = "Previous report cannot be compared: " + str(error)
    payload = json.dumps(report, ensure_ascii=False, separators=(",", ":"))
    with tempfile.TemporaryDirectory(dir=output) as staging:
        staging = pathlib.Path(staging)
        (staging / "coverage.json").write_text(payload + "\n")
        (staging / "index.html").write_text(template.replace("__ROM_COVERAGE_DATA__", payload.replace("<", "\\u003c")))
        os.replace(staging / "coverage.json", output / "coverage.json")
        os.replace(staging / "index.html", output / "index.html")
    print(f"Report {report['generation']} · generated {report['generated_at']} · "
          f"{report['totals']['runtime_families']} runtime families", flush=True)
    print(f"Reviewed evidence {report['reviewed_generation']} of {report['evidence_date']}; "
          f"{len(report['pending_review'])} families with evidence pending review.", flush=True)
    if report["historical_dataset"]:
        print(f"WARNING: explicit historical dataset; extractor {report['extractor_generation']}. "
              "Omit --dataset/-DatasetDir to follow the current version.", flush=True)
    if previous:
        if semantic_report(previous) == semantic_report(report):
            print("Coverage unchanged; index.html and coverage.json regenerated with a new timestamp.", flush=True)
        else:
            print(f"Comparison with {previous.get('generation', '?')}:", flush=True)
            for key in ("identified_data", "runtime_available", "unclassified", "runtime_families"):
                if key in previous["totals"] and key in report["totals"]:
                    delta = report["totals"][key] - previous["totals"][key]
                    print(f"  {key}: {delta:+d}", flush=True)
    print(json.dumps(report["totals"], indent=2))
    print(f"HTML: {output / 'index.html'}\nJSON: {output / 'coverage.json'}", flush=True)
    if getattr(args, "dataset_cache", None) and getattr(args, "cache_keep", 0):
        try:
            removed = prune_coverage_cache(args.dataset_cache, args.dataset, args.cache_keep, lease=lease)
            if removed:
                print(f"Coverage cache: removed {len(removed)} superseded generations; "
                      f"retained up to {args.cache_keep}.", flush=True)
        except (OSError, ValueError) as error:
            print(f"Report updated; cache cleanup incomplete: {error}", file=sys.stderr)
    return 0


def semantic_report(report):
    """Ignore generation time/build identity when comparing actual coverage."""
    result = {k: v for k, v in report.items() if k not in ("generated_at", "source_revision", "comparison", "comparison_note")}
    result["code"] = {k: v for k, v in report.get("code", {}).items()
                      if k != "analyzer_executable_sha256"}
    return result


if __name__ == "__main__":
    raise SystemExit(main())
