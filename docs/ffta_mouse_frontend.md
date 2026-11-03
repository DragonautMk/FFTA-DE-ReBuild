# Experimental EUR front-end mouse adapter

Enable FFTA_MOUSE=1. Move to select, left click to confirm, right click for
original B/back. Manual keyboard/gamepad input cancels queued mouse input.
Scene changes cancel pending clicks; empty areas are not targets. The same
logical 8x8 interaction grid and window/zoom coordinate conversion are reused.

Exact EUR SHA256 and short instruction checks are required. No ROM patching.
USA mouse front-end remains unchanged except for the name keyboard adapter.

2026-10-17: physical typing on the name screen queues the original glyph selector
and A, never writes the name buffer. ASCII upper/lowercase and digits are mapped;
Backspace sends B and Enter sends Start. Typed keys are consumed before gameplay
bindings, so A/B letters do not also trigger their mapped GBA buttons. Numeric
page selection uses the original L/R keys (EUR3 pages, USA2 pages). Unsupported
printable characters are consumed without triggering gameplay shortcuts.
USA checks: page commit0812AC2E, row-width literal0812AAB4 ->083A8320.
MEASURED: simulated uppercase/lowercase/digit selection and one original confirm per
character, Backspace/Enter sequencing in both profiles. NOT MEASURED: real typing on
device, including the USA name-screen RAM signature; user acceptance pending.

## Measured selectors

| Scene | State / selected RAM | Replay target IDs |
|---|---|---|
| Startup language | 0200F3D6; existing full OAM validation | 0..4 |
| Press Start | title task 08139D29, state 0200F3B8=5; original Start key | 64 |
| Main menu | title state=7, 02017D68=0; 02017D36 | 32..35 |
| Color Mode | 02017D68=1; 02017D37 | 40..42 |
| Inline Language | 02017D68=2; 02017D38 | 48..52 |
| Resume Battle / Load | IWRAM030027D1=4; descriptor0201152C; selected02011631 | 56..57 |
| Name keyboard | screen=2, cursor tile0120; col020166F4,row020166F5 | 80..139, valid cells only |
| Name categories | page020166DA; original L/R, one key to any of three pages | 144..146 |
| Name OK / Delete | original Start / B | 147..148 |
| Confirm name Yes / No | screen=2; pointer0200918C=02017370; selected020173B1 | 152..153 |

Main selector write Thumb0813A736, short check0813A734=01300870.
Saved selector write Thumb0808BE74, caller0813E494; check0808BE72=28403070.
Name page write Thumb0812E81C; check0812E81A=01302070. Keyboard navigation
0812E5F8..0812E614 reads six row widths per page at ROM083AD2FE via literal
0812E6A0 (FED23A08). Row widths come from the user ROM, not copied game data.
Name Yes/No write Thumb08011BFA; check08011BF8=51400170. Its cursor cycles
OAM tiles0125..0127: all three animation frames must be accepted.

## Validation and limits

MEASURED: complete boot, actual click on Press Start, Color Mode, TV, Saved Game,
Resume/Load selection. Another complete boot clicks New Game, then a private
synthetic A sequence advances story to the name keyboard. Mouse clicks exercise
letters, Delete, all three categories, OK and Yes/No. The story A sequence is a
test driver; arbitrary story dialogue clicking is not implemented here.

Changing language or backing out of Saved Game can restart the original title
sequence; this behavior is preserved. A separate run selected Italian and
confirmed the original language reload. Back from the language submenu was
also tested without reload. All investigations use blank/private save state;
no original save is written.

NOT MEASURED: loading/resuming a valid saved slot. Available save files contain
only FF bytes; the original game rejects empty Resume/Load. Actual populated
file-slot submenus are not adapted or claimed tested. Battle mouse controls,
USA front-end and physical mouse acceptance of this build remain pending.

Native replay of the recorded front-end clicks exactly matches the interpreter
framebuffer at1490 frames. USA and EUR extra=0 framebuffer AND OAM match their
previous original reference for600 frames each. Windows build is
 target/release/recomp-mouse-main.exe.

Private evidence: target/menu-click-all.*, target/menu-click-recorded.*, 
target/menu-click-native.*, target/menu-new-click-all.*, 
target/menu-new-symbol-click.*, target/menu-name-mouse-final.log.
