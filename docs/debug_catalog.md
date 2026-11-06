<a id="catalogo-dei-casi-di-debug--dbg-03"></a>

# Debug case catalog — DBG-03

The browser reads `catalog.json` from the existing local collection. Menu and
CLI use the same projection: name, region, source slot, variant, provenance,
recorded status, test scope and file integrity. The collection and its
classifications are not rewritten.

**DS identifies a test case; IN and GEN identify sources; a slot is a separate destination.**
A DS-20 checkpoint can be imported into slot 2 of a copy. Variant aliases may
share bytes without becoming new DS evidence. DS-03 and DS-15 remain uncovered
when they have no checkpoints.

<a id="dal-gioco"></a>

## From the game

To use the project's collection directly:

```powershell
.\recomp.exe play 'C:\Games\FFTA EUR.gba' --no-bios --interp `
  --debug-catalog 'C:\FFTARecomp\debug_saves\ready\catalog.json'
```

Open **Debug → Debug catalog**. Without the flag, the menu looks for
`debug-catalog/catalog.json` in the configuration directory; in portable mode
this is `Data/config/debug-catalog/catalog.json`. If absent, the integrated
content selector explains what to select. Use **Choose / reload catalog** to
select a catalog; external file and folder dialogs require an explicit action.
Copying a collection requires including the files referenced by the JSON.

The local **`IN-04/EUR`** entry is the user-supplied cartridge save described as
before the final battle, with movement starting it. The user confirmed EUR
compatibility. Select it in the catalog, or use **Game saves Slot 4** in the EUR
collection (`EUR/ffta-slot-04.sav`). Its unchanged master and provenance are in
`debug_saves/user-supplied/`. It shares bytes with IN-04/USA; the scene has not
been independently replayed, and DS-26 retains its partial status. Cartridge
save slots and snapshot slots are separate.

The user's three EUR final-battle snapshots are retained separately in the
ignored local `debug_saves/user-supplied/royal-valley-final-2026-11-06/`
directory, with a private `catalog.json` and provenance record. The entries
`USER-FINAL-EUR-SLOT-2`, `-3` and `-4` describe, respectively, the reported
starts of phases two and three and the victory scene before the credits. Select
that catalog explicitly to open an isolated case, or use the same keys with
`debug-catalog open`. Full ROM/BIOS state loading and isolated case creation
were checked without running the game. The scene descriptions are the user's;
the snapshots retain cheat ancestry and are not ordinary-play evidence.

The [private debug asset export](private_debug_assets.md) also builds an accessible
projection of the collection's loaded checkpoints and EUR base, preserving the
original catalog separately. Its `DEBUG-ASSETS.html` index links additional
root-level `DEBUG-CATALOG-EUR/USA-*.json` pages for discovered workspace assets.
Use **Choose / reload catalog** to select a page, then clear the filters to All.
Each page stays within the existing catalog limits. Discovery and envelope
integrity do not establish gameplay validation; raw invalid/unidentified files
remain available through the index with their limitations.
Default export omits intermediate numbered micro-snapshots while preserving
collection masters and recording/report inputs. Use `--all-snapshots` for the
integral collection. Raw/preview copies of the same core share one discovered
catalog item, and the index initially shows distinct saves with source aliases.

Up/Down selects a row. Left/Right or Confirm changes the filter or entry:

- **Region** starts with the active ROM; All includes other regions. Cases
  without checkpoints remain visible even under the EUR or USA filter.
- **Recorded status**, **Kind** and **DS case** filter entries without changing
  test status. Status labels follow the menu language; stored values such as
  `validata` and `non_coperta` remain unchanged. The DS filter does not interpret
  IN-20 as DS-20.
- **Entry** cycles through results. The panel shows a preview when available,
  provenance, source slot and test scope. A legacy raw file without a preview
  shows No preview; no capture is fabricated.
- **Open isolated case** creates and opens a [portable copy](debug_sessions.md).
  Checkpoints restart from the selected snapshot; standard IN/GEN saves start
  with normal boot using that SRAM. ROM and BIOS must be compatible;
  an already active dataset is copied using the normal checks.
- **Import to free snapshot slot** is available inside an isolated copy.
  It copies the exact checkpoint into the first free slot and opens Save states
  on that slot. The game is not loaded automatically: choose Load state.
  Import can be undone with Undo last operation, like a new save.

The normal window stays paused while the copy is in use. Close the current
copy before opening another session; other compatible checkpoints can instead
be imported into free slots. From there, **Debug → Start input recording**
starts a [snapshot recording](debug_recordings.md) with relative frames and a
verifiable endpoint; the catalog and test-case status remain unchanged.

Each copy opened from the catalog retains `catalog-entry.json` with provenance
and a local `debug-catalog.json` referencing the existing seed. The entry keeps
its original key after relocation and requires no third snapshot copy. The menu
can therefore reopen the selected entry's catalog without finding the external
collection. Choose / reload can select the complete collection again. For a
standard save, the local entry checks the current SRAM copy: if the game
changes it, the difference from the initial checksum remains visible.

<a id="dalla-cli"></a>

## From the CLI

The versioned JSON contains `schema = 1`, `kind = ffta-debug-catalog` and
the filtered entries. PowerShell examples:

```powershell
.\recomp.exe debug-catalog list '.\debug_saves\ready\catalog.json' `
  --case DS-20 --region EUR --kind case --json

.\recomp.exe debug-catalog list '.\debug_saves\ready\catalog.json' `
  --status non_coperta --json

