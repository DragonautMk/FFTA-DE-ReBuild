<a id="metrica-del-tutorial"></a>

# Tutorial character metric

MEASURED: complete native replay from frame 0, input ffta-batalla-3.input,
private save target/ffta-replay3/start.sav, without a window or vsync.
Analyzed interval: 11340..12540, 1201 consecutive frames.

MEASURED: unit table 020223A4, stride 0x90. Each record contains
ID at +0x64, body instance at +0x5C and auxiliary instance at +0x60.
Pieces are grouped by body instance and then by logical ID 0..7.
Individual OAM entries are not counted. Cursor 0201C988 does not belong
to those body records; auxiliary instances and UI producers are not
counted either. Published pieces retain their owner and call group
to check for duplicate publications.

ASSUMPTION awaiting user confirmation: the eight reference characters
correspond to IDs 0..7. There is another record, ID 8, body 0201C698,
named Leslaie, category 3 versus 0/2 for the other records. It is reported
separately; it is neither hidden nor treated as a duplicate of another unit.
The meaning of these categories has not been established by disassembling their consumer.

MEASURED: present counts IDs published to the PPU; visible counts IDs with
body pixels that win in the final composition. Both are recorded per frame.
A body outside Y=0..159 may be published without contributing pixels.
With extra=0, observer-tagged OAM is used without enabling the host list.

MEASURED: current, FFTA_PATCH_SPR=1 and FFTA_PATCH_VIS=0, 32:9:
995 frames with present<8; 998 with visible<8. Range of present: 2..8.
MEASURED: proposed, FFTA_PATCH_SPR=1 and FFTA_PATCH_VIS=1, 32:9:
493 frames with present<8; 512 with visible<8. Range of present: 2..8.
The success criterion is NOT met. Zero duplicate groups observed.
Including the additional body, 9 bodies are published in 204 frames
of the current run and 702 of the proposed run; there is no claim that the scene has only 8 bodies.

MEASURED: complete lists, one row per frame with counts and missing IDs:
target/ffta-eight/current32-failures.csv and proposed32-failures.csv.
The visible-failures.csv variants contain the pixel evidence.
The frames.csv files include every frame, not only failures.
Regenerate with python scripts/ffta_character_metric.py target/ffta-eight.
The analyzer uses proposed32-corrected.log; the earlier provisional log
contained a Y calculation error and is excluded from the results.

MEASURED: at frame 12539, predicate 030062EC, LR 0808F837:
x=288 y=-16 -> 0; x=256 y=-16 -> 0; x=176 y=-8 -> 0;
x=240 y=-40 -> 0; x=144 y=-8 -> 0; x=208 y=-40 -> 0.
All six instances remain in RAM. At 12540 only IDs 1 and 2 are published.
The new predicate extends X but retains the original vertical rejection.
ROM Thumb 08A38F90 cmp r0,#0; 08A38F92 blt 08A38FA6;
08A38F94 cmp r1,#0x9f; 08A38FA0 bgt 08A38FA6.
The consumer does not call the producer for rejected bodies.
NOT MEASURED: exhaustive attribution of each of the 493 failures to a single
condition; the publication delay requires correlation by instance.
MEASURED: alternative producer 030068A4 was not called in the final interval
of the earlier run. This is not generalized to other scenes.
MEASURED: the six final instances each have 320 tile bytes examined,
175..191 nonzero bytes and zero addresses outside OBJ VRAM.
NOT MEASURED: visual integrity of all their tiles, or of units never produced.

MEASURED: extra=0, original versus proposed: 1201 identical metrics;
41 identical framebuffer, OAM and OBJ VRAM samples, zero differences.
Check against the original predicate in the user's ROM: 70 identical cases.
No long ROM byte sequences are distributed; checks use hashes and short
headers. Neither the ROM nor the original save is modified.

Next proposal, NOT IMPLEMENTED: also remove the predicate's Y rejection in
widescreen to retain all bodies in the host pipeline, with the same ROM and
RAM checks. It requires approval because it changes the game's decision.
This does not make bodies outside the 160 vertical pixels visible; the
checks for eight published and eight visible bodies are distinct.
