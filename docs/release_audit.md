# Source upload audit

Date: 2026-10-12. This is a filesystem/source audit, not legal certification.

- MEASURED: public working files scan reports zero suspect ROM/save/dump files,
  FFTA ROM headers, exact-ROM hashes or generated game C/library artifacts.
  .git and target are excluded from the public set.
- MEASURED: an additional scan including target examined 31,510 files and
  reported zero findings; only .git history was excluded from that scan.
- MEASURED: both gamedb.sqlite copies contain function metadata tables only:
  SHA256, addresses, end addresses, modes and names. No ROM-byte column exists.
- MEASURED: literal FFTA disassembly was removed from ffta_sprite_routines.md
  and ffta_estado.md. Host implementations, addresses, hashes and short original
  byte checks are retained; runtime hooks validate the user's ROM/RAM.
- MEASURED: source packaging excludes ROMs, saves, dumps, native binaries,
  generated artifacts, toolchains, backups, target and .git. It refuses a
  detected FFTA ROM header or generated game C file in the selected source set.
- MEASURED: 726 locked Cargo dependencies are attributed, with 313 distinct
  notice texts preserved. Eleven dependencies have declared licenses but no
  recoverable original notice file; their entries explicitly say so and link
  to the published package. No copyright notice was fabricated.
- MEASURED: controller mappings have a separate zlib notice. Upstream
  gba-recomp copyrights, MIT/Apache terms and gamedb CC0 terms are retained.
- MEASURED: input-config and menu tests: 26 passed, zero failures. Windows and
  Linux release builds completed after the default gamepad mapping changed.
- NOT MEASURED: physical controller behavior for the new default bindings.
  Existing saved configurations are preserved; Default layout applies the new
  trigger/bumper settings and resets the gamepad zoom bindings.
- NOT MEASURED: provenance/ownership of project-supplied logo and icons. These
  are not covered by a claim of ownership arising from the software licenses.

Reproduce with scripts/audit_release.py and scripts/package_source.py.
For manual GitHub uploads, extract target/github-upload/FFTARecomp-source.zip
and upload its contents. Do not upload the working directory, build caches,
downloaded toolchains or private saves by dragging them into the browser.
