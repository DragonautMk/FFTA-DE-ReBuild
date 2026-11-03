# U31 upstream audit: approved decisions

On 2026-11-02 the user approved the bounded proposals previously presented and
then approved the additional recommendations below. These decisions supersede
older statements in individual audit reports that those units still await user
approval. Approval is not evidence of implementation, successful validation or
an unrestricted upstream merge. Reuse upstream code when compatible with the
fork, adapting it where needed and preserving existing user data and behavior.

## Previously approved units

- Desktop recognized battle-menu miss guard, Android Party selection, and shop
  category replay repair: implement within their documented bounded scopes and
  acceptance gates. The desktop EUR shop scrollbar is approved conditionally
  on a demonstrated need and the documented readiness and replay prerequisites.
- Desktop machine autosave: the accepted opt-in, timing, retention and isolated
  recovery settings are recorded in [U31-07](U31-07-autosave-proposal.md).
  Android autosave needs a separate design.

## Additional decisions

- **U31-01:** offer desktop fast-forward speeds from 100% through 800% in
  10-point increments, with existing shortcuts changing the speed by 20 points
  and existing settings preserved. From 150%, the shortcuts select 130% and
  170%. Importing upstream's floating-point configuration format remains a
  separate decision. See [the bounded proposal](U31-01-150-percent-proposal.md).
- **U31-04:** first investigate an optional, disableable transition for one
  observed panel; treat smooth zoom as a separate unit.
- **U31-03 geometry:** first establish a visible reproducible defect, then fix
  the affected scene if confirmed; this is separate from the approved miss guard.
- **U31-05:** first identify an original-game screen with problematic margins;
  adapt that screen rather than automatically applying the change to all menus.
- **U31-06 dialogue:** investigate a specific two-answer Android dialogue and,
  if needed, add direct selection while keeping experimental touch optional.
  This is separate from the approved Party unit.
- **U31-09:** keep the current default menu shortcut and consider Y as a
  configurable option, with conflict analysis and a device check.
- **U31-10:** defer Android color profiles until there is a concrete measured
  need; consider them as options if pursued later.

Do not import upstream's entire save-card or packaging subsystems without a
specific need. Preserve the fork's existing regional support and checks rather
than adopt a blanket USA exclusion. Remaining scene, platform, input, replay,
save and native execution checks are specified in the corresponding audit
reports. Individual investigations do not imply broader platform coverage.
