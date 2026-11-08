+++
id = "laboratory"
nome = "The science laboratory"
oggetti = []
oggetti_nascosti = []

[uscite.west]
verso = "corridor"
descrizione = "the armoured door to the corridor (it opens from the inside with a button)"

[uscite.up]
verso = "air_ducts"
descrizione = "the ventilation grate in the ceiling"
+++

## Description
The laboratory of Dr Ripley-Smith, the mission's science officer: white benches, test tubes, a microscope and, in the middle, the **biological analyser**, a machine as big as a fridge with a slot for samples and a screen that says "INSERT SAMPLE". On the whiteboard, a diagram of an egg with the words "THEY ARE NOT FOR OMELETTES!!!" underlined three times. An armoured door to the west, a grate in the ceiling.

## Details
- On the bench, the doctor's diary: last entry, "The eggs contain galliform DNA with artificial modifications. Somebody designed them. Who?"
- An empty cage with its door ripped open from the inside.
- The analyser has a sticker: "Property of MegaCorp Unlimited — Advanced Poultry Division". Curious.

## Secrets
- Analysing the `eggshell` (or another trace of the creatures' slime, but the only sample available is the shell) the analyser reveals: the creatures have a hypersensitive sense of smell and an extreme aversion to the compounds of fermented dairy products; the smell of sour cheese sends them into a panic.
- The sticker and the diary are clues to the conspiracy, which is fully revealed only on the bridge.

## Mechanics
- Putting the `eggshell` into the analyser: set `weakness_found = true` (+10), the shell is used up. The machine prints the result in a squeaky voice: "AVERSION DETECTED: SOUR DAIRY PRODUCTS. ADVICE: AVOID PIZZERIAS." AUNTIE: "Sour dairy? Sweetie… the cargo hold is full of mozzarella that's gone off!"
- Without a sample the analyser answers "NO SAMPLE. SORRY. BEEP."
- Going out through the door to the corridor does not require the key card (inside button).
