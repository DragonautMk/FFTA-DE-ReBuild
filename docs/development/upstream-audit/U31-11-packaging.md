# U31-11: Installer, build and release packaging

Status: **source comparison; SKIP wholesale import and DEFER absent a specific
verified defect**. Upstream `6bdddbc559ac459c4a18325c15c28d040d87f407` is
compared with fork `decomp` `83f245232c7dbd0206d4fea2a556e031e98fbf37` on
2026-11-02. The tag distinction is documented in
[U31-02](U31-02-scrollbars.md). No package or build was produced or inspected.

The upstream comparison adds `scripts/package_release_outputs.py`, which copies
specific Windows runtime files, existing Linux/Deck archives and optionally an
Android debug APK into `Installer`, then writes SHA-256 sums and a JSON build
manifest. The selected Windows ZIP paths are enumerated in source; this is not
evidence that the referenced prebuilt executables or Linux/Android artifacts
are current, correctly licensed, private-data-free or reproducible. The script
writes canonical destinations and is not a read-only checker.

The upstream Linux packaging changes build a separate `steamdeck`-feature
`recomp` payload for North/Y, discussed in [U31-09](U31-09-steam-deck-y.md).
Its changed tar command names an explicit set of payload paths; the same diff
removes the prior commands that copied `TinyCC`, `Licenses` and README into that
archive. This comparison concerns the script's listed contents; it does not
certify what any released archive actually contains. The fork installer README
already describes a Deck installer presentation, while the fork's game build
does not have upstream's Y feature. Region-facing installer text differs; see
[U31-12](U31-12-rom-compatibility.md).

The fork has separate Windows/Linux build and portable packaging tools, with
`package_ffta_portable.py` and synthetic tests that inspect a whitelist,
manifest, archive structure, modes and rejection/preservation paths. The
existing [portable guide](../../portable.md) and relevant test/validation
records describe their own scope. These tools and tests were inspected only;
no current package or successful build follows from their existence.

Preserve the fork's existing distribution workflow, notices and source/license
materials, user-data exclusions, target paths and platform-specific results.
If a concrete packaging defect is discovered, compare only the affected
script and prepare a separate fix with synthetic fixtures: missing/stale inputs,
archive contents and modes, atomic failure/preservation, privacy exclusions,
licenses and correct checksums. Confirm real Windows/Linux artifacts and Deck
behavior separately before making release claims. Do not run the upstream
packager against canonical output merely to inspect it.
