# European task descriptor catalogs

The source model in
[task_descriptors.rs](../crates/gba-core/src/rom_data/task_descriptors.rs) covers
three bounded EUR catalogs:341primary records at3997b0,20auxiliary records at
39a258, and51script records at39a2f8. Each8Brecord contains a nullable Thumb
callback pointer, a u16context size and a raw u16metadata field. The412records
occupy3296B in three adjacent source spans, disjoint from the41managed e47owners.
There are268distinct nonzero callback words and three null records. Shared
callback addresses remain explicit; no callback code is extracted into this model.

## Source extent and original readers

Primary constructor080d7018 truncates the input ID to u16. For IDs at most341,
it reads callback/context size from3997b0+(ID-1)*8; for higher IDs it uses
39a258+(ID-342)*8. ID0 therefore reads before the projection. There is no lower
bound check and no upper bound in the higher branch. The special ID345path also
reads a primary record's +6metadata halfword selected by live R6 and copies it
to message+16; it copies R6 to message+18 and overrides callback/context size
with080e5ea9/40. The source model leaves this guest control flow in place.

Secondary constructor080d7328 uses39a2f8+(ID-363)*8 after the same u16truncation.
Consequently primary ID362 and secondary ID363 share the first script record;
the last script record has primary ID412/secondary ID413. Lower secondary IDs
can alias the earlier physical catalogs. A single linear task-ID interpretation
would lose this difference. Model address helpers reproduce both original
wrapping formulas; projection membership never caps the live guest ID.

Two independent facts establish the suffix endpoint beyond pointer shape.
Original routine080d9b56 passes39a490as graphics source to08011434; it is a
different structure, outside these catalogs. Separately, directory row2at3aca2c
contains kind0/ID413. The original16Bdirectory reader at08124eb8..08124ec2and
kind0path08124f54..08124f80forward that ID to the secondary constructor, selecting
39a488, exactly the last8Brecord before39a490. This defines the bounded source
projection; it does not prove that all dynamic callers constrain their IDs.
The earlier456record hypothesis extending through39a5f0is rejected.

Reader080d76a8 independently truncates its selector, computes primary
(ID-1)*8, and reads byte+6for comparison with0x10. Both metadata bytes are kept
without invented flag meanings; the suffix has142zero metadata bytes with no
direct typed metadata consumer established in the declared catalog paths.
Guest indices outside a source projection must retain the ordinary owner/raw
ROM path. Context memory, allocation, message handling and scheduling stay live.

## Typed source model

Three fixed named catalogs preserve physical offsets, canonical row ordinals,
nullable ROM Thumb offsets, context sizes and every metadata bit. Strict JSON
rejects unknown fields and invalid integer widths. Shape validation checks table
locations/counts/ordinals/Thumb offsets; source verification authenticates the
exact16MB EUR image and compares the complete decoded model. Independent
source spans reconstruct the original bytes and record SHA-256provenance.
Original selector/load/boundary/directory fingerprints guard the contract.

The source/model unit left EUR e47-d46and USA e8-d7unchanged. The subsequent
managed integration below adds the EUR resource independently. Full allocator/
callback/scheduler ABI, other task catalogs and Linux/USA/native Windows checks
remain pending. No performance measurement is part of this work.

## Reduced Wine source validation

The affected Windows release test build completed without warnings. One combined
Wine check passes independent fields, three source hashes, strict JSON and exact
roundtrip, with10shape/4source-edit/7JSON/2ROM-identity rejection cases. Original
instruction prefixes pass415primary loads (including412projected records),
59secondary loads,346metadata halfword loads/copies,346metadata byte loads,
18low16truncations and9branch decisions. All nullable/aliased callback words and
context sizes match the original loads. RAM, stack and registers outside the
specified scratch/output effects are checked in the field/metadata prefixes.

The original loadsites observe3156distinct owned bytes. That includes all callback/
context fields and all primary metadata, plus the first suffix metadata halfword
read through an explicit raw alias probe. The other140suffix metadata bytes stay
losslessly reconstructed and hash checked; this is a limit of these probes, not
a proof that arbitrary live guest selectors cannot read them.7191instructions/
39PCs describe coverage only. No full allocator/callback/scheduler, native code,
complete-state/clock or ordinary-scene claim follows from this source check.

