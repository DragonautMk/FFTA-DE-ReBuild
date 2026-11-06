# Local cheat reference catalog

The root `cheats/` directory is a private reference collection for future work.
Open `cheats/README.md` locally to browse the Markdown index. The directory and
all derived Markdown/JSON are excluded from Git: the supplied third-party
compilations include an explicit copyright notice and their redistribution
permissions have not been established. This documentation contains no cheat
payloads and does not assign a license to the source material.
The [distribution provenance review](development/CHEAT_DISTRIBUTION_PROVENANCE.md)
distinguishes the EUR entries actually used, source-informed validation and
the outstanding routes to a distributable replacement.

The [integrated-debug follow-up checklist](development/DEBUG_TOOLS_V2_TODO.md)
tracks a local cheat menu using these stable CH identifiers, with EUR candidates,
format validation, isolated application and replay provenance. Read-only
browsing, a limited manual one-shot and bounded maintained interpreter plans
are implemented as described below; device equivalence and replay provenance
remain open.

## Organization

The initial collection contains 341 distinct code blocks from five supplied
documents, after merging 228 duplicate occurrences. Browsable pages separate
European Codebreaker/GameShark SP/Xploder codes, USA and Japanese GameShark
sections, and Codebreaker/Xploder and Action Replay lists with unconfirmed
regions. Separate pages retain the prose guide, modifier lookup lists, source
attributions and quality findings. The original language and values are kept.

Each block has a stable `CH-` identifier, ordered code lines, source references,
aliases and links to the master code supplied by each source. Deduplication uses
the entire ordered block, declared device format and region together. The
unconfirmed-region group records identical text without asserting that its
sources target the same ROM. Repeated instructions inside a block are retained.

