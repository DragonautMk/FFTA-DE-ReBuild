# Android preview

This Android app replaces the initial mock hardware, simulated compilation,
and fallback checkerboard with the repository's real `gba-core::Machine`, PPU,
timers, DMA, interrupts, backup media, and C recompiler.

- Import your own exact FFTA USA or Europe ROM through Android's document picker.
  Both SHA1 and SHA256 are checked in Kotlin and Rust; oversized arbitrary
  files are rejected. Nothing is uploaded.
- Compile & play profiles a separate boot, emits C, and invokes the separately
  packaged ARM64 `libtcc.so` inside the app. Native blocks remain in memory.
  Missing blocks use the cached interpreter and are compiled in background
  batches, with content guards for mutable RAM code.
- ROM import is a one-time operation. Subsequent launches automatically load
  the private ROM and cached relocatable native objects. Objects are re-linked
  in memory, never executed as external files. Checksums and a build fingerprint
  invalidate corrupt/incompatible caches. Both base and adaptive blocks persist.
- The game executes on its own clocked worker at 59.7275 frames/second. OpenGL ES
  uploads completed frames; the Compose UI is not recomposed for every frame.
- Zoom changes both render dimensions using the desktop's 2^(1/6) steps and
  1536x800 bounds. Existing terrain, movement and sprite fixes remain in gba-core.
- Pause menu: resume, aspect/zoom, fast-forward, audio, touch/gamepad settings
  and exit. Settings persist between launches. Full screen
  adapts to the actual viewport; 1:1 and the Note10's 19:9 are also available.
  Zoom and speed changes are applied live without opening another menu.
- Video & zoom also controls game brightness from 50% to 150%, in 10% steps.
  The default is 100%; the setting persists and affects the game image and
  its independent UI panels, not Android's system brightness.
- The Screen setting offers Raw, Unlit, Frontlit, Backlit and Classic, using
  the desktop color models with an sRGB output target. Raw is the default and
  preserves the existing Android appearance. Unlit and Frontlit also offer
  automatic or manual Darken. These settings apply to the game and its UI
  panels and persist between launches.
- Video & zoom includes a Grid setting with Auto and a 0–100% strength.
  Auto is off for Raw and Classic and on for the physical panel profiles.
  Android uses the desktop grid's band-limited aperture calculation with
  nearest scaling; the effect fades out when the image is too small to
  resolve it. Other desktop scaling modes are not yet available on Android.
- Scrollable menus show a right-hand indicator when more content is available.
- The optional FPS counter shows emulated frames, OpenGL draw callbacks, or
  both. The draw count is not a measurement of display refresh or successful
  buffer swaps; fast-forward can produce multiple emulated frames per draw.
- Menu transparency adjusts the pause panel background from opaque to
  transparent; text and controls stay visible. Android may still dim the
  background while its dialog is open.
- Multitouch supports diagonal movement and simultaneous action buttons.
  Controller defaults: A/B, stick/D-pad, Start/Select, LT/RT for L/R, LB/RB
  for zoom, L3 for temporary speed switching and R3 for toggle fast-forward.
- Audio is real GBA stereo PCM continuously resampled from 65536 to 48000 Hz.
  Fast-forward offers Normal, Fast audio and Silent modes; gamepad bindings
  can be remapped as described below.
- Saves are stored privately as `files/ffta.sav`, written on pause, exit and
  periodically while playing. Preparing code never advances the user's game.
- Pause > Restart ROM asks for confirmation, writes the current cartridge save,
  then restarts from boot. Progress not saved inside the game is lost. If the
  cartridge save cannot be written, the game remains paused and is not reset.

The Android interface and native status messages currently use English; this
port has no interface-language selector. They do not change the language of
the game or imported content.

The Android pause menu is separate from the desktop menu. Desktop temporal
response, other scaling modes, save states, backup management
and debug tools are not yet available here.
Desktop window and VSync settings also do not map directly to the Android
full-screen renderer. The Android aspect choices include the desktop ratios,
as well as device, square and 19:9 modes; mobile zoom is capped at level 6.

## Build on Windows

Requires Rust with `aarch64-linux-android`, Android SDK 35 and NDK
28.2.13676358, Java compatible with Gradle 9.5, Python, and a host gcc/clang
for TinyCC's small build-time header generator. The Gradle wrapper and build
scripts are included. Specify your actual SDK and Java paths:

```powershell
$env:JAVA_HOME = 'C:\path\to\jdk'
$env:ANDROID_HOME = 'D:\Android\SDK'
rustup target add aarch64-linux-android
python scripts/build_android.py --sdk $env:ANDROID_HOME
```

Output: `APK/FFTARecomp-android-arm64-preview.apk` (debug-signed,
ARM64 only). The script pins and checksums TinyCC's source download, compiles
both native libraries, builds the app and runs its local unit tests. No ROM
is needed to build the distributable APK.

Open `android/` in Android Studio to edit the UI. Before assembling there,
run the same Python script with `--native-only` to generate `jniLibs` and
the license/source assets. Keep the same signing key to update an installed
preview without uninstalling and deleting its private save.

## Verification and limits

Host tests compare framebuffer and OAM for 600 actual FFTA frames against the
desktop interpreter with zero extra area. A separate synthetic test also
compares CPU and keys. Kotlin tests check render bounds/zoom expansion and
independent input-source releases. These tests do not measure a phone's FPS,
ARM64 compiler correctness or touch/audio quality on physical hardware.

Debug-only internal metrics distinguish real native dispatches from interpreter fallback and
report actual frames, CPU time, native/Java heap, frame hash and audio underruns.
There is no diagnostics screen/button in the user interface.
No fake compilation test or fallback animation is reported as game execution.

TinyCC is LGPL-2.1-or-later and dynamically loaded. Its upstream source ZIP,
configuration, build script and license are included in the APK under
`assets/licenses/tinycc/`. Existing dependency notices are included alongside it.
The original Android Studio project is left untouched; this directory is the
maintained source integrated into this repository.

Zoom-out is capped at level 6 on mobile. Fast-forward offers Normal (1x audio
with a private temporary backlog, discarded when the mode changes), Fast audio
and Silent, matching the desktop choices. Input separates Touchscreen and
Gamepad. Connecting or using a controller hides the touch overlay; a finger
touch restores it. Packaged third-party notices remain in APK assets, without
an About menu.

Gamepad mapping supports A/B/X/Y, shoulders, triggers, Start/Back/Guide,
stick clicks, both sticks and d-pad directions. Bindings persist across restarts;
select an action and press a controller input in the waiting prompt. Defaults match desktop,
including LT/RT for GBA L/R and LB/RB for zoom. Android may reserve Guide.
Touchscreen settings remain unchanged.

Touchscreen > Edit layout pauses the game and places a tinted layer behind the
draggable controls. Buttons toggles individual visibility (including an optional
hold-fast-forward button). Save persists normalized positions and visibility;
Cancel discards edits; Default restores the initial layout. Optional brief
haptics respect Android's system feedback settings. Android Back remains
available if the on-screen Pause button is hidden.

The on-screen Menu button opens the pause menu and appears above the D-pad by
default. It follows the D-pad until moved separately in Edit layout. Existing
custom button positions are preserved when upgrading; Menu can still be hidden
through Buttons.
