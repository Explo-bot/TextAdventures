+++
id = "cargo_hold"
nome = "The cargo hold and the cold rooms"
oggetti = ["sour_mozzarella", "coolant_tank"]
oggetti_nascosti = []

[uscite.up]
verso = "corridor"
descrizione = "the freight lift going back up to the corridor"

[uscite.west]
verso = "engine_room"
descrizione = "a watertight door to the engine room, from which comes a hum"

[uscite.east]
verso = "hangar"
descrizione = "the rescue shuttle hangar hatch, marked \"EMERGENCIES ONLY\""
richiede_flag = "self_destruct_active"
+++

## Description
As soon as the lift doors open, you are hit by a smell so powerful your eyes water: **cheese gone off, chemical-weapon grade**. The hold is full of pallets of Mizardian buffalo mozzarella, swollen and yellowish: the cryogenic system of the cold rooms is broken and spits frost at random. Next to the system, a big blue coolant tank is bolted to the wall. To the west the engine room, to the east the hangar hatch, closed.

## Details
- No feathers, no footprints, no "peeok". It is the only place on the ship with no trace of the creatures.
- The cryogenic system has a display: "FAULT — COLD ROOM TEMPERATURE: +18 °C — CONTACT SUPPORT (ESTIMATED WAITING TIME: 4 YEARS)".
- The coolant tank is ice-cold and fixed with huge bolts: a big wrench would be needed.
- The hangar hatch opens only in a declared emergency (self-destruct procedure).

## Secrets
- The sour mozzarella is the creatures' weak point: carrying one keeps them away.
- The coolant tank, connected to a hose, becomes a weapon able to freeze even the biggest monster.

## Mechanics
- The pallets of sour mozzarella never run out: the hold is always full of them, even when `sour_mozzarella` is not listed among the items present now (that only means Molly already took one). There is at most one `sour_mozzarella` in the inventory at a time.
- Taking a mozzarella **before** `weakness_found = true`: Molly has no idea it could be useful and refuses, disgusted. Narrate a soft, plausible excuse tied to the stench (she gags, her eyes water, she would rather lick an engine than touch that thing, "why on earth would anyone carry rotten cheese around?"), with no state change, even if the player insists or gives a reason. Never hint that the mozzarella is useful or that the creatures fear it.
- Taking a mozzarella **after** `weakness_found = true` (Molly learned in the laboratory that the creatures hate sour dairy products): allowed, with `inventario_aggiungi` even if it is not listed among the items present now. Molly holds her nose and grabs one. AUNTIE: "Sweetie, you smell like a pit-aged cheese. I love you anyway."
- Removing the tank: requires `wrench` in the inventory; without it, the bolts do not move. With the wrench: `coolant_tank` goes into the inventory.
- Combining here (or elsewhere) tank + hose + duct tape → `cryo_cannon` (see World rules, +10).
- Opening the hangar hatch without the self-destruct active: impossible (the engine blocks it), AUNTIE explains that it opens "only in a real emergency. This is a normal emergency."
- No ambush is possible in the cargo hold.
