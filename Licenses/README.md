# Licenses and third-party notices

The project's MIT license applies to project-owned changes. It does not replace
the licenses of upstream code, dependencies, fonts or the TinyCC compiler.

## What the pending notice entries mean

THIRD_PARTY_NOTICES.md inventories the locked Rust dependency set, including
optional platforms, build tools and tests. Entries marked `No notice file
packaged` identify packages whose original notice file was not recovered in
that inventory. That marker does not by itself establish that a package is unlicensed, and
it does not imply that all such packages are shipped in every binary. Their declared license, version,
authors and source link are recorded. Applicable original notices still need
to be checked/recovered for the components actually distributed; this note is
not a substitute for missing notices and does not claim the audit is complete.
The latest comparison found 133 marked entries among 726 locked dependencies.

## Where the TinyCC notices live

- TinyCC/COPYING: the LGPL text supplied with the pinned TinyCC source.
- TinyCC/LICENSE-ADAPTER-MIT.txt: the project's desktop adapter license.
- ../TinyCC/: exact upstream source.zip, adapter, build script and replacement
  instructions. The original upstream license files remain inside source.zip.

Desktop builds put the notice folder at payload/Licenses/TinyCC and the source
bundle at payload/TinyCC. Notices are grouped here rather than shown as installer
screens or left as loose files beside the compiler source. They accompany the
library distribution; this organization does not remove its license terms.
The installer copies payload/Licenses into the installed game directory.
Android already groups its notices and corresponding TinyCC source inside
assets/licenses/tinycc in the APK; they are not installation dialogs.

The command console embeds Droid Sans Fallback for CJK glyphs. Its upstream
notice and Debian copyright record are in `fonts/Droid-NOTICE.txt` and
`fonts/Droid-COPYRIGHT.txt`.

Keep the font notices in fonts/ and the applicable dependency notices with
binary distributions too. A repository link alone is not a replacement for
required accompanying notices. No ROM, BIOS, save or generated game library
belongs in this folder.
