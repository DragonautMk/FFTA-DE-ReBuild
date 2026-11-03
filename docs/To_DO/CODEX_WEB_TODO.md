<a id="fftarecomp--todo-per-codex-web"></a>

# FFTARecomp — Codex Web TODO

**Status:** planning; no item is considered complete solely because it appears in this document.

**Repository:** `DragonautMk/FFTARecomp`

**Reference branch:** `decomp`

**Objective:** develop tools complementary to recompilation and ROMDATA extraction through independent cloud assignments and reviewable pull requests.

> **Instruction for Codex Web:** this is a roadmap, **not** an order to implement everything in one session. Choose the first open unit compatible with the current code, finish it and propose a PR limited to that unit. Before each assignment, reread the updated branch: local Codex state may be ahead of GitHub.

<a id="0-confini-del-lavoro-e-regole-obbligatorie"></a>

## 0. Work boundaries and mandatory rules

- [ ] Read `AGENTS.md`, `README.md` and the effective development instructions on the branch.
- [ ] Consult `docs/development/ROM_DATA_TODO.md`, `ROM_DATA_INVENTORY.md`, `TASK_HANDOFF.md`, `DEBUG_TOOLS_TODO.md` and the relevant family contracts read-only.
- [ ] Identify features already implemented before designing alternatives: also check `tools/`, `scripts/`, `docs/report_home.md` and existing modules.
- [ ] Record the starting commit SHA, branch and files to be changed.
- [ ] Create a dedicated branch for **one** work unit; prepare a PR against `decomp`, without automatic merge.
- [ ] Do not change without specific authorization: original extractors/decoders, `rom_data.rs`, runtime readers, source map, manifest, ROMDATA revision, guest algorithms, ABI, saves, native recompilation code and already accepted equivalence tests.
- [ ] Do not change `TASK_HANDOFF.md` or shared checklists managed by local work; update only documentation directly related to the PR and this checklist when the project owner integrates it.
- [ ] Do not upload or commit ROMs, BIOS, saves, recordings, dumps, extracted datasets, original assets, private cheat catalogs or unauthorized third-party material.
- [ ] Use reproducible **synthetic fixtures** with fictitious data. Do not require the ROM to be made public to develop or test a tool.
- [ ] Do not call something “verified on the game” when tested only with synthetic data; distinguish compilation, unit tests, real-dataset tests and runtime parity.
- [ ] Run only tests relevant to the unit, without performance campaigns or ROM/Wine checks already completed by local Codex.
- [ ] Avoid operations that delete user data or implicitly rewrite existing files. Keep private outputs separate and ignored by Git.

<a id="p0--prima-attività-consigliata-esploratore-e-wiki-offline-romdata"></a>

## P0 — Recommended first activity: offline ROMDATA explorer and wiki

**Purpose:** turn an already extracted dataset into browsable documentation without interfering with extraction. **This is the first activity to propose to Codex Web.**

<a id="cw-01--analisi-e-contratto-del-generatore"></a>

### CW-01 — Generator analysis and contract

- [ ] Inspect the actual manifest, JSON/TOML schemas and documented identities/versions on the current branch.
- [ ] Choose **one or two** families with sufficiently verified formats and relationships (for example objects and links), explaining the choice; do not necessarily start with the latest still opaque records.
- [ ] Define an independent CLI and a module in `tools/` or an equivalent location already consistent with the repository.
- [ ] Establish input, output, size limits, behavior on missing data and offline operation; do not duplicate ROM extraction logic.
- [ ] Write a short contract: which fields are semantically verified, which are identifiers only and which remain opaque.

**Completion:** short technical document with involved files, a synthetic example, supported schema and proposed command; no core changes.

<a id="cw-02--mvp-wiki-statica"></a>

### CW-02 — Static wiki MVP

