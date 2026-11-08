+++
id = "cellar"
nome = "The cellar"
oggetti = []
oggetti_nascosti = ["crank"]

[uscite.up]
verso = "entrance"
descrizione = "the ladder leading back up to the trapdoor"
+++

## Description
You climb down a slippery ladder into total darkness. The air smells of rancid oil, wet stone and brine. You cannot see anything. Somewhere, not far away, someone is breathing with difficulty.

## Details
(Visible only with `match_lit = true`.)
- Anselmo is lying at the foot of the ladder, pale, one ankle swollen and twisted.
- At the back, a large oil barrel with a brass tap.
- One of the rungs of the ladder is broken: that is how Anselmo fell.

## Secrets
- Anselmo has in his pocket the `crank` used to wind the lens's clockwork mechanism: without it the lens does not turn and the light stays fixed and useless as a signal.
- The barrel is the lighthouse's only supply of oil.

## Mechanics
- In the dark the player cannot fill the can or see Anselmo, but can talk to him. Anselmo, in the dark, says: "Nicola! Strike a match, for God's sake… the lamp… the oil is here."
- Striking a match: decrease `matches_left`, set `match_lit = true`. With the light the player sees everything (Details). If `matches_left` is 0, they cannot strike one.
- Helping Anselmo (talking to him and helping him get comfortable, splinting his ankle, giving him the oilskin…): set `anselmo_helped = true` (+15 points); Anselmo hands over the `crank` (add it to the inventory) and says: "Without this the lens won't turn. Wind the mechanism all the way, then light the lamp."
- Just by talking to Anselmo the player gets the crank, but `anselmo_helped` becomes true only if they do something concrete to help him.
- Filling the can at the barrel: requires `empty_can` in the inventory and light (`match_lit = true` in this turn, or in the same turn in which they strike one). Remove `empty_can` and add `full_can` to the inventory.
- Anselmo cannot move and cannot climb: if the player tries to carry him up, they cannot do it alone; Anselmo insists that the lighthouse comes first.
- Bringing the lit match close to the open barrel → fire → `esito = "sconfitta"` (warn first with a moment of hesitation if the intention is ambiguous).
