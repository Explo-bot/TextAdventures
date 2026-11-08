+++
titolo = "The Punta Nera Lighthouse"
descrizione_breve = "A stormy night, a dark lighthouse, a ship heading straight for the rocks."
autore = "Example for the Avventura engine"
versione = "1.0"
lingua = "en"
stanza_iniziale = "cliff"
max_turni = 40
inventario_iniziale = []
spostamenti_liberi = false

[flag_iniziali]
trapdoor_open = false
matches_left = 3
match_lit = false
anselmo_helped = false
lamp_filled = false
lamp_lit = false
lens_turning = false
boat_turned = false

[oggetti]
brass_key = "a tarnished brass key"
empty_can = "an empty tin can"
full_can = "a tin can full of oil"
oilskin = "the keeper's yellow oilskin"
matches = "a box of matches wrapped in oilcloth"
crank = "an iron crank"
logbook = "the lighthouse logbook"
+++

## Introduction
October 1893. You are Nicola, seventeen years old, assistant to old Anselmo, the keeper of the Punta Nera lighthouse.
You went down to the village in the afternoon for supplies; now it is night, the wind is howling and the rain comes in sideways.
Climbing back up the path you notice something that chills your blood: **the lighthouse is dark**.
Out there, beyond the waves, the navigation lights of a brigantine are shining, and it is heading straight for the rocks.

## Setting
The Ligurian coast at the end of the nineteenth century, a night of violent south-westerly gale. The lighthouse is a three-storey stone tower on a headland.
Anselmo, the keeper, went down to the cellar to fetch oil, slipped off the ladder and broke his ankle: he is alive, in the dark, and nobody can hear him. The lamp went out because it ran out of oil.
The brigantine "Santa Lucia" is coming back to port, trusting the lighthouse.

## Goals
Relight the lighthouse before the brigantine is wrecked: refill the lamp with oil, light it and set the lens turning again with the clockwork mechanism. Secondary goal (score): help Anselmo.

## Victory conditions
Victory when, at the same time, `lamp_lit = true` and `lens_turning = true`, by turn 40.
Narrate the beam of light sweeping the sea and the brigantine turning away at the last moment. If `anselmo_helped = true`, add that at dawn Anselmo, splinted and limping, shakes your hand.

## Defeat conditions
- The engine declares defeat at turn 40 (the brigantine runs aground): the AI does not need to do it.
- The player goes out onto the outer gallery of the lantern room without securing themselves to the railing/rope: the wind throws them off → `esito = "sconfitta"`.
- The player sets fire to the full can or pours oil on an open flame outside the lamp → fire → `esito = "sconfitta"`.

## Tone and style
A nineteenth-century seafaring tale, tense but not gruesome. Concrete, sensory sentences: wind, salt, the smell of whale oil and wet iron. Never ironic. The passing of time is felt through the lights of the brigantine.

## World rules
- Time flows: every action is a turn.
  Turns 1–14: the brigantine's lights are far away. Turns 15–29: they are clearly closer, the sails can be made out. Turns 30–39: very close, the ship's bell can be heard. Mention it when the player looks at the sea or out of a window, and now and then spontaneously.
- Matches: there are 3 at the start (`matches_left`). Every time the player strikes one, decrease the flag. A lit match lasts a single turn (`match_lit` goes back to `false` on the next turn, unless it is used to light something). Without matches the lamp cannot be lit: do not make other sources of fire appear.
- The cellar is dark: without a lit match the player sees nothing and cannot find items, but can hear Anselmo groaning and talk to him.
- Score: +10 finding the key, +10 finding the matches, +15 helping Anselmo, +10 refilling the lamp, +20 lighting it, +35 making the lens turn.
- Items are few and precise: do not invent other usable ones (ropes, lighters, extra lanterns…). You may describe furniture, but it does not become something that can be taken.
