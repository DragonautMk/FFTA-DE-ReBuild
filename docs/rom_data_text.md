# European text and font resources

The broad offline model now covers all nine pointer-string selectors and all
29relative-archive selectors found in the declared original reader block,
across five language IDs. Current e12prepares/authenticates and directly consumes
the broad tenth file. The earlier overlapping ninth file stays authenticated;
the full catalog owns its reads and counters.

## Broad pointer catalogs and relative text archives

`text_catalogs::Catalog::from_rom` authenticates the exact EUR image and uses
the original reader literals recorded in `text_roots.rs`. The nine pointer
selectors `08016588..08016888` have767/753/107/725/92/512/52/128/84entries per
language. Every table's end meets the next independently selected payload
start; no pointer/header sweep defines the contract. Across five language IDs
there are16100distinct source strings,10735compact/5365extended and229314source
bytes. The model reuses the proven short-string byte/code representation and
preserves all45index tables and exact source pointer identity.

The29root selectors `080168e8..08017288` choose145relative archives. Original
`08013f8c` indexes little-endianu16offsets relative to the archive root, then
reads a16-bit program header. The first offset bounds the index table; the
final`ffff`is explicit sentinel metadata with no program. There are17035programs:
6138raw extended,8898custom-LZ extended and1999raw compact. Their exact source
extents total1766190bytes; alignment between programs remains outside those
extents. All33581pointer/archive/font provenance ranges are disjoint.

Program headers retain every bit. Bit0selects compact encoding; bit1selects
original custom LZ/RLE `080054a0`, shared with graphics. Program units retain
game glyph IDs and original command opcodes/operand bytes. Compact scripts use
byte glyph IDs and`ff`termination; extended scripts useBEglyph codes and zero
termination, with commands introduced by`40`. Command parsing follows original
`08014010`/`0801416c`: zero operand bytes remain inside commands, and command53
retains two embedded zero-terminated byte strings. Dynamic substitutions remain
commands; names/unit context/flags are not baked into extracted text.

Compressed programs retain their exact original encoded representation,
decoded units and any byte after the logical terminator.2012streams end at the
terminator and6886have one further decoded byte, retained without interpretation.
Validation reconstructs command/glyph bytes and checks them against bounded
custom decoding. It refuses truncated operands, invalid encoding/commands,
bad index order/sentinels, overlapping extents and encoded/typed mismatches.
The combined catalogs reference128glyph IDs1..486; complete128-byte packed
slots and variable metrics are retained without claiming Unicode or all possible
font slots. The earlier65glyphs remain independently covered.

Two new reduced Windows/Wine checks pass after warning-free release compilation:
a command/codec guard check and one exact-EUR catalog/original-reader oracle.
Strict deterministic JSON/source authentication matches independent retained
static classifiers for all pointers/indices/source extents/hashes, and rejects
coherent metric edits and malformed index metadata. Newly covered pointer
catalogs use80first/last boundary reader/copy cases, preserving output/guards;
the closed107-entry all-string oracle is not repeated. All145archive root
selectors match, and300archive cases cover every archive/language boundary,
three encoding/compression forms and all21command forms present in the source.
Original output/guards match a script reconstructed from decoded units; the
original uncompressed return adjustment above511bytes is checked separately.
All63new glyph slots/metrics match the original copier. Total968calls and
867138guest instructions are coverage counts, not performance measurements.

Dynamic command80requires a valid active unit/mission context. The retained
pub snapshot has none: index0underflows to`ffff`at original08013dd2, causing the
initial oracle to read outside its selected archive. The final fixture uses
the original nullable-context fallback with one-based index1for both original
and rebuilt programs. This is an explicit reader fixture, not an ordinary
gameplay/scene or full-state/clock-equivalence claim. The failed diagnostics
remain retained; decoder/model/bounds were not relaxed. Only the failed new
oracle was repeated, while the passed codec check remained closed.

Evidence: `target/rom-data-text-catalogs/reader-roots.json`, `pointer-catalogs.json`,
`archives.json`, original bounded disassemblies, `windows-core-archive-codec.log`,
`windows-results.json`, `windows-europe/result.json` and typed `text-catalogs.json`.
Original ROM/slot/cache/loader/canonical hashes remain preserved.
Broader ordinary scenes for this broad model,
all font slots/Unicode and other reader catalogs remain open. New Linux/USA/
native Windows checks remain pending, with no performance or space campaign.

## Broad managed catalog