- [ ] Read and validate an already extracted local dataset without ROM access.
- [ ] Generate a static HTML homepage and an index of the families actually present.
- [ ] Generate browsable pages for selected families with stable IDs, documented fields and cross-references **only when actually demonstrated**.
- [ ] Add local search/filtering, navigable links and a visible indication of unknown or incomplete fields.
- [ ] Report dataset provenance, version and hash, distinguishing them from the generator version.
- [ ] Escape every string inserted into HTML; protect against path traversal, unsafe links, malformed JSON and oversized files.
- [ ] No online uploads, telemetry, CDN or remote dependency required to browse the output.
- [ ] Prepare tests with synthetic fixtures, including errors and missing relationships.
- [ ] Document a usage command for Windows and Linux, expected output and limitations.

**Completion:** HTML site generated locally from fixtures, navigable pages, passing tests, no original assets in the commit. Real-data testing **remains to be done locally**.

<a id="cw-03--estensione-futura-della-wiki-non-automatica"></a>

### CW-03 — Future wiki extension (not automatic)

- [ ] Progressively add missions, units, encounters, jobs, abilities, maps and scripts **only when their schemas and links are proven**.
- [ ] Distinguish `identificato` (identified), `integrato nel runtime` (integrated into the runtime), `osservato in un test mirato` (observed in a targeted test) and `osservato in gameplay ordinario` (observed in ordinary gameplay).
- [ ] Link available evidence and source offsets without inventing field meanings.
- [ ] Support schema changes and incomplete datasets with explicit messages, without silently correcting them.
- [ ] Allow tabular exports (CSV/JSON) without changing the source dataset.

**Completion:** one PR per extension; tests for new families and backwards compatibility verified against earlier fixtures.

<a id="p1--esportazione-degli-asset-strumenti-separati"></a>

## P1 — Asset export (separate tools)

<a id="cw-04--convertitore-grafico-offline"></a>

### CW-04 — Offline graphics converter

- [ ] Study the decoders actually present (`obj_graphics`, `bg_graphics`, palettes, fonts, animations) and document what can already be reconstructed without guessing layout or transparency.
- [ ] Create pure conversions verifiable against **synthetic** tiles/palettes for supported, actually demonstrated formats.
- [ ] Generate PNGs and provenance metadata only when the tile + palette + layout combination is established with certainty.
- [ ] Preserve original palette indices, colors and references in a metadata file; do not replace ROMDATA with PNGs.
- [ ] Handle errors, out-of-range indices, unusual dimensions and unproven transparency without arbitrary assumptions.
- [ ] Keep spritesheets and animation reconstruction as later extensions, with frames/timing verified separately.

**Completion:** synthetic graphics fixtures converted with exact pixels and automated tests; no claim of complete coverage of game sprites.

<a id="cw-05--convertitore-audio-offline"></a>

### CW-05 — Offline audio converter

- [ ] Study the PCM samples and related parameters actually decoded in `audio_tones`.
- [ ] Export WAV from synthetic samples with explicitly defined rate, signedness, length and channels.
- [ ] Preserve any loops and instrument/tone references in metadata.
- [ ] Do not call an isolated sample a “complete track”; rendering M4A sequences requires dedicated work and validation.
- [ ] Verify waveform, WAV headers and sizes with deterministic tests.

**Completion:** a WAV converter for the demonstrated subset, documented and tested with synthetic fixtures.

<a id="p1--fondamenta-del-catalogo-cheat-senza-modifiche-live-alla-memoria"></a>

## P1 — Cheat catalog foundations, without live memory changes

<a id="cw-06--catalogo-e-modello-dei-cheat"></a>

### CW-06 — Cheat catalog and model

- [ ] Examine only the public interface and available contracts; **do not** import the private local catalog of third-party codes into the cloud.
- [ ] Define a versioned model of cheat entries with name, description, region, format, provenance, requirements and verification status.
- [ ] Distinguish simple write codes, conditional codes, device-specific formats and master codes; do not treat them all as address/value pairs.
- [ ] Implement search, filters, import/export and limited parsing with entirely synthetic codes.
- [ ] Reject unsupported or ambiguous codes; do not execute arbitrary expressions or catalog-provided scripts.
- [ ] Produce a reusable interface or CLI prototype **without** enabling RAM/ROM writes in the runtime.
- [ ] Document the future contract with isolated sessions, replay, auditing and change provenance.

