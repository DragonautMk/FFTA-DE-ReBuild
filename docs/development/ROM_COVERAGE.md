<a id="mappa-della-copertura-rom"></a>

# ROM coverage map

The [navigable map](rom-coverage/index.html) represents the entire 16 MiB EUR ROM by offset, with 64 KiB cell selection, address navigation, family selection, linked evidence and the largest unclassified regions. Open the HTML file in a browser; it works offline without a server or dependencies. The [JSON data](rom-coverage/coverage.json) contains exact intervals, while cell colors summarize their density.

The **DBG-07 verification of October 28, 2026** uses the already authenticated **e76-d75** EUR snapshot without new extraction: ROM SHA-256 `8fab1e390cb9d5e695bfdb86e81246892e1aa8c0cb4014db8235ff76538ea632`, manifest `bd0881e0c16abbf9602a1faefd2243defbd64be3c884b37962b24081060ddf3b`. The extractor committed during verification is e79-d78, so the page explicitly identifies a **historical dataset**. The default path continues to follow the latest commit; dataset generation and evidence review are separate.

- **Candidate code:** 260,472 B, **1.55%**, from 28,455 retained ROM entry points; 404 RAM entries excluded. This does not measure executed code or completed decompilation.
- **Identified data available to the runtime:** 10,325,073 B, **61.54%**, in **68 families**. Each decoded file and ROM interval is hash-verified; the runtime remains opt-in. No source-only remainder exists in this snapshot.
- **Positive ordinary consumption:** **51 of 68 families**, inherited from e61 evidence only when decoded files and sources match. Thirteen families have evidence awaiting review; a new map does not automatically promote historical records. Positive family intervals cover 9,789,234 B, not bytes read.
- **Imported audits:** three existing **DS-20-EUR** reports, union of **31,061 B observed**; the long audit is **TRUNCATED**. Both complete short audits observe **12,416 B**. Selecting only the short typed audit: 11,495 B Typed and 997 B Raw, with 76 B overlap. These are not newly validated scenes.
- **Data shared by reference:** at least 1,359,934 B, **8.11%**, in **294 intervals**. Shared reads between different DS cases are another layer; three reports for DS-20 alone do not demonstrate sharing between different scenes.
- **Unclassified:** 6,253,479 B, **37.27%**, after data, header and candidate code. This does not mean free or unused space. Audits do not reduce it.

The categories **must not be added**. There are 24,120 B with multiple owner families and **61,996 B of code/data overlap**, to investigate as classification conflicts. The analyzer can interpret data as instructions. The distributed database contains zero functions for this ROM; extents are not inferred from the next symbol. The 188 B technical header `0x4..0xc0` remains separate from the initial branch.

<a id="interpretazione-delle-prove"></a>

## Interpreting evidence

[evidence.json](rom-coverage/evidence.json) associates each family with versioned records and retains the limits on actually observed fields. Evidence may belong to earlier generations whose files were retained: a single session with every family active is not invented. New or changed files enter the map as authenticated, with ordinary evidence awaiting review; they do not block HTML updates. The registry retains hashes for both decoded files and source maps of the 56 reviewed e61 resources. For example, the 48 B template records 96 source B in one scene: these are rereads, not coverage of 96 distinct bytes. Decompressed outputs and encoded bytes also have different units and are not ROM coverage addends.

The map's blue layer represents **intervals of families with positive use**, not individual bytes read. Even when a family is positive, other records, fields or languages may remain open. Selection gates separately retain their targeted oracle (831 distinct field B on real source), subsequent runtime integration and new ordinary dispatch evidence of 1,000 frames (277 source B, first 26 physical records). The latter does not prove reads of all 1,024 B or all 64 records.

The sharing inventory is a minimum. Ownership overlaps among fonts, glyphs or OBJ banks are calculated separately; they do not by themselves establish how many consumers read them. Not all textual, dynamic and indirect references have been enumerated. The newer audio inventory remains limited to song/header and tone/wave links: it does not exhaust sequences, banks or other graphs. Blocks copied to RAM are not colored entirely as ROM code merely because their mixed container is known.

<a id="scene-letture-e-confronto-dbg-07"></a>

## Scenes, reads and comparison (DBG-07)

[DBG-06](../debug_rom_audit.md) `rom-audit.json` reports are imported explicitly; the private collection is not scanned and the game need not be rerun. Page checkboxes select audits to display, including individually to distinguish raw/typed or a truncated window. Clicking the map or jumping to an offset shows families, cell intervals, DS cases, historical evidence and links to audit identity/limits. The family column counts only bytes observed in the current selection.

Layers remain separate:

