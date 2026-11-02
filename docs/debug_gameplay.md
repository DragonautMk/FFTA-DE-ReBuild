<a id="task-script-e-unità--dbg-17"></a>

# Tasks, scripts and units — DBG-17

In a paused [isolated copy](debug_sessions.md), **Debug > Start task / script
trace** arms the collector for one frame. Step through or replay the sequence,
pause again and choose **Save gameplay report**. Open
`Data/config/gameplay/<run>/index.html`; `gameplay.json` contains the same data.
The English page filters family, context address and text; details connect resources
to authenticated contracts and show each field's provenance. Captured source text
is preserved verbatim.

New reports use English descriptions for the observed fields and their limits.
The event-byte cap includes those descriptions, so wording length can affect how
many events fit; previously saved reports retain their original text.

<a id="cli-equivalente"></a>

## Equivalent CLI

Open using the runtime containing the tool; an older session retains its own
authenticated executable:

```powershell
.\recomp.exe debug-session open 'target\gameplay case' --paused
```

From a second terminal:

```powershell
.\recomp.exe debug-control gameplay-start 'target\gameplay case' --frames 2 --events 512 --family all
.\recomp.exe debug-control step 'target\gameplay case' 2
.\recomp.exe debug-control gameplay-report 'target\gameplay case'
```

Collection filters: `--family all|task|script|unit` and optional `--context HEX`.
The address is exact, without reconstructing an object pool. For a descriptor the
context is a ROM row, for a script a RAM pointer, and for graphics observations the
actor or task indicated by registers. Browser filters act on the collected prefix.
`status.gameplay_trace` exposes settings, frames, events, bytes, exclusions and stop;
`gameplay_report` identifies the latest HTML.

Start and Save require a paused copy without probes, pending steps or replay. Another
Start or separate replay requires Save first. State restore freezes the trace; origin
and current state may belong to different timelines. Closing without Save loses the
in-memory prefix.

<a id="campi-e-provenienza"></a>

## Fields and provenance

Start authenticates the exact European ROM and existing [task](rom_data_tasks.md),
[script](rom_data_scripts.md) and [unit/OBJ](rom_data_unit_obj.md) catalogs. Another
ROM is refused. Contracts and plans are identified by hash; no projection limit
becomes an additional game constraint.

- Descriptor load sites show nullable callbacks, context size and raw metadata,
  with ROM address and physical row. Primary and secondary selectors may alias;
  primary345 has a separate override. The descriptor callback is not presented
  as a final callback already scheduled.
- The primary script shows signed16 cursor at `context+0`, stream at `+12`, opcode
  byte and directory advance/callback; dispatch also retains the callback register.
  After storing the cursor it exposes the callback result: `0x110` repeats the
  original fetch. The collector does not normalize relative branches, execute
  callbacks or replace the interpreter.
- The auxiliary script retains action/slot at `+0/+1`, signed16 cursor at `+2` and
  stream at `+4`. Opcode zero terminates before dispatch: its ROM row exists but is
  not an observed call. Advance and wrapping remain guest behavior; action, slot
  and opcodes outside projections remain raw values.
- Verified graphics-task points show raw flags, sprite at `+6`, countdown at `+10`,
  sequence cursor at `+12`, frame count at `+14`, marker at `+0x20` and frame
  pointers at `+0x34/+0x38`. Reset entry also exposes sprite/animation/part arguments
  before reset. Sprite query/result retain actor pointer, raw register and low16
  derivation. Initial-pose resources are only correlations with animation0/part0.

Each field identifies register or RAM/ROM address/width, physical offset and
observed PC. Derivations such as low16 are declared. Immutable reads use backing,
distinguishing mirrors and ROM/RAM; I/O, protocols and non-RAM contexts are refused
with an unknown value. Unknown differs from zero/null, a nullable callback and a
candidate sentinel. `FFFF` sprite and `FFFFFFFF` marker remain raw: they do not
prove a valid asset or unit identity.

Other fields, general flag semantics, allocator, object lifetime, persistent unit
name/ID and entire pools remain unknown. A RAM address can be reused. A resource
link indicates membership in a projection, not proof of reachability, typed
consumption or acceptance of a new scene.

<a id="limiti-e-conservazione"></a>

## Limits and retention

Observation occurs before eligible interpreter instructions or native dispatch;
Halt, pending IRQ and HLE IRQ return are not mistaken for an instruction executed
at the PC. Native-block interiors are unobserved: missing points do not mean absent
activity. The normal runner remains when tools are disarmed; the observer does not
change guest fields, state format or ABI.

Limits: **30 host seconds, 60 frames, 512 events, 1 MiB event bytes**. Stop freezes
the prefix without interrupting a guest frame midway. Completed frames and active
input are recorded for the collected window; the suffix is unknown. At most eight
private directories, including incomplete attempts, and 2 MiB JSON. No overwrite
or automatic cleanup; a Save failure retains the frozen trace for retry. Output
paths must be ordinary directories. The collector creates no snapshot.

The report identifies runtime/build, ROM/BIOS, selected manifest/pins, initial/
current state, input/AV and actual dataset. Values and paths are private and
Git-ignored; review before sharing. The [homepage](report_home.md) finds them after
regeneration. Collection does not enable benchmarks or certify interpreter/native
parity or earlier ROM/DS checks.

<a id="validazione"></a>

## Validation

Actual validation and limits are recorded in the
[DBG-17 record](changes/2026-10-29-add-bounded-gameplay-inspection.json).

Locked/offline Windows feature/default builds passed without warnings.
**79 Wine tests** included affected checks and an optional local test: same seed/
interpreter, observer off/on for two A/release frames, complete state and clock
identical. The 70 prefix events connect entry, checkpoint and exit of the same task
address: countdown10→9→8, with RAM and frame pointers identified, without assigning
a persistent unit. No scripts were observed in this window; another scene need not
be rerun to close the requested sequence.

Default GUI passed 29 CLI invocations including navigation: CLI report 70 events,
menu 35, empty filter 0 with 35 excluded and selected context 3. Simultaneous code
collection retained its eight-event prefix and the expected complete frontend
endpoint. Start/Save left current state intact; full capacity preserved reports.
The six actually selected inputs/runtimes retained hashes/mtimes; the copy closed
with an inactive server. Chrome checked layouts 1400/620, filters, details and
contract links; the homepage's 55 indexes included five gameplay reports and the
coexisting code report. Screenshots inspected. No native Windows/Linux, physical
controller or native GUI capture validation; no earlier ROM/DS acceptance reopened.
Private evidence: `target/debug-tools-gameplay/`.
