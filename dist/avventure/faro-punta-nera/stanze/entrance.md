+++
id = "entrance"
nome = "The lighthouse entrance"
oggetti = ["oilskin", "logbook"]
oggetti_nascosti = ["matches"]

[uscite.south]
verso = "cliff"
descrizione = "the half-open door you came in through"

[uscite.up]
verso = "stairs"
descrizione = "the foot of the spiral staircase climbing into the tower"

[uscite.down]
verso = "cellar"
descrizione = "a wooden trapdoor in the floor"
richiede_flag = "trapdoor_open"
nascosta = true
+++

## Description
The ground-floor room of the lighthouse: whitewashed stone walls, a cold cast-iron stove, a table with the lighthouse logbook lying open, an old worn rug on the floor. From a hook beside the door hangs a yellow sailor's oilskin, still dripping. A spiral staircase climbs into the darkness of the tower.

## Details
- The dripping oilskin is Anselmo's: he came back not long ago, so he is somewhere around here.
- The lighthouse logbook (`logbook`), on its last line, in Anselmo's shaky handwriting: "Oil almost gone. Going down to fill the can."
- The rug is crumpled and crooked, as if it had been moved in a hurry and put back badly.
- From under the floor, if the player listens carefully, comes a muffled groan.

## Secrets
- In the inside pocket of the oilskin are the matches, wrapped in oilcloth and therefore dry.
- Under the rug is the trapdoor to the cellar, where Anselmo is.

## Mechanics
- If the player searches, examines or puts on the oilskin, or checks its pockets: make `matches` visible (add it to the room items) and, if they take them, put them in the inventory (+10 points).
- If the player moves/lifts the rug or searches the floor: they discover the trapdoor; if they open it (even in the same action) set `trapdoor_open = true` and mention the exit downwards.
- Shouting or calling Anselmo: a weak answer is heard coming "from below", without explicitly revealing the trapdoor.
- Lighting the stove is useless and uses up a match: discourage it in the narration but respect the choice.
