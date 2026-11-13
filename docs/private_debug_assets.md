<a id="esportazione-privata-degli-asset-di-debug"></a>

# Private debug asset export

Create a **local, private Windows portable ZIP** with the available workspace
saves, snapshots, inputs, imports, recordings and diagnostic reports. The script
scans the workspace automatically, including later experiments under `target/`.
The default snapshot selection retains representative checkpoints instead of
consecutive micro-snapshots. ROM, BIOS,
extracted ROM datasets, generated/native code, build caches, Wine profiles and
previous exports stay excluded. Never publish this ZIP or add it to Git.

```bash
bash scripts/linux/export_debug_assets.sh
```

```powershell
.\scripts\windows\export_debug_assets.ps1
```

The default `debug_saves` and `saves` inputs are under the sibling
`../FFTARecomp-private/` folder. The old checkout paths are relative links for
compatibility; the exporter reads the real private directories. Working debug
sessions are excluded because their isolated folders also contain a ROM and
an executable. Their versioned outputs remain in the private worktree.

Python 3 is required. `python3 scripts/export_debug_assets.py` also works
(`py -3` on Windows). The script prints a new timestamped ZIP under
`dist/debug-assets/`; it never overwrites an existing output. Extract into a
new folder, launch `FFTARecomp-debug-EUR/FFTARecomp.exe` and select your own
compatible ROM. Debug and Extended saves are enabled in this portable copy.

For the **extended edition**, including consecutive micro-snapshots, use the
separate entry point:

```bash
bash scripts/linux/export_debug_assets_extended.sh
```

```powershell
.\scripts\windows\export_debug_assets_extended.ps1
```

`python3 scripts/export_debug_assets_extended.py` is the direct Python entry
point. It creates `FFTARecomp-debug-assets-extended-EUR-<timestamp>.zip`; the
standard script keeps `FFTARecomp-debug-assets-EUR-<timestamp>.zip`. Each edition
leaves every original micro-snapshot on disk for development/debugging use.
Only the standard ZIP omits intermediate captures. Extract each ZIP into a
separate destination when keeping both portable copies.

## Find and use every exported asset

Open **`DEBUG-ASSETS.html`** at the root of the extracted folder. This local,
searchable index lists every discovered source path and links to its exported
file, identity/issue and save catalog when identified. It opens on **Saves and
snapshots**, with one result per distinct state content and expandable source
aliases. **All exported source paths** also exposes the diagnostic reports and
other assets. Raw and preview-wrapped copies of the same core state share one
catalog item; a preview-bearing copy is preferred. Source files are unchanged.
The index displays the first 500 matches; refine the search to see other matches.

- **Game > Saves** retains the separate EUR working slots and cartridge saves.
- **Debug > Sessions and cases > Debug catalog** opens the original collection
  with its previously hidden `loaded_checkpoints` and EUR base now exposed.
  The original catalog remains byte-for-byte in `DebugAssets/original-catalog.json`.
- For other discovered states/saves, use **Choose / reload catalog** and select
  a root-level **`DEBUG-CATALOG-EUR-001.json`** or **`DEBUG-CATALOG-USA-001.json`**
  file named by the index. Further numbered pages appear automatically, with
  at most 256 entries per file to respect existing runtime limits. Set Region,
  Recorded status, Kind and DS case to **All** when browsing. Use **Open isolated
  case**, or import a checkpoint into a free slot inside an isolated case.
- USA files require a compatible USA ROM; EUR files require their compatible
  EUR ROM. Snapshots also retain their original BIOS and runtime constraints.
  A cartridge's region may be inferred from collection hashes or its source
  path; this is stated as an inference, not new gameplay validation.
- Invalid states, unidentified ROMs and unclassified cartridge saves are still
  included as raw files. The index states why no selectable catalog item was
  generated. Use their original compatible runtime/ROM; damaged files are not
  repaired or presented as playable.

Discovery includes incomplete recordings and experimental/synthetic snapshots.
They remain diagnostic material; export and valid envelope hashes do not promote
any scene to validated status. Source catalogs, imported files and recorded
acceptance evidence are not modified.

## Snapshot selection

By default, numeric capture series such as `step-0…N`, `boot-0…N` and numbered
navigation directories retain their first and last points. Intermediate captures
are omitted from the ZIP and its catalogs. Independent `slot-N` saves, DS/IN/GEN
case identities, descriptive standalone checkpoints, the original collections,
source-manifest inputs, reproduction boundaries and recording/report dependencies
remain included. This deterministic filename/provenance rule does not claim that
the omitted states are semantically equivalent or that scene coverage is complete.