`cheats/catalogo.json` is a local schema-1 export for readonly indexing. It contains
source hashes, `entries`, format/region, canonical code lines, aliases, notes,
status and provenance. Each provenance item retains the source title/category,
original line numbers and raw code lines (or the site's code ID), plus its
`master_id`. Existing IDs should remain stable when extending the catalog.

## Readonly index and CLI — DBG2-17 in progress

`recomp debug-cheats list` reads `cheats/catalogo.json` beside the executable;
if it is absent, the result is an explicit empty state. Select another local
file for one invocation with `--catalog FILE`. Listing is paginated with
`--offset N --limit 1..64`; `show CH-0001 --catalog FILE` returns the complete
original entry, including its ordered lines, aliases, notes, provenance and
source-specific master links. `info` returns the original catalog metadata.
The JSON response includes the SHA-256 of the source and the total count.
Import is readonly and bounded to 4 MiB and 2,048 entries; malformed entries,
duplicate IDs and unsupported schemas are rejected before an index is returned.

`show` also reports each original provenance record's master reference, in
source order. It distinguishes a same-source match, source mismatch, self
reference, unknown identifier, missing or null reference, invalid identifier,
and invalid source. It counts matching provenance records in the linked entry
and reports format/region metadata equality where a target exists. These are
metadata comparisons only: they do not establish which master should be used,
authenticate encryption, or activate a dependency. `inspect` returns the same
report next to its explicitly assumed plaintext plan. The original entry is
preserved unchanged. A focused Windows GNU/Wine test and a synthetic CLI
check exercised these states, the agreement between `show` and `inspect`, and
source preservation; no real catalog or guest execution was needed.

`--rom-sha256 SHA` records a supplied target identity in the response. For the
two exact ROM hashes already recognized by the project, the index compares the
recognized region with the region declared by the source and labels the result
`declared-match` or `declared-mismatch`; an unrecognized hash or source region
remains `unknown`. The CLI does not authenticate the caller's ROM bytes, and a
regional declaration never verifies a code. Every entry remains disabled and
unverified. The index assesses source-linked CodeBreaker plans separately from
the original entry: `decoder.state` describes the bounded source assessment,
and `manual_once.state` describes whether a matching ROM context and runtime
preflight are still required. These states are not device or gameplay proof.
An unresolved source is `not-established`; exhausted assessment limits are
`not-evaluated`. A supported plan with a recognized matching region still
reports `requires-runtime-preflight`, not that it is executable.
`show` includes per-source assessment and, when decoded, its typed plan and
potential footprint. Inspection itself does not apply code; an explicit
guarded one-shot is a separate action described below.
Catalog contents are not included in source or
public packages; CLI output itself may contain private third-party text.
For bounded host work the index assesses at most eight sources per entry, at
most 64 lines per source and 16,384 lines per catalog. Sources beyond a limit
are reported as not evaluated, never silently promoted to unsupported or
verified. The source-bound interpretation assumes the linked master is the
correct seed; ambiguous, missing or undecodable sources remain unresolved.
A listed one-shot candidate still requires explicit source/master selection,
the actual loaded ROM checksum, a paused isolated interpreter and a fresh
preview. A caller-supplied ROM hash alone does not verify the ROM bytes.
For the source-capability index, a frozen Windows GNU build and 12 focused Wine
tests passed. Seven local EUR source plans matched their explicit inspections
without changing the catalog; a synthetic Wine/Xvfb view displayed the ROM
context requirement while the isolated guest remained paused and unchanged.
The first GUI setup attempt failed on its test-helper path; a corrected attempt
passed. These checks do not verify the device or gameplay effects.
Focused Wine tests and a synthetic CLI check cover recognized EUR and USA hashes,
a declared mismatch and an unknown hash without loading ROM bytes. A separate
synthetic Wine/Xvfb view confirms that the loaded ROM hash and the unknown state
are shown without advancing the guest.
The isolated Windows GNU build and focused Wine tests passed. A readonly Wine
check against the local catalog compared all 341 entries and the catalog
metadata with the source, checked paginated order and disabled compatibility
states, and confirmed that the source hash was unchanged. It also checked an
absent catalog and an unknown ID. This does not validate cheat semantics.

The Debug / Cheat page computes the exact hash from the currently loaded ROM
and shows it alongside the catalog hash and the same declared-region comparison.
It opens a catalog view with search, filters and scrollable original details.
The type filter combines the source format and declared region. State filters
can show armed entries, entries not armed, matching declared regions, source
plan candidates or entries needing attention. A plan candidate still requires
runtime preflight and does not establish device or gameplay compatibility.
In a synthetic Windows GNU/Wine GUI test, the armed filter changed from zero
to one entry and back after disarm; candidate, declared-region, attention and
not-armed filters were also checked without advancing the guest.
Highlight an entry and choose **Activate...** (or confirm the list entry) to
open its management screen. It shows the cheat name and guides you through
**Select source**, **Prepare preview** and **Arm** or **Apply once**. After
reviewing and closing the preview, the controller focus moves to **Arm**; the
user must still confirm the action. A currently armed entry is marked in the
list and its contextual button becomes **Disarm**, stopping future writes.
The management screen also offers **Disarm all** and presets.
The preview summarizes the operation count, maximum writes per application
and whether the plan contains conditions; the detailed operations remain
scrollable below. These source-plan conditions do not establish game behavior.
Choose a source and review the operations and potential read/write ranges before
explicitly applying or arming; the shared identity, state, checksum and overlap
guards still apply. Disarm stops future writes without restoring memory.
The catalog's **Disarm all** action is also available when the selected catalog
is empty or unavailable, so it can stop an already armed set.
A catalog next to the executable is loaded when present;
otherwise the page explains the empty state. **Import catalog...** opens the
internal content selector: refresh its declared roots, select `catalogo.json`,
return and choose **Reload**. External file/folder dialogs remain explicit
buttons in that selector. The selected file is checked against its pinned hash
before loading. Selecting a CH only opens its details; execution requires the
separate management view. Cheat action labels use English source keys and follow the selected menu
language; existing Italian keys remain available for compatibility. Details
occupy a separate scrollable panel with the compatibility reasons
before the original record. Long individual records are marked as abbreviated
in the view and remain available in full through `debug-cheats show`. Decoding
and controlled application remain partial, with the one-shot scope below.

To keep a cheat in the favorites list, highlight it and choose **Favorite**;
choose **Unfavorite** to remove it. **Show favorites** switches directly to
favorites, and **Show all** returns to the full collection. The collection
filter also remains available; search, type and state filters still apply.
Clear those filters if they hide a favorite.
Favorites are stored locally by catalog hash; a different or modified catalog
has a separate list. A menu-created interpreter copy receives a snapshot of
the current favorites when the selected catalog is the default catalog; later
changes in that isolated copy do not update the original app automatically.
An invalid or externally modified favorites file is preserved and reported.

For a local build or an extracted portable app, the user's existing catalog can
be kept at `cheats/catalogo.json` beside the application executable. The Debug
/ Cheat catalog page finds it automatically, and the file stays available on
later launches. The catalog is user-supplied local data; it is not embedded in
the executable or included in public packages. For another app folder, copy the
catalog into that folder's `cheats/` directory without replacing a different
existing catalog. New isolated debug sessions copy and pin this catalog in their
own folder; existing sessions are unchanged. In the menu, the persistent Debug page can be unlocked with
↑ ↑ ↓ ↓ ← ← → →. Select a catalog entry, choose **Activate...**, select its
source, prepare a preview and explicitly apply or arm a supported plan.

The EUR CH-0341 source uses conditional slides, which the bounded interpreter
plan now accepts. In isolated native Linux/X11 and Windows Wine sessions, the
catalog opened automatically after session creation; the shared control channel
prepared, armed and disarmed that entry, and its expected writes occurred over
one controlled frame. The separate frame-boundary research path also reached
the first battle's Mission Cleared screen. These results do not verify the
original device hook or other game situations. Unsupported formats and sources
remain identified as such in the catalog.

In isolated Wine/Xvfb tests, keyboard navigation selected a synthetic source,
reviewed its preview, armed and disarmed it, applied it once and disabled all.
Attempts to arm without a selected source or reviewed preview were refused.
A separate test exercised the management sequence with simulated XInput;
physical controllers and native Windows were not tested. Protected inputs and
session pins remained unchanged. This does not prove a real cheat's game effect.
Another isolated test changed the selected catalog after arming; reloading
reported it unavailable, and the catalog-level **Disarm all** still stopped the
armed set. The temporary catalog was restored and protected originals remained
unchanged.

### Assumed plaintext CodeBreaker inspection — DBG2-18 in progress

`recomp debug-cheats inspect CH-0001 --catalog FILE
--assume-plaintext-codebreaker` interprets a single entry declared CodeBreaker
as a **hypothetical plaintext** block. It never infers encryption from the
characters or selects a master. The returned
ordered typed plan includes widths, addresses and values; it is explicitly
non-executable and does not verify the source device, ROM compatibility or
runtime behavior. The original catalog is unchanged. Callers must not use the
plan as authorization to write guest memory.

The initial subset recognizes byte/halfword assignment (3/8), halfword OR,
AND and add (2/6/E), halfword equality/inequality conditions (7/A) guarding
one following operation, including a halfword slide (4) with its continuation.
Halfwords use little endian in the guest; slide count and increments are
reported explicitly. Addresses must be width-aligned; slide count is limited to
1..1024 and all its addresses must fit the encoded address range. An invalid
line, placeholder, unsupported opcode, encryption control, master, incomplete
continuation or nested condition rejects the entire block
without a partial plan. This intentionally does not cover every device or
CodeBreaker opcode; even a syntactically successful block may be encrypted.
GameShark/Action Replay remain unsupported. This assumed-plaintext inspection
never applies guest writes; the separate source-bound manual one-shot below
has its own interpreter-only preflight and limitations.

These interpretations are based on the public mGBA
[CodeBreaker opcode definitions](https://github.com/mgba-emu/mgba/blob/master/include/mgba/internal/gba/cheats.h)
and [decoder](https://github.com/mgba-emu/mgba/blob/master/src/gba/cheats/codebreaker.c),
cross-checked against the [VBA GX CodeBreaker type descriptions](https://github.com/dborth/vbagx/blob/master/source/vba/gba/Cheats.cpp).
They are independent descriptions of emulator behavior, not a device-version
or game-specific compatibility proof. Synthetic tests cover the accepted
subset and atomic rejection; the existing local catalog remains private.

### Explicit seeded CodeBreaker inspection — DBG2-18 in progress

For a block the caller has independently identified as encrypted CodeBreaker,
`recomp debug-cheats inspect CH-0001 --catalog FILE --codebreaker-seed
9XXXXXXX:XXXX` decrypts each entry line under that explicit seed before applying
the same bounded typed inspection, including continuation lines. The seed option
and plaintext assumption are mutually exclusive. Neither mode chooses a source
master or detects encryption; in particular, a ciphertext line can itself begin
with `9`. Unsupported decrypted opcodes, incomplete blocks and invalid seeds
fail the whole inspection. A wrong seed may still produce a plausible plan, so
the result remains non-executable and unverified even when decoding succeeds.
The response includes the caller-supplied seed for reproducibility.

The cipher follows the independently described [GBATEK algorithm](https://problemkaputt.de/gbatek.htm#gbacheatcodescodebreakerxploder),
cross-checked against [mGBA's implementation](https://github.com/mgba-emu/mgba/blob/master/src/gba/cheats/codebreaker.c)
and a [public converter's synthetic collision test](https://github.com/SkillerCMP/CMP-GBACheatConverter/blob/main/tests/test_codebreaker.cpp).
It does not establish the device version, required source master, exact ROM
compatibility or runtime effect, and does not implement automatic reseeding,
master execution or guest memory writes.

Validation compared an independently expressed GBATEK formula with the mGBA
cipher on 257 synthetic cases. The Windows GNU/Wine CLI then produced the same
256 synthetic halfword-write operations across four supplied seeds, including
ciphertext beginning with `9`; tests also covered a decrypted slide and its
continuation, malformed/unsupported input and mutually exclusive modes. The
catalog was unchanged. These checks do not establish correctness for a real
catalog entry or any guest effect.

### Explicit source and master inspection — DBG2-17/18 in progress

`recomp debug-cheats inspect CH-0001 --catalog FILE --provenance-index 0
--master-id CH-0002` selects one original provenance record and the exact master
ID it names. The master must have exactly one provenance record from the same
source and matching declared format and region. Its three source lines must
decode as seed, game ID and hook; the entry is interpreted from that source's
original lines, rather than silently substituting a different source. The result
identifies the selected source and hashes its entry/master line arrays; it also
reports whether the entry's canonical lines match those source lines. Missing,
ambiguous, mismatched or unsupported records reject the complete request.

The typed master description reports its checksum and flags, ROM hook offset,
Thumb mode and handler slot/address. The existing CLI can optionally compute
the whole-file SHA-256 and the CodeBreaker checksum of the first 64 KiB from
`--rom-file FILE` (short input is padded with `0xff`). It reports a match or
mismatch when the master requests a checksum. `--rom-file` and caller-supplied
`--rom-sha256` are mutually exclusive. This reads the selected file only; it
does not install the hook. A checksum or declared region match is not a proof
that the device, source, native backend or cheat effect is compatible. Every
plan remains unverified and non-executable.

With the local reference catalog and the project's exact recognized EUR ROM,
Wine CLI inspection was compared with mGBA decryption for the seven priority
entries. CH-0321 and CH-0336 each produced a slide; CH-0323/0324/0325/0331
each produced one conditional operation; CH-0334 produced two halfword writes.
Each selected source master had a checksum matching those ROM bytes, and the
source lines matched the canonical entry. The full operations remain available
locally through the explicit CLI query; private source payload is not included
in this documentation or the recorded test result. ROM and catalog hashes were
unchanged. This is static decoding and checksum evidence, not proof of actual
game effects, correct hook integration, safe activation or reversibility.

### Manual one-shot in an isolated session — DBG2-19 in progress

After selecting and refreshing a catalog in an isolated session, use the command
console or `debug-command` with `cheats prepare CH-0001 0 CH-0002 GENERATION`.
The response contains `status.cheat_preview`: source/master identity, typed
operations, all potential read/write ranges, frame/clock, state hash and a
preview token. Review it, then explicitly issue `cheats apply-once CH-0001 0
CH-0002 GENERATION TOKEN`. The same typed actions are available as
`debug-control cheat-prepare` and `cheat-apply`. A successful one-shot consumes
the preview; retry uses the existing request ledger and does not duplicate it.

Preparation and application require an idle paused isolated copy without an
active recording or replay, an interpreter backend, the exact recognized ROM,
a matching declared region and a source master checksum matching the loaded ROM
bytes. The selected catalog is rechecked; application recomputes the plan and
requires the same source, generation and complete paused-state hash as the
preview. Preflight validates the entire block before any guest bus access, permits
at most 256 writes to aligned canonical EWRAM/IWRAM and allows condition reads
there or from KEYINPUT. Writes go through the guest bus with their proper width.
Other reads, I/O writes, mirrors, ROM writes, native code and oversized plans
are refused. Operations execute once at the paused frame/clock, in their source
order, without installing the original device hook or maintaining a toggle.
This is an explicit manual experiment, not verification of the code's intended
game effect; restoring a checkpoint remains a separate action. Broader memory
semantics and provenance-aware recording remain unfinished.
Snapshot slots can be saved after a one-shot. Such snapshots carry a bounded,
conservative record of observed cheat ancestry; see [save states](save_states.md).
Input recordings after cheat use remain refused because they do not yet retain
the provenance needed for replay.

For maintained use, prepare the same state-bound preview, then issue `cheats
arm CH-0001 0 CH-0002 GENERATION TOKEN`. Arm itself writes nothing. The frozen
plan runs through the guest bus before each interpreter frame, after the input
mask is latched, including controlled steps. `cheats disarm CH-0001` stops its
future applications; `cheats disarm all` clears the set. These operations are
also available as `debug-control cheat-arm` and `cheat-disarm`. Status reports
the armed identities and footprints; the read-only catalog displays an armed
indicator. A disarm does not undo earlier writes. Loading or restoring a state
clears the armed set. Saving while armed records that fact in the snapshot;
loading never rearms the set. Recording while armed or after cheat use,
instruction search and starting a separate replay while armed are refused.
Manual one-shot is refused until the set is disarmed.

At most eight source-bound plans and 256 potential writes per frame can be
armed. Read/write and write/write overlaps are refused before changing the set,
including conditional reads. Each plan retains the source and ROM identities
verified at arm time; it is an immutable snapshot, not a live catalog reload.
The same interpreter, ROM, checksum and canonical memory restrictions as the
manual preview apply. Timing is a host-defined frame boundary, not the original
device hook; device and gameplay behavior are unverified.

To save the currently armed set without applying it, issue `cheats preset-save
NAME`. Names contain 1–48 ASCII letters, digits, hyphens or underscores. The
new file is stored in `Data/config/debug-cheat-presets/` for a portable isolated
copy; an existing name is never overwritten, and the directory is capped at
32 files. The preset records the ordered members and exact ROM, catalog, source
and master hashes. It contains no game data.

After `cheats disarm all`, use `cheats preset-prepare NAME` and inspect every
member in `status.cheat_preview`; then use `cheats preset-arm NAME TOKEN` with
the returned token. The corresponding direct commands are `debug-control
cheat-preset-save`, `cheat-preset-prepare` and `cheat-preset-arm`. Preparation
and arming require the selected matching catalog, an idle paused interpreter
and an empty armed set. Arming repeats the full source, ROM, paused-state,
footprint and conflict checks and either arms the entire group or none. No
guest write occurs until the next interpreter frame. The **Manage cheat** page
also has **Presets**: edit the name and save the armed set, then explicitly
select a saved preset, prepare and review it, and arm it. Changing the selection
or switching between source and preset views invalidates the stored review
token. Parameter value and range selection remains open work.

Frozen Windows GNU tests and an isolated synthetic Wine/Xvfb session checked a
two-member save/preview/arm flow, duplicate name, changed preset, source lines,
ROM and selected catalog, and a conflicting group. The group was not partially
armed on refusal. The first catalog-change attempt modified a different copy;
a separate check targeted the selected source and confirmed refusal and
recovery. No guest frames advanced, and original files and session pins stayed
unchanged. A separate synthetic Wine/Xvfb session exercised the page with
keyboard and pointer: it saved, selected, reviewed, armed and disarmed one
preset without advancing the guest. The validation runner retained that
session's private test copies; a separate evidence receipt passed. Native Windows
and physical input were not tested.

Frozen Windows GNU builds and focused Wine tests passed. In a synthetic isolated
Wine/Xvfb interpreter, arm left the paused guest unchanged, an overlap was
refused, two independent plans applied on two controlled frames, and disarming
stopped later writes; the original inputs and session pins remained unchanged.
A separate paused synthetic view displayed the armed indicator. A user-initiated
ROM restart through the System menu cleared an armed synthetic set in a fresh
isolated copy. In another paused copy, loading a locally created state and
undoing that load each disarmed the set and restored the expected exact machine
state. Protected inputs and session pins remained unchanged. Native Windows,
physical controllers and gameplay effects remain unverified.

In an isolated EUR interpreter session under Wine/Xvfb, a synthetic block
performed three bus writes with the expected bytes while frame, clock and CPU
registers stayed unchanged during application. The proof rejected a wrong
token, changed source, changed selection with identical content and changed
paused state; retry did not repeat the writes. One controlled frame was stepped
to check stale-state refusal. Session pins and guarded original ROM, catalog
and save inputs remained unchanged. The session and display were closed.
This did not apply a real catalog entry or verify gameplay, native Windows,
Linux runtime, physical controllers or the save/recording guard interactively.

A separate readonly-source Wine/Xvfb experiment explicitly applied EUR
CH-0334 once in a fresh isolated interpreter copy. Both decoded halfword writes
changed the selected RAM values to the plan's values; frame, clock and CPU
registers stayed unchanged during the application. After one controlled frame,
closing and reopening from the same seed restored the original complete state
and the two original memory values. The source catalog, ROM, seed and guarded
original saves stayed unchanged. This establishes a bounded RAM effect, not a
visible gameplay effect, maintained activation, device-hook equivalence or a
reversible toggle. The retained result excludes the private code payload.

On a separate isolated EUR world-map seed, CH-0324 was armed for one
Select+Right frame and disarmed. After 90 neutral frames, the Item Shop greeting
and Buy/Sell/Leave menu appeared. A second copy with the same ROM, runtime and
exact starting state received the same input and frame count without the cheat
and remained on the world map. Loading the first copy's seed restored the exact
state; protected originals remained unchanged. This verifies that one bounded
interpreter scenario reaches the shop, not original device-hook timing or every
scene. In the same seed, CH-0334 left displayed clan funds at 7,749 after one
armed frame, 30 neutral frames and a city-menu redraw; the earlier RAM writes do
not establish a visible funds effect in this scenario.
In a distinct EUR shop purchase, two isolated copies started from the exact same
quantity-one Bronze Helm checkpoint and received the same 98 input/neutral
frames. Without a cheat, the 500-gil purchase reduced displayed funds from
7,749 to 7,249. With CH-0334 armed throughout the same purchase, the displayed
result was 99,999,499. The cheat was explicitly disarmed without changing the
paused state, and both copies restored their exact checkpoint. The source ROM,
catalog, checkpoint and guarded original saves remained unchanged. This verifies
the resource effect for this purchase and interpreter timing, not the original
device hook or every game scenario.
In a separate copy of that EUR seed, the Party screen showed Marsh at JP 4.
CH-0336 was armed for one frame and reported 24 writes, then disarmed; after a
bounded next/previous input sequence, Marsh still displayed JP 4. No intermediate
selection image was captured, so the redraw itself is unconfirmed. The original seed
was restored exactly. This is a negative observation for that unit and scene,
not a conclusion about other units, scenes or device behavior.

For the advanced level group, CH-0340 was armed for one frame in an isolated
EUR Party screen, then disarmed. After a confirmed selection of Montblanc and
return to Marsh, Marsh remained level 5 while displayed EXP changed from 78 to
99. The same-seed/input/frame control remained at EXP 78; both copies restored
their exact seeds. The plan recorded 24 writes. Montblanc displayed EXP 99 in
the enabled run and 92 in the control, but these secondary captures were at
different frames. No actual level-up, other unit's prior value or original device
hook was verified. The effect persisted through the observed selection change
after disarm; disarm itself did not restore memory.

In an isolated EUR Party scenario, CH-0320 was armed for one interpreter
frame and then disarmed. Marsh's Steal: Ability displayed 180 beforehand and
MASTER! after reopening the list and scrolling; the same-seed/input/frame
no-cheat control still displayed 180. The expanded enabled list and the control
had different selected rows, but the same ability was visible in both. The
observed change persisted after disarm; both exact seeds were restored and
protected originals remained unchanged. This checks one ability and scene, not
all abilities or characters, other side effects, native Windows or equivalence
to the original device hook.

In an isolated session, the local control channel and command console can
select a discovered cheat catalog by ID, refresh an immutable snapshot, list
bounded pages and retrieve complete entry or source-capability JSON in UTF-8
chunks with their respective hashes. The snapshot
reports the source hash and ROM identity checked at refresh; it does not claim
that the underlying file remains unchanged afterward. See
[debug control](debug_control.md) for syntax, cancellation and limits.

Wine/Xvfb validation exercised the empty state, explicit internal import,
search from the first through the last of 341 public synthetic entries,
scrollable details and simulated XInput navigation. A separate readonly test
compared the menu representation of all 341 local entries with their original
records without printing them. The guest state/frame/clock and catalog sources
remained unchanged. Native Windows, Linux and physical controllers have not
been checked.

## Review and limitations

The original catalog validation checked preservation and organization without
applying codes to a runtime, ROM or save. The later isolated CH-0334 one-shot
established only the bounded RAM effect documented above. USA/Japan labels
follow the supplied sources. English text alone does not establish a ROM region. Device versions,
encrypted-code semantics and compatibility with FFTARecomp remain unverified.

The local quality page flags two different Codebreaker master codes, apparently
swapped P3/P4 level labels, a possible Player 3 modifier typo and unresolved
placeholders. Values remain unchanged. The saved European web page contains
23 actual code blocks but 25 empty groups loaded on demand, and one missing
popup note; its collection is explicitly incomplete.

The static check verified all 796 source code-line occurrences, the 341 rendered
blocks and their source-specific master links, 1,544 local Markdown links, guide
and modifier-list preservation, and Git exclusions. It also verified all 42
original files against their SHA-256 archive manifest. The active directory now
contains ten Markdown files and one JSON catalog; 37 saved-browser support files
and the five superseded source documents were removed from that directory.

The reversible original archive, manifest, conversion/verification scripts and
cleanup receipt are retained locally under
`target/cheats-catalog-2026-10-28/`. They are ignored and unavailable in a fresh
clone. Missing local copies do not authorize downloading third-party collections
or recreating game fixtures automatically. The scripts describe this one-time
conversion and are not a product importer.

## Adding references

Record the source and attribution, ROM identity/region, exact device version,
complete ordered block and actual verification status. Keep master-code
relationships and regional variants explicit. Review suspected typos against
an independent source or an isolated validation before changing values.
Reassess redistribution permission before publishing any collection content;
the local ignore rule alone is not a license or a packaging exclusion.