**Completion:** catalog and parser with positive/negative tests, no guest state changes. Any cheat engine and in-game menu require subsequent **local** integration with interpreter/native parity tests and deterministic write logs.

<a id="p2--report-e-qualità-del-progetto"></a>

## P2 — Reports and project quality

<a id="cw-07--report-diagnostici-unificati"></a>

### CW-07 — Unified diagnostic reports

- [ ] First check what already exists in `docs/report_home.md`, DBG tools and their HTML/generator; **do not create a second duplicate index**.
- [ ] Identify a single bounded gap (e.g. readable summary export, identity filter, links to existing reports).
- [ ] Extend the existing implementation only if the change is independent of the runtime and testable with synthetic JSON reports.
- [ ] Clearly show `pass`, `divergente` (divergent), `non eseguito` (not run), `parziale` (partial), `troncato` (truncated) and `incompatibile` (incompatible) without automatically promoting evidence.
- [ ] Keep local paths and game attachments private; no automatic upload.

**Completion:** a small PR with parsing/HTML tests and verified offline output.

<a id="cw-08--test-di-robustezza-e-integrazione-continua"></a>

### CW-08 — Robustness tests and continuous integration

- [ ] Identify a real test coverage gap in the new tool; do not repeat accepted ROMDATA tests.
- [ ] Add synthetic fixtures for valid, truncated, inconsistent and hostile input and incompatible versions.
- [ ] Create reproducible automated commands requiring no ROM, Wine, BIOS, private data or external services.
- [ ] If useful, add CI limited to the new tools, with minimum permissions and without publishing game artifacts.
- [ ] Document the limits of cloud checks relative to real local tests.

**Completion:** reproducible cloud execution, passing tests and logs without game data.

<a id="flusso-operativo-obbligatorio-per-ogni-incarico"></a>

## Mandatory workflow for each assignment

1. [ ] **Reread:** update the base branch, read contracts and check that no other change has already completed the assignment.
2. [ ] **Short plan:** propose a minimal solution, touched files, dependencies and tests; if the unit requires ROMDATA/core/ABI changes, stop and report that instead of changing them implicitly.
3. [ ] **Bounded implementation:** work on a dedicated branch with minimal changes, without mass reformatting.
4. [ ] **Verification:** run unit tests and negative cases with synthetic fixtures; check the diff, unexpected files and generated artifacts.
5. [ ] **Documentation:** write a usage command, fictitious examples, limits and tasks reserved for local testing.
6. [ ] **Delivery:** create a PR against `decomp` with a summary, changed files, test results, risks, checks not run and instructions to test a real dataset **without committing it**.
7. [ ] **Local integration:** after review, test on the computer with private data; merge only when compatible with ongoing extraction.

<a id="ordine-di-esecuzione-consigliato"></a>

## Recommended execution order

1. **CW-01 → CW-02:** first offline ROMDATA explorer/wiki. *(Start here.)*
2. **CW-04 or CW-05:** isolated graphics or audio exporter; choose the one with the more complete contract.
3. **CW-06:** cheat model and catalog, without cloud runtime manipulation.
4. **CW-07:** only a real gap in existing reports.
5. **CW-03/CW-08:** wiki extensions and tests/CI for tools already created.

**Do not start concurrent assignments on the same files.** Do not replace local extraction work with new ROM analysis unsupported by real tests.

<a id="prima-richiesta-da-incollare-in-codex-web"></a>

## First request to paste into Codex Web

> Read `CODEX_WEB_TODO.md` and `AGENTS.md` on the `decomp` branch of `DragonautMk/FFTARecomp`. Perform **only CW-01**, the contract and minimum architecture for the ROMDATA explorer/wiki; analyze the actual schemas before selecting the first families. Do not touch extractors, the runtime or local Codex checklists. Use only synthetic data and prepare a separate pull request with the plan and checks possible in the cloud environment. Do not proceed autonomously to later units.
