<a id="breakpoint-watchpoint-e-memoria--dbg-11"></a>

# Breakpoints, watchpoints and memory — DBG-11

These tools operate on the [paused isolated copy](debug_control.md).
They do not write RAM values or patch code. Only one command can await a
result from the local `debug-control` channel.

<a id="leggere-registri-e-memoria"></a>

## Read registers and memory

```powershell
.\recomp.exe debug-control inspect 'target\case' 080003d6 32
.\recomp.exe debug-control inspect 'target\case' 02000000 256
```

The address is hexadecimal; length is decimal (**1–256 bytes**). The report
includes r0–r15, CPSR and ARM/Thumb mode, frame/clock, bytes, region and physical
offset for each address. **Debug > Inspect PC / registers** reads 16 bytes from
the current PC. The latest inspection is in
`Data/config/debug-control/inspection.json` and the command's JSON response;
`status` can retrieve it along with current status. Frame and clock in the
`inspection` section identify the read point, which may precede the current
state after another command. No additional snapshot is generated.

Inspection uses immutable buffer references: BIOS backing, EWRAM/IWRAM,
palette, VRAM, OAM and ROM backing. It explicitly exposes mirrors and VRAM
folding without calling Bus functions. **I/O, open bus and save protocols are
refused**, including keypad/timer and Flash/EEPROM. ROM shows backing bytes
even where the normal bus would address GPIO/EEPROM: the report does not
simulate these devices or BIOS protection. Out-of-buffer windows and wrapping
addresses are refused in full.

Optional labels come from `debug-labels.toml` in the copy's root:
[gba-labels](labels.md) format, the same ROM's SHA, declared names and bounds.
A label identifies a declaration; its `end` does not establish executed code.
A file from another ROM is refused. The report records the hash of the same
bytes parsed; it does not reread the file to interpret a different version.

If the copy has an authenticated dataset, owners are intersections with its
manifest's source ranges. Overlapping ranges are authenticated against the ROM;
manifest identity, ID, offset, length and role remain in the report. No match
means **unknown owner**. These are identified data: they do not prove a typed
read, reachability or new ROM/DS acceptance. For reads actually served, use
[the DBG-06 audit](debug_rom_audit.md).

## Pinned observations — DBG2-25

In an isolated paused session, the command console or `debug-command` can pin
up to 16 session-local observations. Labels and owners are explicit; neither
identifies a persistent guest object. For example:

```text
pin-add pc analyst register 15
pin-add bytes analyst memory 0x02000000 8
pin-add timer analyst field unit countdown 0x02000000
pin-view changed 0
pin-refresh
pin-remove timer
```

The equivalent local-control commands put the session path after the command,
for example `debug-control pin-add SESSION pc analyst register 15`. A register
index is 0–15; a memory window is 1–32 bytes of supported immutable backing.
I/O, open bus and backup protocols are refused. The EUR-authenticated decoded
fields use the existing task descriptor, primary/auxiliary script and unit
context offsets described in the gameplay inspector. Add them while paused at
the matching inspector site with its context register equal to the supplied
address. This establishes the initial observation, not a persistent object
identity or lifetime; a later allocation can reuse the address.

The console keeps up to three pins visible, with previous and current values,
owner and source; amber marks a change and red marks stale context. `pin-view`
selects `all`, `changed` or `stale` and a zero-based offset. Full source,
frame and clock for both samples are available in the returned status JSON.
A completed step
refreshes the pins once at its endpoint; `pin-refresh` explicitly samples an
idle paused machine. Load, undo, restart or save restoration marks all pins
stale and prevents refresh: remove and recreate them only after checking the
new context. A ROM change starts a new session and does not carry pins across.
Creating, refreshing and displaying pins read backing data only and do not
write guest state. The feature remains subject to its recorded validation scope.

<a id="fermarsi-su-pc-e-modo"></a>

## Stop on PC and mode

```powershell
.\recomp.exe debug-control break 'target\case' 080003d6 thumb --steps 100000
```

The PC must be aligned for `arm` or `thumb`. Search starts at the current point
and stops **before the interpreter step** when PC and mode match, provided that
step is eligible to execute an instruction: Halt, IRQ delivery and BIOS HLE
return stubs are not presented as target instructions. A match at the current
point executes zero steps and preserves guest state. The debugger does not
evaluate the opcode's CPU condition: a conditional instruction that does not
execute its body is still an instruction entry.

<a id="fermarsi-su-accessi-bus"></a>

## Stop on bus accesses

```powershell
.\recomp.exe debug-control watch 'target\case' read 03007ffc 03008000 --width 4 --steps 100000
.\recomp.exe debug-control watch 'target\case' write 02000000 02000004 --width 4 --value 00000001 --steps 100000
```

The copy's runtime needs the **rom-data-read-audit** feature. The `START END`
range uses guest addresses and an exclusive end, without normalizing mirrors.
Select `read` or `write`; `--width 1|2|4` and `--value HEX` filter the observed
bus call and its value. A value must fit the selected width. The debugger does
not read an I/O address to check the value: it compares the access actually
performed by the game. For writes, this is the value requested from the bus,
even when the device ignores or transforms the write.

