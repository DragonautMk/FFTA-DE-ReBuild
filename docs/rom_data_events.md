# European effect/event producer directories

The [source model](../crates/gba-core/src/rom_data/effect_event_directories.rs)
covers 75 event records, 51 effect selector records and two same-state dispatch
tables with 28 words. These four disjoint source ranges contain 1516 bytes and
do not overlap the 42 managed EUR e48 resources. The strict, lossless source
model initially left publication unchanged. Managed e49-d48 integration now
adds the resource as described below; USA stays e8-d7.

## Independent source boundaries

The event reader at `08124eb8` addresses 16-byte records from `3aca0c` using a
live u16 context index. Its advance path at `081253da..081253e6` increments that
index, stores its low halfword and compares the result with 74. Independently,
the six-byte catalog reader at `08126628..08126636` uses `3acebc`, exactly after
75 event records. Together these establish the declared 1200-byte projection.
Relative jumps and arbitrary live indices can leave it; this is no new guest
index guard or proof that all callers enter with zero context.

The four-byte selector reader at `08128b08..08128b2c` uses `3ad174`. The next
independently referenced source is `3ad240`, read by `0812c7fe..0812c80a` with a
three-bit index. The difference establishes 51 source rows/204 bytes. That
extent is not inferred from the separate task descriptor catalog's count.

The script frontdoor at `0812746e` converts its u8 parameter q into
`(q - 1) & 65535`. The helper truncates to u16, selects its four-byte row and
sets the secondary task ID to `(index + 363) & 65535`. Thus q=0 retains raw
index 65535 and task ID 362; q=52 selects the first row outside this source
projection. The model's address helpers reproduce this arithmetic. Runtime
integration must retain the existing owner/raw-ROM path outside each range.

## Fields and original control flow

Each event preserves these fields:

- Byte 0: opcode, compared with 21 before dispatch.
- Byte 1: raw field with no reader established here.
- Halfword 2: raw ID/operand, also read as a byte by one handler.
- Byte 4: post-event delay, zero-extended into the live context halfword.
- Byte 5 and halfword 6: raw fields with no reader established here.
- Words 8 and 12: raw argument bit patterns loaded before dispatch.

The four raw bytes per event total 300 bytes. They are preserved and source
authenticated; their observed zero values do not justify discarding them or
changing guest control flow. Arguments and IDs keep all bits, including values
later interpreted with signed arithmetic by the original handlers.

Each selector keeps its kind and three u8 parameters. Kinds 0..5 select the
original six handlers; parameters are read according to the handler and live
actor state. The model does not replace actor selection, allocation, messages,
event progression, waits or callbacks with host logic.

The event table at `124efc` contains 22 words and the selector table at `128b50`
contains six. These are **even Thumb same-state targets** consumed by `MOV PC`;
they differ from the odd BX callback words in task descriptors. Their model
reconstructs even words and keeps Thumb state. Opcodes above 21 and kinds above
5 use the original default branches.

An initial read-only candidate at `128b4c` was rejected: it is the LDR-PC literal
whose value points to the actual table at `128b50`. The source plan was written
only after this correction. A new static range guard was also corrected to
include the valid common-exit target `08125398`, four bytes past `08125394`.
Neither correction changed original source bytes or reopened an old proof.

Opcode 0 supplies 19 explicit secondary task references. Selector ordinals
0..50 supply another 51 references through the original ID transform. Both
resolve to the existing `engine.task-descriptors` script catalog; nullable
callback entries remain nullable. These are real asset links for the future
manifest dependency, not a source-size inference.

## Typed source validation

Strict JSON preserves canonical event/selector/dispatch indices, exact source
locations, descriptor references, raw fields and even ROM target offsets.
Unknown fields and invalid integer/array widths are rejected. Shape validation
allows raw opcode/kind values with original default behavior; authentication
still rejects any changed base field against the exact 16MB EUR image.
The source contract authenticates 92 halfwords and eight root/boundary words.