EUR schema1/extractor12/decoder11publishes fixed tenth file `text-catalogs.json`,
resource `text.catalogs`, format `text-catalogs-v1-json`, with33581source ranges
and dependency `text.short-strings`. The complete typed base is compared against
the authenticated ROM before activation, including indices, headers, commands,
operands, encoded streams/padding and glyph words/metrics. Whole-set identity
includes the file. Earlier e11and all eight prior decoded files are retained
byte-for-byte; USA keeps six-file e8-d7. Guest/native/save identity is unchanged.

Readable JSON is retained without packing. Its malformed-input guard and the
whole dataset guard are512MiB, manifest16MiB. The first128MiB guard refused the
actual readable file before publication; it was widened, not used as a target
for compression or space optimization. Previous evidence is retained separately.
Three reduced Wine checks cover tenth-file deterministic publication/reuse,
interrupted-write/missing repair and the exact EUR source/whole-set identity,
coherent edits/metadata/path/coverage/dependency/truncated/oversized rejection,
corruption repair, folder move and transactional activation. Only the failed
real-EUR guard case was repeated; passed synthetic checks stayed closed.

Evidence: `target/rom-data-text-catalog-managed/windows-tenth-publication.log`,
`windows-atomic-write-repair.log`, `windows-results.json` and
`windows-europe/result.json`. At that managed-only boundary it was `verified-only`.
The actual production CLI prepare/no-write reuse and one-frame old-native-cache
activation also pass from an unrelated working directory; see
`windows-cli-summary.json`. The canonical executable is restored and all
protected originals/e11files remain unchanged. Managed publication alone did not
prove gameplay consumption; the direct unit below now closes that boundary.
Broader scenes and new Linux/USA/native Windows verification remain open.

## Broad direct consumption

The immutable owner retains the complete tenth-file model. Source bytes for
pointer tables, compact/extended strings, relative indices/`ffff`sentinels,
program headers, raw program glyph/command/operand bytes and font words/metrics
are reconstructed from typed fields. Compressed custom-LZ literal spans map
back to typed decoded program units/terminator/padding; exact size/token bytes
remain original file controls. Small unit offsets/literal maps hold locations,
not a second flat decoded script. Dynamic command resolution, original copy/
decode/render work, every guest instruction and clock stay unchanged.

Fixed declared EUR pages are`495..498`,`49a..49e`,`4a4`,`4f8..6b3`,`71e..766`.
Sorted authenticated bindings preserve previous-family or unmigrated bytes on
crossing reads. Eight separate source-kind counters cover pointer tables,
strings, relative indices, program headers, encoded controls, decoded program
bytes, glyphs and metrics. Under e12the earlier ninth-file subset is still
authenticated, but no second owner is created for its overlapping ranges:
it reports`verified-only`, while`text.catalogs`reports`decoded`.

Three reduced Wine checks pass after a warning-free Windows release test build:

- Synthetic source widths/mirrors/edges/prior-byte merging, zero operands,
  extended strings, glyph486/last font page, BIOS bus mode, SWI CpuSet, DMA and
  Load. Raw migrated ROM spans and duplicate retained encoded literal bytes
  are both altered; source reads still reproduce the exact original bytes.
- Fifteen original/interpreter/native chains cover five language IDs and raw
  extended/raw compact/custom-LZ forms, pointer strings, archive roots/copies,
  glyph486and variable/fixed metrics. All33581raw source ranges are altered
  after activation;90calls match complete serialized states and clocks after
  every call. Compiled read APIs consume60pointer-table,81string,30index,
 30header,396encoded-control,434decoded-program,1892glyph and18metric bytes.
  Original30840instructions/3093native dispatches are coverage counts only.
  The explicit valid nullable-context index1fixture is retained; this is not
  ordinary gameplay scene coverage or a repeat of the300-case offline oracle.
- With only temporary copied files made unavailable, failed activation preserves
  state/owner, and Load/Undo/backup/restart retain the same full model/counters.
  Subsequent typed pointer/string/index/header/program/font reads still work
  with migrated ROM source spans altered.

Evidence: `target/rom-data-text-catalog-consumption/windows-results.json`,
`windows-europe/result.json`, synthetic/native/lifecycle logs and source/protected
hashes. All jobs are terminal/pass; original ROM/slot/e12dataset/cache/loader/
canonical files stay intact. e12-d11, snapshots and native ABI/cache identity
are unchanged. The declared ordinary scene/CLI below now passes; broader scenes,
actual BIOS execution, complete reader/font/Unicode coverage and new Linux/USA/native
Windows checks remain open. No new recording, benchmark/profile/startup/RSS/
space measurement or old archive/scene oracle repetition.