- `runtime_available`: authenticated intervals of resources available to the runtime;
- `ordinary_resource_extents`: whole families with historical positive evidence;
- `scene_observed`: physical union of reads retained in the selected audits;
- `scene_typed`, `scene_raw`, `scene_unclassified`: service declared by collectors;
- `scene_shared`: bytes observed by at least two **distinct DS cases**, not two runs of one case; `shared_references` continues to inventory static references;
- `scene_outside_identified`: reads outside identified data, including any overlap with candidate code, without inventing owners.

Unions deduplicate rereads, widths and mirrors. Bus/host, Typed/Raw, resources and scenes can overlap: do not add their counters. A filter intersecting a read retains the full width of the actual read. `ordinary_distinct_read_bytes` remains N/A: selected audits do not replace the missing inventory of old ordinary evidence.

The generator recomputes unions from events and verifies the ROM, schema, intervals, mirrors, collectors, filters, frames and request/drop arithmetic. It attributes reads to the **map manifest** intervals; a different known plan remains provenance only. If it matches, the generator also verifies declared intersections. The report hash identifies imported content; outcome and run are declared by the collector without retroactively authenticating a new scene acceptance. It embeds no events, game bytes or host paths in the result.

Limits: at most 32 reports, 32 MiB per file and 128 MiB total; audit schema 1, six known readers and at most 50,000 events per audit. **TRUNCATED** and **PARTIAL** remain visible even after successful replay. A complete window covers only the declared thread, collectors and filters: other host/HLE accesses, other threads, native constants, authentication reads and other paths listed in the report are missing. Interpreter fetches are excluded; PC context may be inherited from HLE/DMA. Absences, drops and missing hooks do not prove bytes were never read.

Each generation compares the previous `coverage.json` in the output. `--compare` / `-ComparePath` selects a different snapshot of the same ROM. The page shows dates, generations, commits, manifest and previous report hash, plus bytes added/removed per layer, even when the total is unchanged. A different audit selection is flagged: an observed delta is not extraction progress. An explicitly incompatible comparison fails and preserves the output; an uncomparable old output is flagged and replaced with a valid new snapshot. JSON uses **schema 3**.

Example (replace paths with your own reports):

```powershell
.\scripts\windows\rom_coverage_docker.ps1 `
  -AuditPath 'target/caso-a/Data/config/replay/rom-audit.json','target/caso-b/Data/config/replay/rom-audit.json' `
  -ComparePath 'target/fotografia-precedente/coverage.json' -OpenReport
```

```bash
bash scripts/linux/rom_coverage_docker.sh \
  --audit target/caso-a/Data/config/replay/rom-audit.json \
  --audit target/caso-b/Data/config/replay/rom-audit.json \
  --compare target/fotografia-precedente/coverage.json
```

Audits and comparison are mounted read-only at runtime and remain outside the Docker context. Host Python is unnecessary. Without these arguments the map is generated **without audits**: the old selection is not silently reused. To repeat a selection, pass the same reports again.

<a id="rigenerazione"></a>

## Regeneration

Docker avoids installing Python, Rust/Cargo or `nm` on the host. On Windows, start Docker Desktop with **Linux containers**, then in PowerShell:

```powershell
.\scripts\windows\rom_coverage_docker.ps1
```

On Linux, with Docker and Buildx:

```bash
bash scripts/linux/rom_coverage_docker.sh
```

Both use **`Dockerfile.coverage`**, with Git and Docker on the host. They prepare a temporary context with runtime sources from a **HEAD commit fixed at startup**, archived by Git, and current public tools/evidence. Uncommitted extraction changes are excluded from the build. The normal executable, host Cargo cache and extraction jobs are not modified. Archived sources receive new extraction timestamps to prevent a warm Cargo cache from reusing binaries built from different sources.

The archive also includes `scripts/` from the same commit: `recomp` embeds public tools and HTML templates using `include_bytes!`, so these files must be present **before** compilation. Current coverage tools are copied afterward, keeping runtime-embedded assets distinct from map generators. A runtime fix that is only local does not enter the build until committed.

The Dockerfile uses BuildKit's included frontend: it does not separately download `docker/dockerfile:1` from Docker Hub. This avoids the preliminary `failed to fetch anonymous token` frontend failure when Docker DNS cannot resolve. The build still requires networking/DNS for uncached Rust images, Debian packages and crates; report generation continues to use `--network none`. See [Docker troubleshooting](../docker.md#rom-coverage-atlas).

The default path needs only the **EUR ROM**: the container builds the CLI and analyzer, reads the revision from those same compiled sources, prepares the dataset and retains it under `target/rom-data/coverage/<SHA-ROM>/e<extractor>-d<decoder>/`. Subsequent runs authenticate and reuse it without rewriting it. A newly committed revision selects a new cache. Launchers pass `--cache-keep 2`: after successful publication they retain the current dataset and the most recent other generation for that ROM, removing superseded automatic caches. Explicit datasets remain intact. Direct Python leaves cleanup disabled (`--cache-keep 0`); a positive count can enable it. Failed cleanup does not invalidate an already published report. See [artifact retention](LOCAL_ARTIFACTS.md). No GUI, native translation or game is run. Python and Rust are unnecessary on the host. The old direct `Dockerfile.linux --target coverage` target remains a developer path using working-tree sources, without the two launchers' Git isolation.

