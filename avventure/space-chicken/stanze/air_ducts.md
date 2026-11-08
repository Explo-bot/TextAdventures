+++
id = "air_ducts"
nome = "The air ducts"
oggetti = []
oggetti_nascosti = []

[uscite.down]
verso = "crew_quarters"
descrizione = "the open grate above the crew quarters"

[uscite.east]
verso = "laboratory"
descrizione = "a stretch of duct ending at a grate above the laboratory"
+++

## Description
You crawl on all fours through a narrow, hot metal tube, lit only by the service lights. There are feathers everywhere, little bones of who knows what and a stench of a cramped henhouse. Somewhere ahead of you, something goes "peeok… peeok… PEEOK".

## Details
- The duct forks: to the east it goes down towards the laboratory grate; behind you is the crew quarters grate.
- Fresh scratches on the walls, at the height of your face.
- A nest made of socks stolen from the crew quarters. Inside, luckily, no eggs.

## Secrets
The air ducts are the creatures' den: going in means an almost guaranteed ambush.

## Mechanics
- **Ambush on entry**: the first time Molly enters the air ducts (and then once every two visits) a creature attacks: apply the ambush rule from the World rules. With `sour_mozzarella` in the inventory the creature runs away and leaves the way clear.
- If the ambush makes Molly flee, she goes back to the room she came from (set `stanza_corrente` accordingly).
- Stealing the socks from the nest: useless, but AUNTIE comments in disgust.