## Ordinary transition and production CLI

One EUR/Wine legacy/decoded pair reuses the retained slot5LCD B, `pub.input`
397-frame transition and old native cache: entering the Cyril pub, BG62.
Every complete serialized state/clock, central/wide image, OAM/dispatch and
mixed/PSG/FIFO/HLE audio stream matches. The new family records2328reads and
8427source bytes:52pointer-table,185string,2relative-index,4header,27decoded
program,8064glyph and93metric bytes. Encoded-control use is zero: this scene
uses a raw archive program. The separate native04cases prove compressed use;
no compressed ordinary-scene claim is inferred.

A warning-free Windows production candidate is copied aside and the canonical
executable restored. One new decoded CLI command from an unrelated cwd reuses
e12without writes and the old native cache, reporting the same positive text/
font use and the retained legacy final image/dispatch row. The legacy command
is not repeated: protected ROM/slot/input/cache/loader and legacy behavior/ABI
remain identical; the new pair independently covers every state/audio sample.

Evidence: `target/rom-data-text-catalog-consumption/windows-scene-results.json`,
`windows-lcd-b-text-catalog-full-state-audio.log`, `windows-cli-summary.json`,
production build/candidate and source/protected hashes. All jobs terminal/pass;
original ROM/e12/slot/input/cache/loader/canonicals are unchanged. This closes
the declared reduced scene/CLI scope, not all gameplay or languages. Broader
scenes/GUI/archives, actual BIOS/complete font/readers/Unicode and new Linux/USA/
native Windows remain open. No new recording, performance/startup/RSS/space
measurement, old routine/catalog acceptance or extra scene campaign.

## Declared short-string catalog

`gba_core::rom_data::text_short::Catalog::from_rom` extracts the bounded EUR
catalog selected by original Thumb routine `08016648`. It authenticates the
complete EUR ROM before reading its five language tables. Managed preparation
now publishes it as a ninth fixed file under schema1/extractor11/decoder10;
startup authenticates it, and original CPU/DMA/native readers now consume it.

Each selector ID0..4has107entries. The independently read original literals
select pointer-table offsets `725c40`, `734598`, `741b9c`, `750230`, `75f318`.
Their string payloads begin at `7257f8`, `734134`, `7417a8`, `74fdec`, `75ee94`.
The next catalog's first string begins exactly428bytes after each table root,
which independently bounds the10732-bit little-endian cartridge pointers.
All535pointers and exact source extents are retained; padding before tables is
not claimed as text. Resource IDs follow first pointer occurrence, preserving
pointer identity rather than deduplicating equal contents.

Original `08019d98` defines two source encodings:

- Leading `01` selects compact strings. Each following nonzero byte expands to
  the big-endian glyph code `8000 | (byte - 1)`; zero terminates the source.
- Other strings use extended codes. A byte with bit7set consumes a second byte
  as one big-endian glyph code, including a possible zero low byte. Lower bytes
  remain exact control/operand bytes; a zero at that boundary terminates.

The typed model retains game glyph IDs and individual control bytes, plus the
source encoding. It can reconstruct both exact source bytes and the original
copy routine's expanded output. The declared535strings comprise478compact and
57extended sources, totaling5468bytes. They include `40 73` spacing and one
`40 3e 03` sequence; retaining these bytes does not claim a complete command
interpreter or Unicode mapping. Human language labels are not inferred from
selector IDs.

## Referenced font slots and metrics

Original `080135a4` computes `4954dc + glyph_id * 128` and copies the complete
128-byte packed4bpp slot through the initialized IWRAM callback at `03005ee9`
(ROM callback pointer `370eb4`). It then reads the variable width at
`4f885c + glyph_id`, or returns16in fixed-width mode. Original `08013bb4`
independently reads the same metric without copying. The mode is mutable RAM
at `020020e3`, not extracted data; rendering colors/palette remain dynamic.

The short catalog references65unique glyph IDs7..254. Only those65slots
(8320bytes) and65width bytes are extracted, in sorted glyph-ID order, as full
little-endian words and metrics. No pixel rearrangement, palette flattening,
new character interpretation or full-font extent is inferred. The bank limit
before the width root is an input guard, not proof that every possible slot is
used. Five index tables,535strings,65slots and65metrics produce670provenance
ranges with exact source hashes.

## Verification and remaining work

Two focused Windows/Wine checks pass after a warning-free release test build.
The synthetic codec check covers compact/extended paths, a zero low glyph byte,
control operands, truncation/length/encoding guards and pointer overflow.
The optional exact-EUR check source-authenticates a deterministic strict JSON
roundtrip, rejects unknown fields and coherent metric edits, and compares with
an independently retained static inventory.