The first match is retained, excluding interpreter fetches. Execution stops
**after the complete interpreter step**, including HLE, DMA and hardware ticks
started by that step. The `site` section describes the point before the step;
it does not automatically attribute the access to an opcode load/store.
Accesses implemented through nested bus calls may appear at the internal width
if `--width` is omitted. The first matched access remains the one declared in
the report. Direct host accesses to vectors, native fetches and code inside
native blocks are not intercepted by this watchpoint.

In two isolated EUR Wine/Xvfb sessions started from the same seed, local-control
CLI and the interactive console reached the same Thumb breakpoint with identical
search results and endpoint state. On the default build, both refused the same
watch request with the same feature-unavailable error before changing guest state.
This comparison does not establish a live watch hit in an audit-enabled build.

<a id="named-searches--dbg2-24"></a>

## Named searches — DBG2-24

The in-game command console and `debug-command` manage multiple named searches
in the isolated session. Saving a definition creates it disabled and does not
run it; enabling also does not run it. For example:

```text
trigger save entry 'break 0x080003d6 thumb --steps 100000 --hits 2 --frame-start 0 --frame-end 5 --on-hit pause'
trigger list
trigger edit entry 'break 0x080003d6 thumb --steps 100000 --hits 3'
trigger show entry
trigger enable entry
trigger run entry
trigger retry entry RUN REQUEST-ID DEADLINE-UNIX-MS
trigger disable entry
trigger remove entry
```

Names use 1–48 ASCII letters, digits, hyphens or underscores. Save never
overwrites; editing an existing definition atomically disables it until explicitly
enabled again. Invalid edits leave the previous definition intact. At most 32 are
stored under the session's `Data/config/debug-triggers/`. Each is bound to its
session manifest, ROM and runtime pins. A changed manifest or a disabled
trigger is refused. Disabling does not cancel a search already dispatched.

`break` chooses PC and mode; `watch` chooses a read/write range and optional
width/value. Both accept `--hits 1..10000` and a relative frame interval
`--frame-start N --frame-end N`, with start inclusive and end exclusive within
0..600 completed frames from search origin. The defaults are one hit and the
full window; omitting these options preserves the earlier serialized format.
A PC hit counts an eligible instruction entry; a watch hit counts a complete
interpreter step containing a match, retaining its first matching bus access.
Multiple matches within one HLE/DMA step count once. The full step is attributed
to its starting frame, including when it crosses a frame boundary.

For a simple conjunction, add all three options `--when-address 0xADDRESS
--when-width 1|2|4 --when-value 0xVALUE`. The guard compares aligned canonical
EWRAM/IWRAM backing bytes in little-endian order, sampled before the candidate
interpreter step. It does not perform bus or I/O reads. The result reports the
candidate and matching counts, last observed guard value, interval and stop
reason. Other combinations are not implied.

`--on-hit pause|capture|freeze` chooses the explicit action. Pause stops at the
search boundary. Capture completes the current frame with neutral input when
needed, then publishes `capture.ffstate` and `capture.json` under
`Data/config/debug-trigger-captures/`; it may advance one additional frame beyond
the search window. The metadata identifies the session, ROM, selected condition,
hit, final boundary, authenticated trigger name/definition hash and state hash. This is a derived unvalidated snapshot,
without the recording/input bundle provided by [DBG-08](debug_captures.md).
Capture after cheat writes is refused until cheat provenance is supported.
Freeze requires an active recent-event buffer and freezes it at the hit; the
existing `recent-report` command saves it. On a miss no action runs. Failed
actions are reported as failures, including any already completed frame.

Run uses the same idle-paused, recording/replay, backend and observer checks as
the underlying search. PC stops before an eligible instruction; watch stops after
the complete interpreter step including HLE/DMA/hardware work. Watch requires
`rom-data-read-audit` and an exclusive bus observer. Native instruction-level
search is refused because block dispatch has coarser granularity. No background
rearming or automatic retry occurs. An explicit `trigger retry` requires the
original run, request ID and deadline and the unchanged enabled definition; it
uses the existing idempotent request ledger. The named result includes a
definition hash and the original request/result identity; `trigger show` presents the definition,
boundaries and observer constraints. The console also displays the evaluated hit
and action; Ctrl+Shift+C copies the complete JSON. New capabilities are checked
before sending extended settings to an older runtime.

DBG2-24 is accepted in the recorded interpreter/Wine scope. Windows GNU/Wine
focused tests cover legacy serialization, bounds, capability
refusal, PC count/window, guarded reads, feature-gated watch counts, capture
publication and command/recipe regressions. In isolated EUR Wine/Xvfb sessions,
a Thumb trigger reached its second hit, a second trigger missed within its limit,
a PC/RAM guard froze recent events, and a hit produced a one-frame-boundary
snapshot matching the live state hash. No snapshot was created on a miss.
Simulated keyboard input in the game console created, enabled, ran and disabled
a named trigger. Protected inputs remained unchanged and the windows closed
normally. These checks do not establish native Windows, physical-controller,
other-ROM or full device-observer coverage.