The 28,455 entry points are versioned metadata, hash-bound to the ROM, retained list and cache from which they were extracted. **The old local `.dll` is no longer needed**. ROM and evidence are mounted read-only; only local cache and output folder are writable. Extracted assets stay outside the image and Git. Generation uses disabled networking; the first build may download dependencies. On Linux files belong to the calling user.

Output: **`docs/development/rom-coverage/index.html`** and **`coverage.json`**. A newly generated HTML report and terminal show the **actual regeneration date/time**, commit, dataset generation and separate date/revision of ordinary evidence. The log compares counts with the previous report, including small byte increments that percentage rounding can hide. If coverage matches it shows **“Coverage unchanged”**, but HTML/JSON are still rewritten with a new date. Analyzer recompilation and date are not presented as increased coverage.

The checked-in snapshot was subsequently presented in English by translating only saved resource notes and limits. Its original `generated_at` and `evidence_sha256` continue to identify the source analysis; `presentation_translation` separately records the translation and translated evidence hash. Fifteen coverage-layer comparisons showed no changes. This presentation update did not read a ROM or dataset, execute the guest, or generate new analysis or acceptance. Since the translated evidence file has a different byte hash, a future regenerated report may report a metadata difference even with zero coverage-layer deltas.

Every launch also retains a log in **`target/build-logs/rom-coverage-*.log`**, including errors. A validation error does not replace the existing report. On PowerShell, `-OpenReport` opens the result in a browser with a versioned URL to request the new copy:

```powershell
.\scripts\windows\rom_coverage_docker.ps1 -OpenReport
```

To use an existing dataset or separate output:

```powershell
.\scripts\windows\rom_coverage_docker.ps1 -DatasetDir 'D:\FFTA dati\e65' -OutputDir 'target/coverage-preview'
```

```bash
bash scripts/linux/rom_coverage_docker.sh --dataset '/dati/FFTA e65' --output target/coverage-preview
```

An explicit dataset remains read-only. A version different from the extractor is accepted as a **historical snapshot**, with a visible HTML and terminal warning. Omit `--dataset` / `-DatasetDir` to follow the current version. The ROM, decoded files and sources must still authenticate. `-RomPath`, `-EvidencePath` and `-OutputDir` correspond to `--rom`, `--evidence`, `--output`. `-TranslationPath` / `--translation` remains an optional check on the retained original ELF cache; it is not required for the normal path. All relative paths start at the project root. Spaces are supported; commas, double quotes and newlines are unsupported in mounts. Docker must be able to access host paths.

Developer alternative with Python 3.11+ and Rust/Cargo:

```sh
cargo build --release --locked --offline -p recomp-core --example rom_code_ranges
python3 scripts/rom_coverage.py \
  --rom 'rom/1194 - Final Fantasy Tactics Advance (E)(Surplus).gba' \
  --dataset 'target/rom-data/coverage/8fab1e390cb9d5e695bfdb86e81246892e1aa8c0cb4014db8235ff76538ea632/e65-d64' \
  --analyzer target/release/examples/rom_code_ranges \
  --output docs/development/rom-coverage
python3 -m unittest discover -s scripts -p test_rom_coverage.py -v
```

A newly committed generation updates data automatically. Promoting ordinary-use evidence instead requires reviewing individual acceptances and explicitly updating `evidence.json`. The generator rejects ROMs, source hashes, decoded files or caches with a revision different from the extractor that must produce them. Retain the previous snapshot when comparing progress. Intervals use file offsets and exclusive ends `[start, end)`; add `0x08000000` for the first CPU mirror. Mirrors do not multiply ROM bytes.

Output contains metadata only. ROMs, saves, inputs, game images, decoded assets and generated code remain outside versioned files. No new benchmark, recording or accepted runtime test was repeated for the map. Report validation includes counts against an independent byte-set model, authenticated real generation and browser checks.

The Docker path was verified with real e65 EUR extraction, comparison with the 61 already accepted files, reuse without writes and HTML/JSON regeneration. Tests cover new/historical generations, resources awaiting review, source-only migration, Git isolation, timestamps, preserving reports on error and independent counts. The PowerShell launcher was verified with mocked Docker on Linux; native Windows/PowerShell 5.1/Docker Desktop remain untested directly.

DBG-07 was verified with 26 generator/import tests, eight Bash tests and 12 PowerShell 7.4 checks in a container (mocked Docker), real Docker import of retained audits, rejection of incorrect identities and 1400/620 px browser checks. The e76 dataset and audits remain read-only; no scenes were rerun. Native Windows/PowerShell 5.1/Docker Desktop remain to be tested.
