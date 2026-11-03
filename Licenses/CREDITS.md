# Credits and licensing

| Component | Credit / upstream | License / distribution |
| --- | --- | --- |
| This fork’s original contributions | Michele Savarese and contributors | MIT; see LICENSE |
| Original FFTARecomp changes | [Jorge.R and contributors](https://github.com/Jorge262007/FFTARecomp) | MIT; original copyright notice retained in LICENSE |
| gba-recomp foundation | [JRickey](https://github.com/JRickey/gba-recomp) | MIT OR Apache-2.0; original notices retained in LICENSE-MIT and LICENSE-APACHE |
| gamedb.sqlite | JRickey | CC0-1.0; function addresses, lengths, modes and names only; see gamedb.sqlite.license and LICENSE-CC0 |
| Gamepad abstraction | [GilRs project](https://gitlab.com/gilrs-project/gilrs) | MIT OR Apache-2.0; see THIRD_PARTY_NOTICES.md |
| Windows API bindings | [Microsoft windows-rs](https://github.com/microsoft/windows-rs) | MIT OR Apache-2.0; see THIRD_PARTY_NOTICES.md |
| Controller mappings | [SDL_GameControllerDB contributors](https://github.com/mdqinc/SDL_GameControllerDB) / Sam Lantinga | zlib; see LICENSE-gamecontrollerdb.txt |
| Graphics, audio and window libraries | wgpu, winit, softbuffer, CPAL and their contributors | Exact versions, authors and license texts in THIRD_PARTY_NOTICES.md |
| Installer UI | egui/eframe contributors | MIT OR Apache-2.0; bundled font notices are included in THIRD_PARTY_NOTICES.md |
| Runtime C compiler (Windows, Linux, Android) | [TinyCC contributors](https://github.com/TinyCC/tinycc) / Fabrice Bellard | LGPL; the source notices permit LGPL 2 or later and COPYING supplies LGPL 2.1. See ../TinyCC for exact source, platform build script, adapter and replacement instructions. |
| Windows build toolchain | [w64devkit](https://github.com/skeeto/w64devkit) / Christopher Wellons and individual tool authors | External download; GCC/binutils and other components retain their own licenses. Distribute their accompanying notices and applicable source with a toolchain release. The source upload does not bundle the toolchain. |
| Linux cross compiler | [Zig contributors](https://ziglang.org/) | Build-only download; MIT plus bundled LLVM/runtime notices. Not included in the Linux installer. |
| Logo and icons | Project-provided artwork | Artwork provenance is not established by the code-license audit; the software licenses do not grant rights to third-party trademarks or game artwork. |

## Windows controller APIs

XInput, Windows Gaming Input and DirectInput are Windows APIs, not third-party
game source. GilRs uses Windows Gaming Input by default in this build; the
project also has its own DirectInput adapter using windows-rs. Xbox-style
button labels do not mean the XInput backend is enabled. No XInput or
DirectInput runtime DLL is copied into the source upload or installer payload.
System DLLs are supplied by Windows; their terms remain Microsoft's.
See [Microsoft's XInput version documentation](https://learn.microsoft.com/en-us/windows/win32/xinput/xinput-versions).
Linux gamepad input uses GilRs/evdev rather than XInput.

## Game ownership and repository contents

Final Fantasy Tactics Advance and related names are owned by their respective
rightsholders. This fan project is not affiliated with Square Enix or Nintendo.
Users must supply their own ROM. ROM images, BIOS images, saves, RAM/VRAM dumps,
generated game C/assembly and translated game libraries must not be uploaded.
Own host code, function addresses, hashes and short verification signatures
are retained to validate hooks against the user's ROM.

THIRD_PARTY_NOTICES.md preserves the notices shipped with the locked Cargo
dependencies. Its inventory includes build/test and optional platform packages
as well as runtime dependencies. Each dependency retains its own license;
the project's MIT license does not replace those licenses.
Some crate publishers omit notice files. Those cases are explicitly marked in
the inventory with their declared SPDX license, authors and original source
link; the report does not invent missing copyright notices.

## Native pause-menu fonts

Cinzel by Natanael Gama and Inter by Rasmus Andersson, obtained from the
Google Fonts repository (ofl/cinzel and ofl/inter), are bundled unchanged.
Both use the SIL Open Font License 1.1. Complete notices are in
[fonts/Cinzel-OFL.txt](fonts/Cinzel-OFL.txt) and
[fonts/Inter-OFL.txt](fonts/Inter-OFL.txt).
Controller silhouettes are own native vector drawings adapted from the user's
HTML design. ab_glyph and tiny-skia retain their existing dependency notices.
