+++
id = "shuttle"
nome = "The rescue shuttle"
oggetti = []
oggetti_nascosti = []

[uscite.out]
verso = "hangar"
descrizione = "the hatch to the hangar"
+++

## Description
The cockpit of the Little Badger: two seats with seatbelts, a panel with the big green "LAUNCH" button and, at the back, a red emergency hatch with a "QUICK RELEASE" lever. And then there is **him**: a creature more than three metres tall, hunched under the ceiling, with a tuft of feathers on its head and a beak full of teeth. It looks at you. It licks its beak. "…PEEOK."

## Details
- The monster is standing right in front of the emergency hatch.
- The seatbelts are sturdy, the four-point kind.
- The launch button has a safety cover and a warning light: "HATCHES CLOSED: NO".

## Secrets
- To win you have to: hold the monster off (sour mozzarella), freeze it (cryo cannon), fasten your seatbelt, open the emergency hatch (the frozen monster is sucked out into space), close it again and take off with Mr. Whiskers.

## Mechanics
- **Holding the monster off**: showing/throwing/waving the `sour_mozzarella` → the monster backs away in disgust against the hatch: `monster_held_off = true` (the mozzarella stays in the inventory if it is only shown, it is removed if thrown: in both cases the effect lasts until the end of the fight).
- Without `monster_held_off`, any action other than fleeing outside makes the monster attack: apply the ambush rule (at the second ambush suffered: defeat).
- **Freezing**: with `cryo_cannon` and `monster_held_off = true` → `monster_frozen = true` (+15): the monster becomes a chicken-shaped ice lolly. With only the `coolant_tank` (no hose) the jet goes everywhere and is not enough: no effect, turn lost.
- **Seatbelt**: fastening it sets `seatbelt_fastened = true`.
- **Emergency hatch**: opening it with `monster_frozen = true` and `seatbelt_fastened = true` → the monster is sucked out into space: `monster_ejected = true` (+15). Without the seatbelt → defeat (Molly flies away together with the ice lolly). With the monster not frozen → the monster clings on with its claws and stays inside: ambush.
- **Launch**: pressing "LAUNCH" requires `monster_ejected = true` (the hatch must be closed first: the AI closes it automatically in the narration). If Mr. Whiskers is not in the inventory, Molly stops: "I'm not leaving without Mr. Whiskers." and AUNTIE reminds her where the cat was (the player can go back for him if time allows). With Mr. Whiskers: `shuttle_launched = true` (+15) → victory (see gioco.md).