It invokes the original selector for all535entries and the original copy
routine for every string, comparing all output bytes and destination guards.
All65font slots and both original width readers match in variable/fixed modes;
the original unknown-language fallback also matches. Total:1331calls and
89446guest instructions. These are coverage counts, not performance results.
The retained slot5only initializes the original IWRAM memcpy helper; no new
recording, boot, scene or full-state gameplay acceptance is claimed.

Local evidence: `target/rom-data-text-fonts/static-short-catalog.json`, bounded
disassemblies, `windows-results.json`, `windows-europe/result.json` and the
strict typed `windows-europe/short-texts.json`. Original ROM/slot/native cache/
loader/canonical executable hashes remain unchanged. The initial pre-guard
outputs are retained separately; final outputs include the pointer guard.

Ordinary gameplay scene proof remains open. Other short-text categories, relative-offset dialogue
archives, name keyboard producers, complete font coverage/control semantics,
Unicode and other languages/catalogs require separate contracts. New Linux,
USA and native Windows verification remain pending. No benchmark, startup/RSS
measurement, space optimization or AndroidPort change was performed.

## Managed publication and activation

EUR `v1/e11-d10` adds resource `text.short-strings`, fixed file `short-texts.json`,
format `short-texts-v1-json`, with670source ranges and no dependency on BG
descriptors. Complete-set verification compares the typed strings, indices,
glyph words and widths against extraction from the exact source ROM, refusing
coherent edits even after manifest/file hashes are updated. Dataset identity
includes the new file. The source ROM, guest snapshot and native-cache identity
remain unchanged. USA keeps its existing six-file `v1/e8-d7` contract.

Older e10generations and all seven prior decoded resource files remain intact.
The manifest input guard rises to1MiB to accommodate the new provenance, without
packing or space tuning. Invalid/missing/partial files are repaired by publishing
a fresh complete generation atomically; failed writes preserve the current
pointer and prior generation. Fixed relative names remain mandatory, and a
folder move reuses the generation without persistent absolute paths.

Three reduced Wine checks pass: deterministic nine-file publication/reuse,
partial-write failure/missing repair, and exact EUR source/whole-set identity,
coherent-edit/metadata/coverage/dependency/path/file-size guards, corruption
repair, move and transactional failed activation. The managed JSON equals the
closed offline02output; the old original string/font oracle was not repeated.
Actual Windows CLI prepare, no-write reuse and one-frame old-native-cache
activation from an unrelated working directory also pass. All protected
originals and retained e10files remain unchanged, and the canonical executable
is restored after the warning-free production build.

Evidence: `target/rom-data-text-short-managed/windows-results.json`,
`windows-europe/result.json`, `windows-cli-summary.json` and build/source/hash
manifests. Activation logs `runtime=verified-only`: source authentication and
file publication alone did not prove text/font consumption. The direct-consumer
unit below now closes that boundary. New Linux/USA/native Windows, ordinary
gameplay scenes and complete text/font coverage remain pending. No performance
measurements or old scene/oracle rerun.

## Direct consumption

The immutable runtime owner now retains the typed short-text/font model from
the authenticated ninth file. Exact table bytes come from stable string IDs
and source offsets; compact/extended string bytes come from the typed glyph
IDs and control bytes; packed glyph words and metric bytes come from their
decoded records. Small unit-position maps contain offsets, not duplicated raw
text. Original terminators, markers and big-endian codes remain exact.

Only fixed declared EUR source pages enter this projection: `495..49d`, `4f8`,
`725`, `734`, `741`, `74f..750`, `75e..75f`. Sorted authenticated bindings handle
mapped bytes; reads crossing a boundary preserve previous-family projections
or unmigrated adjacent ROM bytes. CPU, DMA, original BIOS bus mode and native
APIs share this source view. Separate table/string/glyph/width byte counters
identify use; no instruction, guest copy, render work or cycle is skipped.
e11-d10file formats, guest snapshots and native ABI/cache identity stay unchanged.

Three reduced Wine checks pass after warning-free Windows release test builds:

- A synthetic check covers compact/extended codes, zero low glyph bytes, all
  byte/halfword/word widths, ROM mirrors, boundary/prior-byte merging, last
  font/source pages, BIOS bus mode, SWI CpuSet, DMA and load-owner retention.
  Migrated raw ROM ranges are altered after attachment. BIOS bus mode is not
  a claim of actual BIOS-code execution.