.\recomp.exe debug-catalog open '.\debug_saves\ready\catalog.json' `
  --entry 'DS-20/EUR/20' --rom 'C:\Games\FFTA EUR.gba' `
  --dir 'C:\Debug\DS-20' --no-bios --interp --launch
```

`open` requires a new destination and a single entry. Without `--launch`, it
prepares the copy and returns the folder without running the game. `--bios FILE`
and `--no-bios` are alternatives. `--dataset DIR` copies an optional authenticated
dataset. A DS selection may include multiple regions or variants: use the exact
`--entry` key reported by list to remove ambiguity.

To import into a **closed** copy:

```powershell
.\recomp.exe debug-catalog import '.\debug_saves\ready\catalog.json' `
  --entry 'DS-20/EUR/20' --session 'C:\Debug\DS-20' --json
```

The result identifies the chosen slot. `--slot N` selects an explicit
destination, always between 1 and 99 and always free. The CLI uses the session's
pinned ROM and BIOS; it does not accept startup overrides. Above slot 9, the
result requires Extended saves: enable it in the copy's menu. CLI import does
not alter configuration. An open session is protected by its exclusive lock.

`list` without `--json` prints a summary. A missing or changed file is reported
in entries with `integrity = invalid`, `available = false` and the cause;
the CLI exits with code 1 if selected results contain such errors. An uncovered
case, which has no file by definition, is not corruption. Other operations exit
with code 1 on incompatibility, ambiguity or an occupied slot. Linux uses the
same arguments with `./recomp`.

<a id="integrità-e-limiti"></a>

## Integrity and limits

Loading accepts the private schema-1 catalog, bounded to 4 MiB and 1024 entries.
Only `file` paths are opened, within the collection; provenance paths are
metadata. Snapshots also undergo envelope, preview and payload checksum checks.
Opening and importing recheck bytes and use the full validator with the
selected ROM/BIOS before writing. An occupied or concurrently created slot
is not overwritten.

`available` and `integrity` identify artifact availability and integrity;
alone they do not establish full compatibility or scene reachability.
Recorded status and scope remain historical. DS checkboxes are not updated,
checkpoints for DS-03/15 are not invented, checksums are not repaired, and RAM,
flags, inventory and source files are not changed. Catalogs, copies and evidence
remain private and outside Git and packages, like the original collection.

<a id="verifiche-del-28-ottobre-2026"></a>

## Validation on 28 October 2026

Offline Windows release build on frozen public base `2fbf9f7`, with only the
tool files overlaid and a separate Cargo cache. The local candidate is
`target/debug-tools-catalog/recomp.exe`, SHA-256
`d55a7e115f9fd1ca924e6f1b2f3e709818df8b24ea4a194bda69eedf2b3dfccd`.
Normal Windows/Linux executables and concurrent extraction work are preserved;
the candidate is not installed in their place.

- **61 tests under Wine:** 8 catalog, 43 menu and 10 slot tests. They cover
  regional filters/status, distinct identifiers, checksums/envelopes, paths,
  compatibility, exact import without loading, Undo and overwrite refusal.
- **15 real CLI cases:** 92 collection entries, DS-20 EUR selection, portable
  opening, free import and explicit import into slot 99; refusal of ambiguity,
  occupied slot, standard save used as snapshot, fileless case and changed files.
- **Wine/Xvfb GUI:** DS-20 selection, refusal to import into the normal game,
  opening the copy with an identical seed, local catalog, exact import into
  slot 2, preview and explicit Load, closing the copy and returning to the parent.
  Parent state/clock/progress and hashes/mtime of the 140 protected inputs unchanged.
  An initial incorrect test navigation is retained with logs; the correction
  concerns only the local script. Final screenshots were inspected.

Local results: `test-results.json`, `cli-results.json`, `cli-summary.json` and
`gui-results.json` under `target/debug-tools-catalog/`. Native Windows, Linux,
physical controllers and native gameplay in the copy are not verified by this
unit. No new DS acceptance, performance test or ROM extraction.

DBG-08 produces a [capture bundle](debug_captures.md#riutilizzare-il-punto) with
local `debug-catalog.json`, snapshot, origin and authenticated input. Open it
with the picker or `debug-catalog list/open/import`. The entry has key
`CAP/<run>`, variant kind and status `variante`; the DS name remains provenance
and is not promoted to a validated case. The personal catalog remains unchanged.
