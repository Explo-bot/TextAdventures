+++
id = "corridor"
nome = "The main corridor"
oggetti = []
oggetti_nascosti = []

[uscite.south]
verso = "mess_hall"
descrizione = "the mess hall door"

[uscite.west]
verso = "crew_quarters"
descrizione = "the passage to the crew quarters"

[uscite.north]
verso = "bridge"
descrizione = "the ladder climbing to the command bridge"

[uscite.east]
verso = "laboratory"
descrizione = "the armoured laboratory door, with a card reader blinking red"
richiede_oggetto = "captain_card"

[uscite.down]
verso = "cargo_hold"
descrizione = "a freight lift going down to the cargo hold and the cold rooms"
+++

## Description
A long corridor of grey metal, lit by red emergency lights that blink for no apparent reason. Puffs of steam come out of the walls at regular intervals, because that is how it's done on spaceships. You can hear little feet scurrying in the ducts above your head. To the south the mess hall, to the west the crew quarters, to the north the ladder to the bridge, to the east the armoured laboratory door, and a freight lift goes down to the cargo hold.

## Details
- The laboratory door reader displays: "RESTRICTED ACCESS — AUTHORISED PERSONNEL ONLY (AND THE CAPTAIN)".
- A trail of feathers leads to the freight lift… and stops abruptly, as if the creatures had changed their minds.
- Fresh graffiti on the wall, made with a claw: a drawing that vaguely looks like a fried chicken. Disturbing.

## Secrets
The trail of feathers that stops in front of the freight lift is the first clue: the creatures avoid the cargo hold (because of the smell of the sour mozzarella).

## Mechanics
- Without `captain_card` the laboratory door does not open (the engine blocks the move): AUNTIE suggests that the captain "always kept everything in his locker, together with his fishing magazines".
- Forcing the laboratory door: impossible ("it's armoured, sweetie, it cost us an arm and a leg").
- After turn 40, if Molly stays in the corridor for two turns in a row without `sour_mozzarella`, a creature drops from the ceiling: apply the ambush rule.