- Five original/native reader chains, one per selector, alternate compact/
  extended sources and variable/fixed widths. With all670raw source spans
  altered after activation,20calls match complete serialized states and clocks.
  TinyCC covers original ROM and initialized IWRAM instructions; wrapped native
  read APIs observe20table,124string,636glyph and6metric bytes from typed data.
  Fixed-width cases correctly read no metric bytes. Total1984original traced
  instructions/920native dispatches are coverage counts, not performance data.
- Load/Undo, backup recovery and Restart retain the same owner and counters
  after only temporary copied dataset files become unavailable. Failed
  attachment preserves machine state/owner; subsequent reads of all four kinds
  still return typed values with raw ROM spans altered.

The first native probe had an overstrict test expectation requiring metric
reads even in fixed-width mode. Its states already matched; correcting that
expectation required no production-reader change. The failed log/object and
initial successful synthetic check remain retained; only the affected native
test was repeated, followed by the previously unrun lifecycle check. The
closed535-string offline and managed publication oracles were not repeated.

Evidence: `target/rom-data-text-short-consumption/windows-core-text.log`,
`windows-results.json`, `windows-europe/result.json`, native/lifecycle logs and
source/protected hashes. All jobs are terminal. Original ROM/slot/e11files/
cache/loader/canonical executable remain intact. Ordinary gameplay text scenes,
actual BIOS execution, broader catalogs/font/control/Unicode and new Linux/USA/
native Windows checks remain open. No new recording, production CLI/scene rerun,
performance measurement, packing or AndroidPort change.

## Complete European font and name keyboard

The auxiliary caller inventory identifies16direct callers of the original
archive reader and a separate EUR name-entry archive at `4f95c4`. Original
`0812e900` constructs three120-byte pages of big-endian selectable codes;
`0812f15c` selects by page, row and column. Each page has six rows and ten storage
columns; the18navigation row widths are at `3ad2fe`. The typed keyboard retains
relative indices, three raw extended programs, exact commands, navigation
widths and derived cells, including the space command and unused zero cells.
Runtime pointers in generic/dispatch callers remain outside this static catalog.

The keyboard references glyph3174, whose128-byte slot ends exactly at the width
root `4f885c`. The complete packed font bank therefore contains3175slots from
`4954dc`, retaining all32little-endian words and the width byte per slot. No
Unicode mapping or classification of bytes after those3175metrics is inferred.
Two reduced Wine checks now pass using the retained Windows test build: strict
source/JSON/shape guards and the3047new original glyph copies, all three page
outputs and180selectors. The original3230calls/628218instructions are coverage
counts, not performance measurements. Earlier128glyph oracles were not repeated.

EUR `v1/e13-d12` adds fixed `font-bank.json` (`font.complete-bank`) and
`name-keyboard.json` (`text.name-keyboard`), bringing the dataset to12files/
11resources. The manifest authenticates two font and five keyboard provenance
ranges, resource dependencies and all file hashes. Source-equal edited base
semantics remain required even when a file hash is recomputed. Font JSON has
a16MiB input guard and keyboard JSON a1MiB guard; these reject malformed inputs
and are not space optimization targets. Previous e12generations and all nine
decoded files remain unchanged. USA keeps its six-file e8-d7contract.

Both files are retained as immutable typed values for CPU/DMA/native source
reads. Font bytes come from packed words and metrics; keyboard bytes come from
indices, headers and typed program units. The original guest still builds its
cell pages, copies/renders glyphs and advances every instruction and cycle.
The complete bank takes over earlier overlapping font reads; the earlier text
files remain authenticated. Independent counters report glyph, metric, index,
program and navigation-width use, with separate host row-width reads. Host mouse
navigation uses the same immutable row widths while selection and cell codes
remain mutable guest RAM. Save-state/native-cache ABI and source identity stay
unchanged; no file I/O occurs during these reads.

Three reduced Wine checks pass for managed publication and runtime integration:
complete publication/reuse/repair/move/e12retention, interrupted writes, source/
metadata/path/coherent-edit guards, full-set identity and transactional failed
activation; five original/interpreter/native calls with all seven old raw
ranges altered and complete states/clocks equal, plus source widths/edges/
mirrors/native API; Load/Undo/backup/restart with temporary copied files made
unavailable. Compiled reads consume124glyph/2metric/6index/387program bytes;
row widths have separate bus coverage. The guest is never skipped or replaced.

