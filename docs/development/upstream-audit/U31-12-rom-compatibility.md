# U31-12: Regional ROM compatibility constraint

Status: **source comparison; SKIP importing the upstream USA exclusion as fork
policy**. This is a compatibility constraint, not proof of either region's
complete runtime behavior. Compared upstream
`6bdddbc559ac459c4a18325c15c28d040d87f407` with fork `decomp`
`83f245232c7dbd0206d4fea2a556e031e98fbf37` on 2026-11-02. See
[U31-02](U31-02-scrollbars.md) for the tag distinction. No ROM was read and no
build or runtime check was run for this comparison.

## Policy, validation and feature gates are distinct

The upstream release notes say USA is temporarily unsupported, and the pinned
installer and Android onboarding text instruct the user to provide an EUR ROM.
That presentation must not be mistaken for a universal technical rejection:

- Upstream `gba-core/src/ffta_rom.rs::version` still recognizes exact USA and
  EUR SHA-256 identities. Its `eligible` also requires the applicable sprite
  profile and map-producer verification; identity alone is not patch eligibility.
- Upstream installer `KNOWN_SHA1` includes both regions and `rom_policy::plan`
  checks both a known SHA-1 and `eligible`. Unsupported or altered ROMs of
  plausible size receive an unpatched plan under the current `false` refusal
  setting. Its warning now says EUR, even though the code's known list has both.
- Upstream desktop `validate_rom` checks size and warns about an unusual entry
  byte; the separate in-game `rom_switch::validate` requires a known SHA-256
  and checks any package pin. These are different entry points and guarantees.
- Upstream Android `engine::verify_rom` checks both exact SHA-1 and SHA-256 for
  USA or EUR despite the EUR-only onboarding copy. This checks identity, not
  successful gameplay or every new feature.

Conversely, some new upstream behavior is explicitly Europe-specific: for
example `autosave.rs::shared_loader` and `is_saving` require `europe_profile`
and particular callbacks. Other `ffta_battle_mouse` paths have their own
region/profile checks. Thus retaining two recognized hashes does not disprove
the release's support warning or establish feature parity.

## Fork requirements

The fork's corresponding desktop core version lookup recognizes both exact
regions, its installer contains both known SHA-1 entries and uses `eligible`,
and its Android engine has the same two-region identity check. The fork
installer's current warning mentions USA and Europe. Desktop generic `play`
validation and the stricter ROM switch retain their distinct meanings. Fork
ROMDATA work has explicitly scoped EUR evidence and open USA work in the task
handoff; recognizing USA does not convert that evidence into USA acceptance.

Any proposed port must retain the fork's existing region-specific checks,
source identities, hook and native-cache constraints. Do not copy EUR-only
onboarding/warnings, hard-coded callbacks or unconditional feature paths into
the fork as though they applied to USA. A feature may be disabled for an
unverified region when necessary, with its scope stated and existing behavior
preserved; recognition, installer eligibility, creation/reuse and actual
runtime functionality must be checked independently.

Source test inventory includes fork `version_for_sha256` identity tests,
installer refusal tests and optional user-ROM checks, and Android arbitrary
file rejection. Their assertions were inspected, not rerun. Acceptance for a
specific integration requires exact EUR and USA inputs with region-specific
positive and negative guards, relevant installer/launch/feature behavior,
state and save preservation, native and interpreter paths as applicable, and
results for each claimed platform. The absence of a USA-only runtime result
must remain explicit; no unsupported-ROM test authorizes changing user ROMs.