Use the **extended script** for an integral snapshot export (equivalent to the
standard script's **`--all-snapshots`** option), or repeat
**`--keep-snapshot PATH`** to retain a particular intermediate checkpoint. The
inventory lists every omitted micro-snapshot, its checksum, reason and retained
sequence representative. The scripts contain no source cleanup or retirement:
all original states remain available to the system. Other debug asset types keep
their existing discovery scope.

## Inventory, scope and consistency

`DEBUG-ASSETS-INVENTORY.json` records all discovered paths, archive destinations,
SHA-256 values, byte counts, snapshot ROM/BIOS identities, catalog pages,
recording completeness/integrity, snapshot selection and explicit discovery exclusions. It states
the discovery policy. `DEBUG-ASSETS-MANIFEST.json` authenticates the exported
payloads; `PACKAGE-MANIFEST.json` describes the portable base only.

The scan includes saves/snapshots/inputs/SPS throughout the workspace and
supported diagnostic text, JSON, logs, recipes/configuration, HTML, screenshots
and audio under artifact trees such as `target/`, `debug_saves/`, `saves/`,
`cheats/` and portable `Data/`. This includes recognized graphics memory dumps
and raw PCM/FIFO audio captures. Reports keep their relative attachment layout.
It avoids source mirrors for diagnostic support
files and excludes extracted resource filenames maintained by the ROM extractor.
Symlink/junction directories are never followed. Links and excluded artifact
paths are reported. Unrecognized binaries are not treated as debug assets.
Git metadata files and directories named `.git` are excluded from copied asset
trees, including worktree pointer files containing host-local paths.
Scopes with an active or interrupted `.debug-artifacts-lease` are left untouched
and listed as excluded in both editions. This protects work in progress without
stopping jobs or reclaiming their locks. Export again after the owner completes
the work to include newly stable outputs.
No external home folders are scanned automatically. To include another local
asset directory, add repeatable `--scan-root PATH` arguments. A deliberate
`--exclude-scan PATH` is recorded in the inventory; it narrows completeness.

The original `debug_saves/`, `saves/debug-cases/`, source-manifest files,
dedicated complete recordings and F-09 replay remain included. Choose alternate
inputs with `--assets`, `--legacy`, `--source-manifest`, `--recordings` and
`--replay`. `--help` lists the defaults. Catalog hashes include loaded checkpoints
and the EUR base. Original source hashes and complete selected recordings must
match their declared identities. Discovered incomplete/mismatched recordings
are retained with explicit status rather than silently skipped.

The exporter checks Windows PE architecture, portable-base hashes, paths,
ROM/BIOS exclusions, ZIP integrity and copied SHA-256 values before publishing.
If a copied source changes during discovery/export, publication fails with its
path; retry when that source is stable. It does not stop another session or
modify a source. Files already retired from disk cannot be reconstructed by
export; the inventory covers available files observed during discovery.

Required base inputs are `dist/debug-tools/FFTARecomp-windows-debug-tools.zip`,
`target/x86_64-pc-windows-gnu/release/recomp.exe` and the adjacent `ffta-play.exe`.
Build these first or select existing inputs with `--base`, `--runtime` and
`--launcher`. Use `--output` to choose a new destination. No build, guest
execution, ROM extraction or source modification is performed by export.

Instruction contents and messages use English. Legacy `LEGGIMI.txt`,
`LEGGIMI-DEBUG.txt` and asset-folder names remain compatible. Existing collection
text and imported evidence are retained verbatim. Ordinary public packages
remain free of private assets. Native Windows gameplay, physical controllers
and playability of every discovered experiment are separate checks.

## Validation on 2026-11-05

Both real editions passed ZIP/hash/manifest checks. Standard exposes 784 distinct
discovery entries; extended exposes 2,394 and includes all 1,657 intermediate
snapshot paths omitted by standard. Every omitted original remained present with
its exact SHA-256. All 31 loaded checkpoints and the EUR base are accessible.
Fifteen synthetic checks, the generated index UI/syntax and existing Linux
runtime CLI reads of all 5 standard and 12 extended catalogs passed. No guest
scene was executed. Wine startup was blocked by the sandbox's read-only `/run`;
native Windows, PowerShell execution and GUI/gameplay compatibility remain
separate unverified checks. Evidence: `target/validation/units/` units
`debug-assets-standard-extended-346hs8wu`,
`debug-assets-standard-native-catalogs-tp13xnhk` and
`debug-assets-extended-native-catalogs-o03vwr58`.