One ordinary397-frame EUR/Wine Cyril-pub transition matches every full state,
clock, central/wide image, OAM, dispatch and audio stream with8064glyph/93metric
bytes from the new complete bank. It consumes no keyboard program; original
keyboard producer coverage above is separate. Three actual Windows CLI runs
prepare/reuse/replay from unrelated cwd, reuse e13without writes and the old
native cache, and match the retained legacy final row. No new recording or old
catalog/scene campaign was repeated.

Evidence: `target/rom-data-font-keyboard/{windows-results.json,windows-europe/result.json}`
and `target/rom-data-font-keyboard-managed/{windows-results.json,windows-europe/managed-result.json,windows-europe/native/result.json,windows-europe/lifecycle-result.json,windows-scene-results.json,windows-cli-summary.json}`.
Ordinary name-entry scene/GUI/archive coverage, other text roots/control semantics/
Unicode and new Linux/USA/native Windows checks remain pending.

Two additional focused Wine mouse checks pass: all three pages/180cells and
target bounds match with the old row-width source zeroed; removing the owner
makes that negative fixture fail as expected. The existing no-dataset mouse
check also passes. This is a controlled scene fixture, not ordinary name entry.
No performance, startup/RSS or space measurements were taken; AndroidPort is
unchanged.


## Script-selected localized text source contract

The five further EUR archives at `BB064C`, `C076F0`, `C5FBF0`, `CB7C48` and
`D10644` contain script-selected dialogue. Original `08009ab0` selects them by
live language IDs0..4, with other language values falling back to0. Each ends
at the next independently selected language root; the last ends at `D68C2C`,
the independently rooted script-key resource. The five ranges total1803744bytes
and are disjoint from all45published e51resource owners. This closes their
static inventory, rather than their model or runtime migration.

`08009b08` obtains a four-byte script key through `08009aa0` and uses its text-
bank byte to read a u32offset relative to the selected language root. The234
source key records reference bank IDs up to62; the minimum top-level offset is
`FC`, agreeing with63four-byte entries per language. The first word points to
a late shared bank0; it does not describe the top-level index length. Eleven
other normal bank entries share that bank per language. These source domains
do not introduce guest key, bank or message bounds.

Bank61has a second index of26u32offsets, also relative to the language root.
For the normal message path, the original reads the live byte at`02003C2A`,
computes `n = byte == 0 ? 0 : byte - 1`, and selects variant`n / 10` through
unsigned helper`08146474`. The separate message remapper`08009b7c` uses remainder
helper`081464ec` and adds `(n % 10) * 24` with original u16truncation. Division
and remainder have different roles; neither becomes a fixed text choice.
The root selector tests the full u16message against239, while the remapper
checks its low byte for special IDs. Preserve those distinct branches and the
original fallback key`AA`; do not replace them with new caps or a common guard.

After resolving aliases and variants there are77physical u16-indexed banks per
language,385in total. Their41395index entries include385`FFFF`sentinels and name
41010physical programs,8202per language. Original producer`0801916c` selects
these roots for text kinds3/5 through`08009b08`/`08009b7c`. Caller`08011906`
passes the returned root and remapped message through`08011920`→`08011e28`;
`08011e80` invokes the existing `08013f8c` index/header/expansion reader. The new
source uses the existing `text_catalogs::Program` and custom-LZ grammar.

Programs retain all16header bits, glyph IDs, commands and operands. There are
9813custom-LZ extended programs,5709raw extended programs and25488raw compact
programs. Their212011command units use11existing forms. Compressed data has
8784one-byte decoded tails and1029streams ending exactly at the terminator.
The147referenced glyph IDs1..3112 already belong to the complete font resource;
no new font payload is needed. Actual asset dependencies are `engine.script-archive`
for the key selectors and `font.complete-bank` for the glyph references.

The physical partition contains1260top-index bytes,520variant-index bytes,
82790bank-index bytes,1684407program bytes and34767alignment bytes. All34314gaps
are one–three bytes, with exact contents retained without assigning text
semantics. Four three-byte gaps contain mixed values. An initial private
uniform-fill assertion was rejected before writing a plan or producing any
model/runtime/test; only that assumption was corrected.

Independent static verification reconstructs and hashes all1803744bytes exactly
once, validates15195key/variant target formulas,349instruction halfwords and13
literal words, actual dependency hashes, all glyph references and disjointness
from every earlier owner. No guest execution, build or Wine test is part of
this inventory; existing text/codec/native/lifecycle/CLI/ordinary proofs were
not repeated. Evidence is under `target/rom-data-script-text/`:
`source-contract.json`, `static-source-verification.json`, bounded original
reader disassemblies and `rejected-uniform-padding-assumption.json`.