<a id="limiti-ripresa-e-interruzione"></a>

## Limits, resumption and interruption

Both searches require a paused copy, interpreter, no frame budget and no
active recording or replay. The native backend is explicitly refused: no
instruction-level stop inside its blocks is claimed. Readonly inspection can
instead describe a stopped native copy without executing it.

`--steps` accepts **1–5,000,000**, default **100,000**; there is also a limit
of **600 completed frames**. Search advances in batches of at most 2,000 steps,
keeping the menu responsive. During search the sidebar counter shows remaining
steps; in DBG-10 advances it shows frames. Guest input is neutral. A match
returns `probe.outcome:
"hit"`; exhausting the limit returns `"not-reached"`. Both complete the command
with transport success and leave the copy paused. Read the JSON to distinguish
the outcomes. `status.probe` retains the latest completed search; an incompatible
command does not replace it. For the current request's outcome, read `ok/error`
and its identity without confusing it with that earlier result. Frame, clock
and snapshot hash describe the actual stop point, which may be inside a frame;
it is not a fabricated complete frame.

F6/Escape, Resume, Load/Undo/Restart and closing can interrupt a search.
A pending command receives the interruption; the 120-second client timeout
means the wait expired, not cancellation. There are no automatic retries.
Resuming or normal Step 1 completes emulation from the current point without
rewinding. Guest taps produced in a partial frame remain until the next
complete frame; host audio stays silent without backlog during search.
Displayed images are the last completed frame, not an invented rendering of
the partial point. Settings and explicit timeline changes in the copy remain
under user control.

If Windows temporarily prevents replacing the result because a client is
reading it, the server retains the same response and retries publication on
subsequent ticks before consuming more requests. It does not reexecute the
command. On close, the final publication attempt is bounded to about half a
second; persistent I/O errors remain reported through client timeout and logs.

Files remain private in the Git-ignored copy. Only the latest inspection and
response are retained, without dump history. The isolated candidate runtime
is under `target/debug-tools-probes/`; the normal executable stays separate.

<a id="verifiche-del-28-ottobre-2026"></a>

## Validation on 28 October 2026

Windows release feature/default builds without warnings from `07be4d2` plus
eleven selected files, including accepted DBG-10 foundations and excluding
concurrent extraction code. Wine checks cover **88 distinct tests**: five
search/inspection, three readonly/watch core, 25 bus regressions, six
control/channel, 46 menu and three label-parser tests. Runs: 84 initial,
eight targeted after parsing from the authenticated buffer and six controller
tests after response correction. The checks do not change Machine, saves or native ABI.

**21 CLI/GUI checks** on the DS-20 copy verify readonly 256-byte reads, RAM
mirrors, registers/labels and refusal of I/O/backup/wrap/length/foreign ROM
without changing full hash/frame/clock. Three real triggers pass:

- Thumb PC `080003d6`, before the step after 66,720 search steps, clock
  6,241,285,276 in guest frame 22,219. A second match at the current point
  executes zero steps and preserves state exactly;
- word read `03007ffc` with value `03000f10`, stopping after the IRQ-service
  step: the `site` section reports `eligible_instruction: false`;
- word write `03007f78` with requested value `6000003f`, stopping after the step.

A one-step limit without a match, stationary waiting at the partial point and
subsequent completion of one frame also pass. Synthetic tests compare complete
state against unobserved execution, exclude fetches and verify owner provenance
and refusal of altered sources. These synthetic owners do not constitute new
acceptance of real resources.

**Nine final checks on the default build**, plus two controlled searches,
verify PC/inspection without audit, watch refusal without the feature, hash of
the same parsed label source, readonly menu, recording/replay refusal during
search, F6 cancellation and owner shutdown with confirmed interruption.
Screenshots were inspected. All **202 checked original inputs** and normal
Windows/Linux executables retain hashes and timestamps.

The first guard check pressed Return before the Wine client delivered the
request; recording started legitimately and search refused it. It is retained
as a test-ordering error, without repeating the 21 completed checks. A later
check found a real Windows-channel race: publication of `result.json` denied
while the client read the old file, with an actual 120-second timeout. The
server now retains the response for republication, and a test forces failure
and recovery without reexecuting the command. Final checks pass even when the
log detects a transient refusal. The Cargo attempt with `--lib` on the
binary-only package was refused before compilation; only the test command
was corrected and the log retained.

Final verification authenticates the eleven overlays, candidates and originals;
it confirms hooks retain original bus expressions and the search loop is
identical to the three accepted checks. Results/preimages/attempts remain in
`target/debug-tools-probes/`. No managed publication directory or browser profile
was created: the associated ROM cleanup does not apply.

Native Windows, Linux/Wayland, physical controllers, other scenes/ROMs and
native search remain outside validation. The 600-frame and 5,000,000-step limits
are checked in code and configuration bounds; no real search of that duration
was completed. A forcibly killed process and persistent I/O errors remain
separate from verified clean shutdown. No benchmarks, new ROM/DS acceptance,
native repairs or Godot work were performed.
