+++
id = "stairs"
nome = "The spiral staircase"
oggetti = []
oggetti_nascosti = []

[uscite.down]
verso = "entrance"
descrizione = "the steps going down to the entrance"

[uscite.up]
verso = "lantern_room"
descrizione = "a locked iron door at the top of the stairs"
richiede_oggetto = "brass_key"
+++

## Description
A hundred and twenty stone steps wind around the central pillar of the tower. The arrow slits let in gusts of rain. Halfway up, a small window looks out to sea. At the top, an iron door.

## Details
- From the window you can see the brigantine (distance according to the World rules).
- One of the steps halfway up is cracked and wobbly.
- The iron door at the top has a big lock; without the key it will not open.

## Secrets
The cracked step gives way if someone runs or jumps over it.

## Mechanics
- If the player says they run/rush up/jump the steps: the step gives way, they fall and are slightly hurt; they stay in `stairs` and lose the turn. If at that moment they have `full_can`, the oil spills: replace `full_can` with `empty_can` (they will have to go back to the cellar).
- Climbing normally: no problem.
- Without `brass_key` the door does not open (the engine prevents the move): narrate the vain attempt and hint that Anselmo keeps the key "in a safe place".
- Forcing the door is impossible.