Private evidence is retained under `target/rom-data-task-descriptors/`:
`source-contract.json`, `static-source-verification.json`, original disassemblies,
`windows-model-tests.json`, `windows-model-result.json` and
`windows-model-europe/result.json`. ROM, prior generations, production executable,
cache, saves and earlier validation artifacts are preserved. Earlier completed
blend/palette/audio proofs were not repeated. Linux/USA/native Windows checks
remain pending; no new performance measurements were run.

## Managed task descriptors and typed guest reads

EUR e48-d47adds `task-descriptors.json`, resource `engine.task-descriptors`, as
file43/resource42. All41previous decoded files and the e47generation are retained;
USA remains e8-d7. The resource has no decoded-asset dependency: shared Thumb
entry representation is Rust code reuse. Extraction source-set coherence still
requires the immediately preceding blend owner to keep fixed manifest indices
consistent before any file check.

The runtime binds the three authenticated source spans to the typed descriptor
fields. It reconstructs callback words/context sizes/metadata directly, without
a ROM-byte shadow. Guest selectors, allocator/callback/message/scheduler work,
nulls, copied RAM and both constructor formulas stay original. Byte/halfword/word
reads merge prior owners and ordinary ROM at boundaries. Pages399and39aenter
the existing decoded-ROM path; native API, CPU, SWI and DMA use that same owner.
Counters track aggregate callback/context-size/metadata byte reads, distinct
records and catalogs outside serialized machine state. Save/native ABIs stay
unchanged. The resource participates in complete-set authentication and atomic
preparation/reuse/repair, with all files resident after activation.

Affected Windows release tests compiled without warnings. Two reduced Wine checks
pass on the first run:13coherent authentication/source-set guards, interrupted
publication/create/reuse/repair/move, all41previous decoded files/e47generation,
missing-file transactional activation, resident ownership through unavailable
files/Load/Undo/backups/Restart, and all3296typed bytes/412records/3catalogs.

The second check passes29original/interpreter/native complete-state/clock
prefixes with all3296raw source bytes altered in decoded arms. Generated native
API reads account for44callback/22context-size/9metadata aggregate bytes. Raw
ID0/65535/main413/secondary414, cross-catalog aliases, low16truncation and the
special metadata-copy/override prefix agree. An out-of-projection ID457keeps the
preceding blend owner and reads6typed bytes even when its raw source is altered.
Three catalog boundaries/outside edges, all widths/ROM mirrors/real-BIOS policy,
CpuSet/FastSet/DMA and2typed+2raw mixed reads pass.206instructions/41PCs/29native
dispatches describe coverage only. The previously corrected prefix endpoint
guard is reused; the production emitter stays unchanged. No whole allocator/
callback/scheduler ABI is inferred. Source-model Wine validation above stays closed. Actual production CLI and
ordinary acceptance are now complete in the declared scene scope below.

## Production CLI and ordinary scene

The Windows production CLI builds without warnings; the canonical executable is
restored byte for byte. From an unrelated working directory, actual Wine CLI
preparation creates e48then reuses all45managed files without changing hashes or
mtimes. All43generation files match the accepted managed dataset, and the existing
native cache loads. The ROM/cache/loader/state/input and prior datasets remain
unchanged.

The authentic EUR Montblanc Richiamo/Pecorelle scene reuses the retained confirm
checkpoint/input, with Aat32/release44,1000frames and classic240x160. CLI reports
11aggregate bus reads/28source-byte reads:16callback/8context-size/4metadata bytes,
4distinct records in2catalogs. These are aggregate byte reads, not28distinct
source bytes. One Wine pair using the actual CLI-published generation and the
already-built current test executable passes every complete state, clock,
central/wide framebuffer, OAM, dispatch count and mixed/PSG/FIFO/bitwise HLE audio
sample across all1000frames. Audio sample counts are2194500/2194500/264000/264000/
2194366; they describe comparison coverage, not performance.

All declared source/model/managed/native/CLI/ordinary children for this bounded
family are closed. Other408records and the third catalog retain targeted-only
coverage; no whole allocator/callback/scheduler ABI or new32:9scene claim follows.
Linux/USA/native Windows and the wider ROM_DATA_TODO remain pending. No navigation,
fixture recreation, guest RAM/state-payload/save edit or old-family proof repeat
was needed. Evidence includes `windows-build.json`, `windows-cli-processes.json`,
`windows-cli-no-write.json`, `windows-cli-summary.json`, `windows-scene-result.json`
and `windows-pecorelle-task-full-state-audio.log` under the private evidence root.
