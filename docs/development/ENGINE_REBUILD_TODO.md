# TODO — Rebuild the FFTA game engine

The goal is to play FFTA from an editable asset pack, preserve the original rules and timing,
and let a player change a line of dialogue, a tune or a battle without rebuilding the
executable. The proposed Rust core would run the rules with or without a window. Godot 4 is a
candidate for drawing the game, playing its audio and handling input.

**Future work.** EUR extraction and consumption retain their current priority. Development
starts when the agreed EUR scope is complete and the user selects a rebuild milestone. The
earlier Godot assessment covers connecting a frontend to the existing Rust runtime. This
checklist covers replacing game systems as well. [ROM work and priorities](ROM_DATA_TODO.md) ·
[Godot frontend assessment and trial](GODOT_FRONTEND_TODO.md)

**First deliverable.** Capture the frame where one line of EUR school dialogue appears. Change
its UTF-8 text in a separate pack and replay the same input. The changed line must appear at
the same cue, and disabling the edit must restore the original glyphs and timing. Then
exercise the same pipeline with a looping song and a playable battle.
[School dialogue reference](../rom_data_text.md#script-text-production-cli-and-ordinary-school-dialog)

## RB-01. Pin the reference and define a match

- [ ] Select the school dialogue for the first asset and the White Flowers encounter at Giza
  Plains for the first playable battle. Record the reference revision, EUR ROM identity,
  dataset manifest, input sequence, initial state and clock. Identify which observations
  already exist and which actions still need a reference.
  [Dialogue evidence](../rom_data_text.md#script-text-production-cli-and-ordinary-school-dialog)
  ·
  [Deployment and movement evidence](../rom_data_maps.md#normal-giza-battle-entry-and-positive-constructor-animation)
- [ ] Define a trace both engines can produce. At each relevant game tick compare inputs,
  random number generator (RNG) draws, actor positions and facing, HP, turn order, event IDs
  and save-relevant flags. At selected ticks compare the classic 240×160 image and the audio
  event timeline. Compare PCM samples at the same point in the mixer and at the same rate. Set
  any tolerance for resampling or final device output in advance.
- [ ] Accept a base replay when discrete state and event timing agree and the picture and
  sound meet the declared criteria. Report the first differing tick, field or sample and its
  expected and actual values. Keep the current CPU, clock, image and audio comparisons as
  reference evidence. Resolve a disputed reference against an independent original-game
  observation. [Replay profiles](../debug_replays.md) ·
  [Existing comparison signals](../debug_comparisons.md#what-is-compared)

## RB-02. Build the path from verified data to editable assets

- [ ] Keep the existing extractor and its authenticated `manifest.toml`, source ranges and
  hashes. Map each selected resource to a stable ID, regional variant, reader and authoring
  file. Preserve aliases and unknown fields losslessly. The verified dataset remains the
  read-only base for every edit.
  [Existing files and verification](../rom_data.md#extracted-files)
- [ ] Specify export, validate, build and inspect operations. Export documented UTF-8 JSON and
  media files to a working directory. Validate types, units, bounds and references with a
  filename, resource ID and field in each error. Record schema versions and migrations.
  Compile validated files into an indexed pack whose identity includes the base, tool version,
  import settings and ordered mod hashes. Load or stream compiled resources by ID so a game
  turn need never parse authoring JSON.
- [ ] Prove an unedited export/import preserves the decoded values and reconstructs exact
  source bytes wherever the format promises a lossless round trip. Two clean builds of the
  same inputs must produce the same pack hash. Run the school-line edit through the actual
  reader, identify the loaded file, then restore the base replay. An invalid edit must leave
  the last working pack usable.

## RB-03. Define a format and an edit test for each resource family

Start with the family required by the current scene. For each format, ship a schema, a small
original example and an edit that can be observed during play. Keep ROM offsets in the private
provenance map so authors can use names across revisions.

- [ ] **Dialogue and fonts.** Store UTF-8 text by stable key and locale in JSON. Represent
  substitutions, pauses, page breaks and other control codes as explicit tokens. Preserve
  glyph mapping and advance widths. Verify a line with a substitution and a page break,
  including an imported language with accented glyphs. English is the default for tools and
  authored examples. [Text and font contracts](../rom_data_text.md)
- [ ] **Items, jobs and abilities.** Use typed JSON records with named fields, units, stable
  references and explicit unknown values. Start with the documented discount on an eligible
  item. Change the discount, then check the price shown in the shop and the amount deducted.
  Add job requirements and ability effects as their readers and rules are understood.
  [Progression and price records](../rom_data_gameplay.md#next-ten-byte-progression-and-price-records)
- [ ] **Sprites, tiles, palettes and animation.** Export indexed PNG sheets with palette and
  animation JSON. Retain original palette words, index order, transparency, tile flips,
  origins, layer priority and frame durations in game ticks. Verify a palette edit affects the
  intended pixels and an animation loops on the expected tick.
  [Graphics contracts](../rom_data_graphics.md) · [OBJ animation banks](../rom_data_obj.md)
- [ ] **Maps and encounters.** Use a JSON grid for terrain, height, layers and links to tiles,
  deployment slots, actors and triggers. Keep tile art in editable sheets. Change one tile's
  height and verify the reachable cells, cursor highlight and actor destination for a unit
  with known movement and jump limits. [Map and terrain contracts](../rom_data_maps.md)
- [ ] **Events and scripts.** Represent proven commands as named operations with typed
  arguments, labels, branches and waits. Preserve opaque instructions with their original
  identity and flag any execution path that still needs the reference runtime. Exercise both
  outcomes of a branch and compare the tick of the next dialogue or sound cue.
  [Script contracts](../rom_data_scripts.md) · [Event consumers](../rom_data_events.md)

## RB-04. Make music and effects editable and listenable

- [ ] Reuse the EUR M4A/MP2K directory, sequences, tones and driver tables. Export each song
  as named JSON events per track with references to its instruments and samples. Preserve
  tempo changes, waits, patterns, loops, note ties, envelopes, keymaps, pan and player
  priority. Store PCM samples as WAV with rate and loop metadata. Keep chip-generated voices
  and driver-specific commands in the editable model.
  [Audio source contracts](../rom_data_audio.md)
- [ ] Render each supported song or effect by ID to a WAV that opens in an ordinary audio
  player. Produce a local index with filenames, shared or empty entries and unsupported
  commands. For looping music, render the intro and a declared number of loops, and record
  loop boundaries in sample frames. Export one-shots through their release tails. Offer Ogg
  Vorbis for compact listening copies or replacements. Reuse the existing short diagnostic
  WAVs to check the mix at a cue.
  [Existing audio previews](../debug_audio.md#formats-limits-and-identity) ·
  [Godot audio formats and import settings](https://docs.godotengine.org/en/stable/tutorials/assets_pipeline/importing_audio_samples.html)
- [ ] Verify one looping song and one overlapping sound effect from the reference. Check cue
  ticks, loop seam, sample count, priority and release. Play the exported files outside the
  engine, change one instrument or supply a replacement recording, then hear the change in the
  same scene. Disabling it must restore the reference mix. Record which sequence features a
  replacement recording can reproduce.

## RB-05. Move one playable encounter into a headless game core

- [ ] Give the proposed Rust core ownership of the game clock, RNG, state and rules. Accept
  input commands on game ticks and emit draw, text and audio events. Godot would consume those
  events and return input through GDExtension. Keep file decoding in the shared asset
  pipeline. Pin the Godot and binding versions when the trial begins.
  [Proposed runtime boundary](GODOT_FRONTEND_TODO.md#gd-02--definire-linterfaccia-runtime) ·
  [Godot native extension interface](https://docs.godotengine.org/en/stable/engine_details/engine_api/gdextension/what_is_gdextension.html)
- [ ] Rebuild deployment, movement and facing, one attack, an enemy turn and the result of the
  White Flowers encounter. Match integer rounding in hit and damage calculations, RNG call
  order and tie-breaks in enemy decisions. Reuse the recorded Giza entry and movement. Obtain
  reference traces for the attack, enemy turn and result before accepting those rules.
  [Giza battle reference](../rom_data_maps.md#normal-giza-battle-entry-and-positive-constructor-animation)
- [ ] Replay the encounter headlessly and through the candidate frontend with the same seed
  and tick-stamped input at different display refresh rates. Require matching RB-01 traces.
  Count raw ROM reads and guest-code execution by subsystem. Run the encounter again with
  guest fallback disabled for the migrated systems and the base assets loaded from the pack.

## RB-06. Save progress and resume the same turn

- [ ] Define a versioned game save with stable content IDs, campaign and battle state, RNG,
  pending events and the active pack identity. Decide which cartridge `.sav` progress can be
  imported at a known boundary. Existing `.ffstate` machine snapshots can start the reference
  replay. Document supported imports, backups, migrations and incompatible versions.
  [Current cartridge saves and snapshot slots](../save_states.md#cartridge-saves-and-restored-state)
- [ ] On a copy, save during the encounter and reload in a fresh process. Replay through the
  next turn and compare to uninterrupted play. Verify a failed import leaves prior saves
  intact, a modded save identifies its required content, and a supported save transfers
  between the first two desktop targets.

## RB-07. Give mod authors a complete edit–play–restore workflow

- [ ] Follow ROMDATA-06 for replacements and ROMDATA-07 for additions. Define
  `mods/<id>/mod.toml` with package/schema versions, compatible base profiles, dependencies
  and resource IDs. Resolve load order deterministically, show which file supplies each
  resource and explain dependency, version and override conflicts before changing the active
  pack. Include the resolved identity in caches, saves and replays.
  [Resource replacements](ROM_DATA_TODO.md#romdata-06--mod-tramite-sostituzione-delle-risorse)
  · [Extended content](ROM_DATA_TODO.md#romdata-07--modifiche-estese-a-contenuti-e-logica)
- [ ] Ship an original example that an author can edit, validate, enable on restart and
  disable using a text editor and ordinary media tools. Demonstrate a dialogue edit and a
  music swap, then a new namespaced ability or encounter with a declarative rule. Try two
  conflicting mods and a missing dependency. Disabling all mods must recover the base replay
  and leave the original saves usable.
  [Godot runtime loading for user-supplied media](https://docs.godotengine.org/en/stable/tutorials/io/runtime_file_loading_and_saving.html)

## RB-08. Measure portability and performance before choosing Godot

- [ ] Start with an exported native Windows build, then run the same pack and replay on native
  Linux. Check case-sensitive paths, portable saves, keyboard/controller mapping, audio-device
  reopen and suspend/resume. Track Android separately in `AndroidPort/` with touch, storage
  and physical-device lifecycle checks if selected. Label Wine and native results separately.
  [Portable storage](../portable.md)
- [ ] Before timing a candidate, specify hardware, resolution, audio settings, replay,
  repetitions and numeric budgets for each target. Use the current frontend as the baseline.
  At 1× its pacing interval is about 16.74 ms. Record p95/p99 frame times, missed deadlines,
  input latency, audio underruns, cold/warm load time and peak memory. Measure the cost of
  copying frames or audio across the Godot boundary. Profile the stage that misses its budget.
  [Current pacing](../../crates/recomp/src/play.rs) ·
  [Existing performance methodology](PERFORMANCE_TODO.md)
- [ ] Record the adoption decision with measured values. Adopt Godot for the tested scope if
  the exported battle meets parity and platform budgets and an author can load an edit without
  the Godot editor. If a gate fails, keep the working frontend and identify the failing
  feature or cost. The editable formats and headless rules can progress independently once
  selected.

## RB-09. Expand from one battle to the campaign

- [ ] Migrate successive playable scenarios. Cover the world map and missions, clan roster and
  equipment, jobs and ability learning, combat AI, laws and judges, shops and rewards, story
  events and endings. For each scenario record the resource readers, game rules,
  regional/language coverage, remaining guest calls and a normal-input replay. Retire a legacy
  path when all its supported callers pass with the replacement.
- [ ] From a clean installation, prepare a private pack locally, then boot with only that pack
  and the rebuilt engine available. Complete a fresh campaign, save, resume and reach the
  ending. Require zero legacy code execution or original-ROM reads for the scope claimed.
  Record the profiles, platforms and mod combinations actually verified. Distribute tools,
  schemas and original examples. Keep ROM/BIOS, extracted game art/music, generated ROM code,
  personal saves and recordings out of source control and packages.
