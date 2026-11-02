<a id="selettore-integrato-dei-contenuti-debug"></a>

# Integrated debug content selector

The page requires [persistent Debug unlock](menu_clan_atlas.md#sblocco-permanente-di-debug):
**↑ ↑ ↓ ↓ ← ← → →** in the menu, or `debug = true` in `av.cfg` and restart.

DBG2-03 adds **Debug > Content selector**, a reusable page over the
[shared catalog](debug_content.md). A confirmed choice is a host reference:
selecting it starts no replay, script, state load or cheat. DBG2-04 migration
is in progress; completed checks remain separate from the new integration.

Opening the page shows the available index or an empty list. **Refresh**
explicitly updates declared roots in a bounded worker; the page stays navigable.
**Browse external file…** adds only the chosen file without examining neighbors.
**Add folder…** adds an explicit folder to scan roots. Only these two buttons
open OS dialogs; cancelling retains roots and current selection. On return,
the frontend consumes close/focus-restoration events for at least 400 ms and
two neutral controller polls, preventing Escape from also closing the menu or
a held confirm from starting other actions. `host_modal_input` in status
identifies this brief wait. The equivalent CLI command for indexing one file
is `debug-content scan --add-file FILE --json`.

The list offers search by name/ID/kind/path/description, category, availability
and compatibility filters, name/kind/recent sorting and All content/Favorites/
Recent collections. Identically named items remain distinct through ID and
path in details. Highlighting changes only the observed row; **Select** confirms
the choice after rechecking the catalog. Changed, missing, nonordinary,
unknown-kind or incompatible files are refused. An index-generation change
prevents using a previous row for a different file.

**Favorite** retains at most 512 IDs; Recent retains the last 32 confirmations.
Choices stay in memory in the current window: persistence and references after
reopening are DBG2-07. Returning from the selector retains the caller and the
confirmed choice. A selection still in progress is discarded when the page
is cancelled; a late result neither reopens it nor applies a choice.

**Details / preview** makes complete IDs, path, identity, status, reason and
messages scrollable. For snapshots with an embedded image, it uses the existing
readonly validator without loading a Machine; raw snapshots promise no image.
Hash rechecking precedes display. Metadata collection and preview do not
replace the actual action's preflight. The final message distinguishes an
authenticated preview, absent image and error; a response for a row that has
since changed does not replace the current preview.

Keyboard: arrows move focus, Tab changes area, Enter enters search or moves
the list to Select. Enter on Select confirms; Escape cancels the editor/details
or returns to the caller. Ctrl+F opens search. In search, characters, Backspace,
Tab, Enter and Escape are consumed by the menu before guest bindings and
debugger/global shortcuts. The editor accepts 128 Unicode characters and IME
preedit/commit; actual layout/composition and glyph coverage depends on platform
and available fonts.

Mouse: hover highlights; button clicks perform one host action; clicking the
list does not confirm a choice. The wheel scrolls the list or details.
Controller: directions, South/Confirm and East/Cancel traverse the same controls;
directions navigate filters, Confirm opens value selection and a second Confirm
applies it. Cancel discards the value being edited. Parameter-form choices use
the same behavior; search offers an on-screen keyboard with pages, space,
delete and Done. A held press must not produce multiple confirmations. While
the editor has focus, shortcuts do not reach the guest.

The page uses one I/O operation at a time, with index generation and view
identity. It retains catalog caps; details wrap text using the renderer's same
fonts, kerning and advances and bound the window to 16,384 characters.
`debug-control status` includes host `content_picker` status for reading count,
generation, pending operation and confirmed selection. This is not yet a new
execution command for the DBG2-13/15 console.

Actual checks, platforms and any open criteria are recorded in the
[DBG2 TODO](development/DEBUG_TOOLS_V2_TODO.md) and
[handoff](development/TASK_HANDOFF.md). The GUI check uses a public synthetic
fixture stopped at its starting frame, separate from DS scenes and personal
data. The three old FileDialog fallbacks are removed by DBG2-04.

DBG2-03 is verified with Windows GNU builds and Wine: all 55 menu, 10 catalog,
4 bridge, 3 contract, 2 UI-collector and 2 modal-return cases in their respective
scopes; 3 Python inventory and 10 packaging tests. The GUI exercises search,
identical names, a single held confirmation, favorites/recent, simulated XInput,
cancelling both dialogs, single-file import and preview at three sizes. The
first Cancel check exposed and retained a regression that resumed the synthetic
copy; after correction, the new check preserves complete state/frame0/clock0.
Native Windows, Linux runtime, physical controllers and native IME remain
unverified. [Full record](changes/2026-10-29-add-integrated-debug-content-selector.json).

DBG2-04 adds **Debug > Tool inputs / current selections**: replay/audit/capture/
compare profiles, save catalog, session, recording, snapshot, input sequence,
dataset, A/B runtime, native library, labels and known plan. Lists are role-bound
and choices retain hash, ID and ROM context. A file not hashed within budget is
refused with a request to narrow the root and refresh. A changed reference stays
visible and is refused before use; there is no fallback to another file or OS
dialogs. The same page includes **Cheat reference catalog**, with its internal
selector bound to the cheat catalog. Confirming the reference and cancelling
the selector keeps the cursor on that row; it neither applies nor activates codes.

Choose replay profile and Choose catalog open the internal page. Replay, Audit,
Capture, Compare and Create issue without a profile open the same page with the
missing requirement: selection does not resume the action automatically. Return
to the menu and start explicitly. CLI references and the default catalog are
authenticated before use. Selected labels feed the code trace; inputs for
preparing new profiles are separate from immutable payloads of already
authenticated profiles (DBG2-05 editor).

Typed selection checks format and identity using existing readonly services
for profiles, catalogs, sessions and recordings; snapshots and sequences are
checked without loading them into the current machine. Labels are parsed for
the ROM; manifests check schema/ROM. Runtimes and libraries remain hashed
references: the worker must still check format, features and ABI without
loading code on simple selection.

Wine validation exercises the twelve roles, seven entries without input,
explicit starts, ten refusals for altered/missing profiles and different ROM,
and six catalog Choose/Open refusals. The trace confirms one dialog per
explicit button and Cancel retains all choices. The main process in accepted
checks stays at frame zero; synthetic workers and an earlier harness-navigation
error are documented separately in the
[DBG2-04 record](changes/2026-10-29-migrate-debug-tool-input-selection.json).

DBG2-05 adds **Debug > Parameters / prepare profiles** with drafts for replay,
audit, capture, comparison, steps, inspection, audio, code, gameplay, recent
events, graphics and issues. Input rows use the typed selector; arrows move
focus; Confirm opens choice editing or the text editor with on-screen keyboard.
A choice changes the draft only on the second Confirm; Escape discards the value
still being edited. Escape cancels the editor or returns from the panel;
Tab changes field/action.

**Verify draft** uses shared parsers/preflight without execution. **Save new
preset** retains only scalar values in portable configuration:
`debug-presets/<tool>/`, at most eight 16 KiB files per tool and 8 KiB of values.
**Load last preset** restores a draft without changing references or starting
tools. Missing/changed inputs and unsupported combinations have readable
refusals. Late responses do not overwrite a reopened draft.

**Prepare NEW profile** creates a new copy in `debug-profiles/`, up to eight,
with authenticated parameters and inputs; it neither overwrites an existing
profile nor starts replay. **Run prepared profile** runs that separately pinned
profile: changing the draft requires new preparation. For local tools,
**Start / apply** passes parameters to the existing handler; arming an observer
does not automatically advance the game. Steps, resumption and reports remain
explicit. Options include timeout/frames/input/ranges/filters/streams/triggers/
comparison mode/symptom; selected labels remain references with rechecking.

Preparation can read immutable pins of the same already open session,
recognizing its root/PID/runtime lease without granting another writer. Other
active sessions remain refused. Profile validation does not promise arbitrary
executable ABI/features; the worker retains final preflight. Actual checks are
in the [DBG2-05 record](changes/2026-10-29-add-debug-parameter-workbench.json).

DBG2-05 is verified with 121 distinct Rust cases in their Windows/Wine scopes
and the GUI chain recording → one explicit step → replay/audit/capture/compare
preparation → separate start → report. Four profiles created with the same CLI
parameters match in JSON and outcome/state/clock. The punctuation page allows
commas and separators through the controller; incomplete parameters are refused
before execution. Initial checks corrected an unpublished canvas and overlapping
badge; negative images and logs remain separate from final checks at three sizes.

## Script and tool library (DBG2-06)

`Debug > Scripts and tools` separates declared host adapters, readonly command
recipes and guest script metadata. Selection, hover, import and Details never
launch a process. The form has Details / verify, Start explicitly, Cancel
process and scalar presets; Back leaves a running job visible in host status.
Cancel remains available while the worker is busy. Closing the application
requests cancellation and waits for its supervisor.

The v1 adapters call the public Python tools directly, using argument arrays:
report homepage (`--root` selected session, `--stdout`), campaign `report`
(required explicit case ID), artifact `preview` (selected closed session). They never
invoke campaign replay or artifact `apply`. Select an explicit Python3.11+
executable in the internal picker; absence is reported without installing it.
PowerShell/Bash wrappers and Docker are not used by these direct adapters.
Version is checked only at Start; Details hashes the executable without running
it. The runtime identity is the selected executable hash, not a claim about
all installed Python modules/DLLs.

Embedded public scripts, helpers and HTML templates are hashed/versioned in
Details and staged into each new run only on Start. With source=`external`, the host-script
reference must have the exact declared filename and bytes; sibling dependencies
must match too. Modified code is refused even after reselecting it. Source=`embedded` ignores
a previous external selection, so switching adapters does not inherit code. The copied
known bytes are executed, avoiding a changed external script between checking
and launching. Python uses explicit UTF-8, isolated/no-site/no-bytecode mode and a constant
bootstrap with source paths supplied as separate argv.

Outputs are private under `config/debug-tool-runs/<new-id>/`: request/result
JSON, per-process stdout/stderr, and homepage HTML or the existing report folder.
The root homepage is never replaced. Eight runs are retained without automatic
pruning, each has a20..120000ms deadline, and each process stream is capped at
8MiB; overflow stops the process with `log-limit`. Cancellation/timeout retains
partial logs and a distinct outcome. Windows starts suspended, assigns a kill-on-close job before resuming the
process;
Unix uses a dedicated process group. Descendants are stopped before collecting
pipe EOF. Containment failure refuses the launch and stops its child.

The initial recipe schema is `schema:1`, `kind:ffta-debug-recipe`, a printable
label of1..200bytes and1..16 `steps`. A step is either
`{"operation_id":"control.status"}` or
`{"operation_id":"control.inspect","address":"08000000","length":16}`.
Unknown fields, other operations and invalid CLI arguments are refused.
Start invokes the selected recomp runtime against the selected session, with
per-step JSON stdout; this readonly subset advances no guest frames. The
broader recipe engine remains DBG2-17.

The separate Guest scripts view reads bounded schema1 metadata or actual typed
extractor projections (without altering their original files), source
hash and declared entry counts. It makes no new verification claim and never
executes its bytes. Gameplay inspector opens the existing parameter form;
starting its observer still requires authenticated supported ROM/models.
Validation: Windows GNU build passes without warnings. Wine covers80distinct
Rust behavioral cases (including the explicit UTF8 proof) and3Python inventory
checks. Actual GUI exercises four explicit readonly runs, typed selections,
script/dependency/runtime refusals, simulated XInput cancellation and confirmed
descendant stop, timeout, guest catalog/inspector link and three client sizes.
The main full state remains at frame0/clock0. Final encoding proof uses the same
closed public source and a new output only; original reports stay retained as
negative locale-encoding evidence. Final HTML preserves the Unicode path in
UTF8. Source/runtime/input/output preservation passes; all GUI jobs are closed.
Evidence: `target/debug-tools-v2-library/`. Native Windows/Linux, physical
controllers and wrapper/Docker execution remain unverified.

## Persistent choices (DBG2-07)

`Data/config/debug-choices.json` follows the selected config root. It stores
app/config-relative internal locations, explicit external locations and added
file/folder provenance, typed input SHA/kind pins,18scoped views (all plus17roles)
and global favorites/recent/highlight bookmarks. Limits:256KiB,8roots,17input
roles,512favorites and32recent entries, plus per-view highlights. Internal paths
never store the installation directory. External paths retain their explicit
location; moving the app does not silently redirect them.

At the first paused menu context, load bounded metadata and asynchronously
check saved primary files with a64MiB budget; the existing type-specific readonly
preflight limits also apply. Directory scans, OS pickers and tool starts stay
explicit. Missing, changed or incompatible roles remain saved with explanations;
valid references alone become usable. Each action still rechecks them. A new
ROM/context re-preflights the bindings, retaining neutral runtime choices while
refusing incompatible labeled/snapshot/resource inputs. Each isolated storage
root owns its preference file.

Views save committed search text, type/state/collection/sort and highlight.
Favorites/recent are remapped by location when an explicitly added internal
root changes ID after a folder move; restoring them does not confirm or run an
item. Incomplete editor text stays a draft. Debounced paused writes and exit
flush use same-directory atomic publication, preserve detected external edits
and leave unknown/invalid-schema files untouched. Late restore results cannot
replace a newer explicit pin or apply to another ROM/storage context.

Windows GNU build passes without warnings;18distinct Wine cases cover schema,
path/pin/atomic storage, scoped history/root rebasing and picker lifecycle.
Actual GUI saves four roles and internal/external roots, then reopens the same
public seed after a Unicode/spaced folder move from `/tmp`. Runtime/snapshot
remain usable; missing labels and a changed external file are explained while
all four original pins remain saved. Query/favorites/recent/sort and the rebased
highlight return without automatic directory scan, dialog, confirmation or
execution. Full main state/frame0/clock0 remains exact. The initial verbatim
Windows drive-prefix failure and its volatile GUI are retained as negative
proof; the corrected guard checks rooted directories rather than the volume
prefix alone. Native Windows/Linux and physical controllers/IME remain
unverified. Evidence: `target/debug-tools-v2-choices/`.
