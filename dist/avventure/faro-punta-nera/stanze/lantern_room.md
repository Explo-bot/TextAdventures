+++
id = "lantern_room"
nome = "The lantern room"
oggetti = []
oggetti_nascosti = []

[uscite.down]
verso = "stairs"
descrizione = "the iron door and the spiral staircase"
+++

## Description
The room at the top of the lighthouse, enclosed by rain-beaten windows. In the middle, the great Fresnel lens with its crystal steps, standing still. Inside it, the oil lamp is out and cold. On one side, a clockwork mechanism with weights and gears; a small door leads onto the outer gallery, where the wind howls.

## Details
- The lamp's reservoir is empty: there is a brass funnel above the filler neck.
- The wick is dry and in good condition.
- The clockwork mechanism has a square hole for a crank; the weights are on the floor: it has run down.
- From the windows the brigantine can be seen very clearly (distance according to the World rules).
- On the gallery, a rope is tied to the railing.

## Secrets
- The lamp must first be refilled, then lit. The lens must be made to turn by winding the mechanism with the crank: a fixed light is not recognised as the light of Punta Nera.
- The order between lighting the lamp and winding the mechanism does not matter, but both are needed.

## Mechanics
- Pouring the oil into the lamp: requires `full_can`; replace it with `empty_can` and set `lamp_filled = true` (+10).
- Lighting the lamp: requires `lamp_filled = true` and at least one match (`matches_left > 0`); decrease `matches_left`, set `lamp_lit = true` (+20). If the lamp is empty, the match burns out and the wick only smokes.
- Winding the mechanism: requires `crank` in the inventory; set `lens_turning = true` (+35). Without the crank: their hands slip on the gears, nothing happens.
- When `lamp_lit` and `lens_turning` are both true → `esito = "vittoria"` with the final narration described in gioco.md.
- Going out onto the gallery: if the player does not say they tie themselves to the rope or hold on to the railing → `esito = "sconfitta"` (the wind tears them away). If they secure themselves, they see the brigantine very close but gain nothing useful.