Independent Python reconstruction passes all fields, four hashes, 70 descriptor
references and disjointness against the 42 earlier owners. The affected Windows
release tests compile without warnings. One Wine source oracle passes on its
first run:

- 77 event header and 77 delay prefixes, including raw indices 75 and 65535.
- 53 selector address/kind prefixes and 43 kind-specific parameter loads.
- 32 dispatch/default cases, preserving Thumb state and unknown-value fallbacks.
- 19 frontdoor, low16, task-ID and increment/wrap prefixes.
- 12 shape, four coherent source-edit, nine JSON and two ROM-identity guards.

The original loadsites observe 1106 distinct owned bytes. The 300 event raw
bytes and 110 selector parameter bytes outside these declared load prefixes
remain reconstructed and hash checked. This is a probe limit, not global
unreachability. EWRAM/IWRAM and non-scratch registers are checked in the field
prefixes, except their declared output stores. The 2582 instructions and 70 PCs
describe coverage only; no performance measurements were run.

No direct Thumb BL or even/odd literal reference to event entry `08124e1c` was
found in the new targeted searches. Its ordinary reachability remains unknown;
these negative searches do not prove dead code. The source oracle deliberately
claims controlled field/dispatch/index prefixes, with no whole event interpreter,
allocator, scheduler, native execution, complete-state/clock or ordinary-scene ABI
claim. Linux, USA and native Windows checks remain pending.

Private evidence lives under `target/rom-data-event-directories/`, including
`source-contract.json`, `static-source-verification.json`, rejected-candidate
notes, retained disassemblies, `windows-model-tests.json`,
`windows-model-result.json` and `windows-model-europe/result.json`. Prior data,
executables, caches, saves, recordings and concurrent user changes are preserved.
Completed e48 and earlier source/native/lifecycle/ordinary checks were not repeated.

## Managed publication and typed guest reads

EUR e49-d48 adds `effect-event-directories.json` / `engine.effect-event-directories`,
file44/resource43, retaining all42previous decoded files and the e48generation.
Its real dependency is `engine.task-descriptors`; complete-set/source equality
and the immediately preceding owner are checked before activation. Four source
bindings reconstruct events/selectors/even dispatch words directly from typed
fields, without a raw ROM-byte shadow. Pages124/128/3ac/3ad enter the existing
ROM-owner path; other owners and raw bytes are merged at boundaries. Guest
indices, default branches, waits, actor selection, allocation, messages and
sequencing remain original. Save/native ABIs do not change. Aggregate category,
event/selector and dispatch-table counters live outside serialized machine state.

A warning-free Windows test build and two reduced Wine checks pass on the first
run. Managed validation covers13coherent/source-set/dependency guards, interrupted
publication/create/reuse/repair/move, e48retention, transactional missing-file
activation and ownership through unavailable files/Load/Undo/backups/Restart.
All1516typed bytes,75events/51selectors/two dispatch tables are checked with raw
source altered in the isolated machine.

The native check passes29original/interpreter/native complete-state/clock prefixes.
Generated API reads account for24event/6selector/24dispatch aggregate bytes.
Event index75, selector indices51/65535, q0, u16wrap, defaults and even Thumb
MOV-PC agree. Event index65535aliases the existing packed font owner and reads
11typed font bytes; it is deliberately not classified as raw ROM. A separate
same-page probe preserves four typed palette-program bytes through the newly
added3acpage. Four-span edges/all widths/ROM mirrors/real-BIOS policy/CpuSet/
FastSet/DMA and2typed+2raw mixed reads pass.186instructions/70PCs/29native dispatches
are coverage only. The existing prefix endpoint guard is reused; the production
emitter and source model stay unchanged. A later test diagnostic label correction
does not alter an executed success path and requires no repeated proof.

This is controlled field/dispatch/index integration, with no whole event
interpreter/allocator/scheduler ABI or event-function reachability claim.
Source-model acceptance above remains closed; no Linux/USA/native Windows or performance
measurement is added by these Wine checks.