The following model and new Wine source oracle close the next criterion.
Publication, direct runtime consumption, production CLI and positive ordinary
acceptance remain pending. EUR stays e51-d50 and USA e8-d7. New Linux/USA/native Windows
checks remain pending; no performance or space campaign, fixtures, navigation,
save/RAM/payload edits or Android changes.


## Typed script-selected text model and reduced Wine source oracle

`rom_data::script_text` models all five ranges with typed top targets and variant
references. Each physical bank owns its `text_catalogs::Program` records once;
shared top entries refer to those banks. Relative top/variant words and u16index
words reconstruct from typed addresses, with explicit `FFFF`sentinels. Exact
one–three-byte alignment records retain arbitrary slack, including mixed values.
The model preserves every header bit, command, operand and encoded/decoded tail.

Validation requires canonical IDs and ordering, valid bank references, complete
nonoverlapping physical coverage, and actual script-key/font dependencies. The
147glyph references already belong to the font asset. Source construction
verifies the exact EUR image,349instruction halfwords grouped into digest
windows and13literal words. Bounded source lookup helpers retain language
fallback and live quotient selection without replacing original guest raw-ID,
message-remapping, text-expansion or callback behavior. EUR remains e51-d50;
no runtime owner or save/native ABI changes are part of this source-only unit.

The first warning-free Windows build and new Wine filter found a strict JSON
defect: the internally tagged unit variant accepted an unexpected field despite
`deny_unknown_fields`. The corrected empty struct variant rejects it, retaining
the assertion. Original guest cases had not started in that failed attempt;
its executable, build/output/log/result remain under `before-strict-variant-json`.
No earlier text, codec, native, lifecycle, CLI or ordinary proof was repeated.

After the correction, a warning-free Windows build and one combined Wine filter
pass all independent fields, hashes and1,803,744source bytes,385banks/41,010programs,
147glyphs/11command forms. Twenty-two shape, six coherent source-edit, three
dependency,12JSON/shape and two identity guards pass. New original-reader cases
cover seven language inputs,1170key/bank selections,1280live quotient choices,
256live remainder choices,90raw/special-message combinations and1170u16index
prefixes including sentinels and raw wrap. The837header cases preserve both
flags and metadata;837original-source versus typed-rebuilt program outputs and
guards agree, including249compressed bodies and decoded tails.

The reader cases reuse the retained initialized EUR checkpoint without changing
it. Controlled scratch arguments use the existing nullable-context fallback;
they are separate from ordinary gameplay acceptance. The2,116,900instructions/
834PCs are coverage counts. These results do not establish native full-state/
clock parity, a complete text/callback ABI, all argument semantics or ordinary
reachability. All earlier4060artifact hashes, failed-attempt files and368hash-
mtime guards remain protected. Evidence under `target/rom-data-script-text/` is
`windows-model-{tests,result}.json`, `windows-model-europe/{script-text,result}.json`
and `windows-model.log`.

The source model and reduced Wine source oracle are closed. The following
sections cover managed publication, direct runtime and production CLI; ordinary
acceptance is recorded separately. New Linux/USA/native Windows verification
remains pending; USA stays e8-d7. No performance or space measurements, new gameplay fixtures,
navigation, edits to retained checkpoint/save/input files or Android changes.


## Managed script text and direct typed reads

EUR `v1/e52-d51` adds `script-text.json` / `text.script-archives`, file47 /
resource46. All45earlier decoded files and the e51generation remain unchanged;
USA stays e8-d7. The manifest declares actual `engine.script-archive` and
`font.complete-bank` dependencies. Activation checks their loaded key/glyph
links, source-set coherence and authenticated base contents before publication.

The owner covers all five ranges/1,803,744bytes through typed top, variant and
bank indices, program headers, encoded controls, decoded glyph/command/operand/
terminator fields and exact alignment bytes. It shares `TextProgramMap` with
the earlier catalogs: encoded literal positions read typed decoded units, while
size/token/reference bytes retain the original codec metadata. Original guest
selection, decompression, text expansion, live RAM and clocks remain unchanged.
Mixed reads merge prior script-archive owners at both shared BB0/D68pages and
raw data outside registered ownership. Save/native ABIs are unchanged; usage
counters and coverage masks stay outside serialized state.

The full source scan reads1260top,520variant,82790bank-index,82020header,
617389encoded-control,733676glyph,216619command,2477operand,32226terminator and
34767alignment bytes. Direct decoded-tail source literals are0: preserved tails
are generated by LZ controls/backreferences. The counter remains explicit;
there is no claim that the original decoder does not produce or read tails.
An initial private scanner's all-classes-positive assumption was rejected.

