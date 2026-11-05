# Cheat distribution provenance

## Scope and outcome

This review concerns the EUR cheats used in the debug and ROM-data work, not
the legality of any individual code. Their current catalog is user-supplied
local data and remains excluded from Git and public packages. We have not
established permission to redistribute the relevant third-party material.
Neither the existence of a copyright notice nor the absence of an identified
license alone proves an infringement. This review is not legal clearance.

The local schema-1 catalog retains source titles, notes, code lines, identifiers
and provenance as well as the selected plan's data. Changing its format,
rewording labels, decrypting its codes or executing them against the game does
not establish independent authorship or a right to redistribute those inputs.

## Used EUR entries

| Local ID | Scoped observation | Recorded source |
| --- | --- | --- |
| CH-0320 | Ability mastery in one Party scene | GH, master CH-0319 |
| CH-0324 | Item Shop transition in one world-map scene | GH, master CH-0319 |
| CH-0329 | Battle attempt with no observed win | GH, master CH-0319 |
| CH-0334 | Funds change in one shop purchase | GH, master CH-0319 |
| CH-0340 | EXP change in one Party scene | GH, master CH-0319 |
| CH-0341 | First final-battle clear in a cheat-assisted research route | GH, master CH-0319 |

GH refers to the locally supplied copy of the
[GameHacking.org EUR game page](https://gamehacking.org/game/4575). The local
source index attributes the codes to MadCatz; neither that attribution nor the
site's statement that its content includes device and community contributions
establishes the particular rights needed to republish them. The site's
[home page](https://gamehacking.org/?page=home) describes providing codes for
other projects, but no grant covering this catalog has been verified. Do not
contact the site or contributors without explicit user instruction.

In particular, CH-0341 was first selected from that source and independently
*decoded* using a separate implementation, then investigated and exercised in
isolated game sessions. Independent decoding is not independent discovery of
the original cheat. The research checkpoint retains the external source and
cheat application provenance. It does not establish behavior in all three
final-battle phases, original device timing or ordinary no-cheat progression.

## Candidate alternative reviewed

The [OpenEmu cheat database](https://github.com/skyzyx/openemu-cheat-database)
declares a permissive public-domain dedication for the repository. Its
[FFTA section](https://raw.githubusercontent.com/skyzyx/openemu-cheat-database/master/cheats-database.xml)
also points to a GameFAQs guide, covers USA and uses GameShark entries. The
repository notice alone does not verify the upstream rights in those entries;
the current bounded runtime does not support GameShark activation, and USA
data does not establish compatibility with the tested EUR ROM. It is not an
approved substitute for the six used entries.

## Work needed for a distributable replacement

Two viable routes remain open:

1. Establish a suitable grant or other basis for the particular third-party
   inputs, including whether redistribution in source and packaged builds is
   covered. Preserve attribution and all applicable conditions; document the
   evidence and scope for each included entry.
2. Design and validate project-authored game-state operations from documented
   observations of a lawfully used local game copy, without copying source
   descriptions, encoded codes, master blocks or selection/arrangement from a
   third-party collection. Record actual influences and evidence; do not call
   prior source-informed work clean-room or independently discovered. Review
   game-data rights and any applicable database rights before distribution.
   Reimplementing an effect still needs its own exact-ROM, runtime, safety and
   gameplay tests; a familiar outcome does not imply identical behavior.

The [EU computer-program directive](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX%3A32009L0024)
recognizes observation, study and testing under specified conditions. That
does not itself grant blanket rights to redistribute a particular derived
catalog. The [EU database directive](https://eur-lex.europa.eu/legal-content/EN/TXT/?uri=CELEX%3A01996L0009-20190606)
has additional conditional protections. Applicable rights and exceptions
depend on the particular material, use and jurisdiction.

Until a specific entry passes that review, do not bundle it, claim it is
copyright-free, or relabel the current imported code as project-authored.
Local catalogs and existing debug safeguards continue to operate unchanged.