## Production CLI and ordinary source-use limits

A warning-free Windows production build and reduced Wine CLI checks pass:
create/reuse without writes of46managed files, an unrelated working directory,
and loading the existing native cache. All44generation files match the accepted
e49dataset. The original canonical executable is restored byte-for-byte.

Three ordinary EUR locators finish successfully but read zero new event, selector
or dispatch bytes: retained Pecorelle1000, a clean boot through the school
courtyard at3000frames, and a genuine suspend boot-to-battle8476scene, all at
classic240x160. Their unchanged states
and input recordings are reused; no new navigation or fixture is created. They
are closed negative results, so none receives an event-required full-state/audio
pair. Positive ordinary selector consumption is established separately by the
later book transition below.

The original six-byte command directory at3acebc maps opcode59to127458 and
5ato12746c→128af4. Read-only, checksum/ROM-authenticated Pecorelle and F09 snapshots
have a zero queued-script root at020159d4. This narrows the search; it does not
prove absence of live script consumers. A retained USA candidate is excluded by
ROM identity before inspection/replay. No new USA verification is claimed.
Private CLI/locator results are recorded in `windows-cli-summary.json` and
`windows-cli-school-courtyard3000.json` under the evidence directory above.

A later8476-frame genuine EUR suspendboot-to-battle CLI locator, selected for
the script command owner, also reads zero event/selector/dispatch bytes. It is
the third closed negative for this event owner; it receives no event-required
fullpair. Positive command consumption in that scene does not establish event
or selector consumption; the later book transition closes that criterion.

## Positive ordinary book transition

The EUR ordinary event-**or**-selector requirement is complete on Wine. A new
normal-input chain advances from a cold boot through name confirmation, the
snowball tutorial and the street to the bedroom/book scene. Nineteen genuine
snapshots through frame39420 pass exact load roundtrips and recorded input/
artifact hashes. Existing saves, snapshots and inputs remain untouched. Two
private Windows capture helpers link the current prebuilt core; one also reads
the existing visible menu descriptor to issue recorded physical Down/A/neutral
keys for Wait. It restores the host detector flag before execution, never calls
mouse selection and never patches guest memory or save/state bytes. Initial
helper dependency/native-library search-path failures are retained and corrected;
the successful helper builds are warning-free. No product/core rebuild is needed.

The three earlier negative contexts stay closed. Two new bounded locators also
finish with zero family reads:3180tutorial frames and6000turn/transition frames.
Neither receives a fullpair. The black transition subsequently advances normally
to the winter street and home. Read-only current VM state then supplies a concrete
cue: key2/program1 cursor0bfb, before its sole command5a→selector50 atB95991.

One new1800-frame locator from the genuine frame39420bedroom snapshot is positive
with the actual CLI-published e52-d51 generation and existing native cache:
2bus reads/5aggregate source bytes =0event +1selector +4dispatch, one selector
record and one dispatch table. One matching Wine fullpair passes all1800full
states and clocks, central/wide images, OAM, dispatch and mixed/PSG/FIFO/bitwise-HLE
audio. Sample counts are[3950100,3950100,475200,475200,3950026]; source use first
occurs at absolute frame39946 and matches the locator. All470hash-mtime guards
pass. Frozen CLI/test executables and the dataset are reused, without preparation,
old acceptance repeats or performance measurements.

This establishes ordinary selector consumption. Event rows remain ordinary0;
other50selectors/the other dispatch table retain targeted coverage. Entry124e1c
reachability, whole callback/allocator/scheduler ABI and a new32:9scene are not
claimed. New Linux/USA/native Windows checks remain pending. Evidence under
`target/rom-data-event-ordinary/` includes `accepted-scene-acquisition-lineage.json`,
`home-selector-locator.json`, `windows-scene-result.json` and
`windows-event-full-state-audio.log`.