Affected Windows core tests compile warning-free. The first frontend compile
finds only a new test counter shape mismatch and an unused import; those are
corrected, failed logs retained, and only the frontend build repeated. The
corrected build is warning-free. Three sequential Wine filters pass on their
first execution: one existing core literal-poison test for the changed shared
mapper, then new managed and native checks. This affected-code check does not
reopen earlier text source/native/lifecycle/CLI/scene acceptance for context.

The managed check passes13guards, atomic create/reuse/repair/move, all earlier
file/e51retention, failed-activation isolation, unavailable-file reads and
Load/Undo/backups/Restart. With raw source bytes altered, every source byte
agrees and all5languages/385banks/41010programs/130variant records/34314gaps are
covered. Another163source bytes from earlier formats0/1/2verify the shared map
on real old programs. The native check passes35original/interpreter/compiled
complete-state/clock prefixes; compiled source categories are
[21,17,17,13,6,4,5,1,3,0,3]aggregate bytes. Raw top255/variant255, u16sentinel/
65535/high-bit truncation, previous2+new2/new2+previous2 boundaries, raw fallback
after the adjacent key owner, widths/mirrors, real BIOS, CpuSet/FastSet and DMA
agree. The379instructions/65PCs/71native dispatches count coverage only; these
are controlled field/selector prefixes, rather than a whole text/callback ABI.

Evidence under`target/rom-data-script-text/` includes `windows-managed-results.json`,
`managed-run-protected.json`, `windows-managed-europe/{managed-result,native-result}.json`,
`windows-{shared-text-map,managed,native}.log` and the retained compiler diagnostics.
The shared-mapper core filter is closed after its new-code pass; source-model,
inventory, e51/e50and old family proofs remain closed.

## Script text production CLI and ordinary school dialog

The warning-free Windows production CLI creates and reuses49managed files from
an unrelated working directory without rewriting them, loads the old native
cache, and matches all47generation files to the accepted e52dataset. The canonical
executable is restored exactly. Candidate SHA-256 is
`6126ce932fda4847579ad8dfb2f28d1a7f4a7a4c640e0fba06402d45440d137e`;
dataset SHA-256 is
`2379f2425ddfe664425502b7f4dcc0f4d0b11c0ebdec617ba5a94c1b2c73fa88`.
All468hash-mtime guards, including retained inputs and the accepted dataset, pass.

The retained8476-frame suspend-boot-to-battle locator reads0new text bytes and
is closed negative; no fullpair runs for it. A read-only audit of74unique EUR
checkpoints supplies no active text-manager cue and does not prove unreachability.
Four USA school/name headers are excluded before payload/replay. This is separate
from the previously closed engine-queue audit.

The genuine EUR courtyard dialog and retained `intro.input` select a distinct
cold-boot school3000/classic240x160 locator on the same CLI/dataset, with no save
or state override. It reads289aggregate source bytes:8top+4bank-index+8header+
93encoded-control+136glyph+39command+1terminator. Decoded program bytes total176;
one language, one bank and two programs are used. Variant/operand/tail/slack reads
are0. This is actual new-text source use, rather than inferred archive usage.
One Wine fullpair from the actual CLI-published e52generation passes every
one of3000full states/clocks, central/wide images, OAM/dispatch and mixed/PSG/FIFO/
bitwise-HLE samples. Observed sample counts are
[6582846,6582846,791212,791212,6561508]. New-text usage matches the locator and
all468hash-mtime guards pass. The already-built test executable, native cache
and retained input are reused, with no save/state override or earlier proof
repeat. Terminal evidence is `windows-scene-result.json` and
`windows-school-text-full-state-audio.log`.

Other4languages/384banks/41008programs,130variant records and34314gaps retain
targeted coverage only; no whole ABI/new32:9claim. The separate
[e49ordinary selector criterion](rom_data_events.md#positive-ordinary-book-transition)
is subsequently closed by a new EUR book scene; three old negative contexts
are not reopened or recounted. New Linux/USA/native Windows checks remain
pending. No performance/space measurement, new gameplay fixture/navigation,
retained checkpoint/save/input edits, Android changes, agent commit or push.
Evidence: `windows-build.json`, `windows-cli-{processes,no-write,summary}.json`,
`ordinary-text-context-audit.json` and `windows-cli-text-school3000.json` with
corresponding logs under `target/rom-data-script-text/`.
