+++
id = "julius_apartment"
nome = "Julius Granger's flat"
oggetti = []
oggetti_nascosti = ["heroin_kit"]

[uscite.west]
verso = "montgomery_block"
descrizione = "the door to the first-floor corridor"
+++

## Description
Two rooms, simple and cared for: nothing luxurious, but not shabby either. A living room with a kitchenette, a table used more as a desk than for eating, covered with books, magazines and sheets of poetry. Urban landscapes and soft-coloured portraits on the walls, a bronze statuette of a stylised figure, a compact gramophone with jazz records and some famous opera arias. An armchair and a small sofa. Through a door, the bedroom with a bed, a nightstand and an opal-glass lamp; beyond it, a bathroom tiled in white and green. The door to the west leads back to the corridor.

## Details
- The wardrobe: clothes put away with care, each like a costume for a different part of his day (a dinner jacket, a worn trench coat, a loud scarf, a crumpled felt hat).
- The nightstand has a drawer.
- In the bathroom: a bottle of laudanum, a dirty piece of gauze, a glass with residue at the bottom.

## Secrets
- In the nightstand drawer there is a metal box: syringes wrapped in paper, blackened spoons, cotton filters, an arm tourniquet, a canvas pouch with whitish powder. Heroin.

## Mechanics
- **Search** (first visit, `julius_on_ferry = true`): opening the nightstand drawer makes `heroin_kit` visible and sets `heroin_found = true` (+10). Vinny can take the box as proof or put it back in its place (both are fine for the report). The bathroom confirms it.
- **Leaving after the search**: Vinny puts everything back in order and locks the door behind him: set `julius_door_open = false` and `julius_on_ferry = false`.
- **The finale, in time** (`finale_started = true`, `shots_fired = false`, Vinny enters by turn `finale_turn` + 3): the door is ajar. The air smells of chloroform. Julius is slumped in the armchair, unconscious, his sleeve rolled up. Bent over him, a syringe in her hand, is **"Jeanne Roche"**; on the sofa, a typewriter and some sheets: anonymous letters. **Miles Harrington**, by the window, turns round with a pistol in his hand. Jimmy is behind Vinny.
  - Any decisive action by Vinny (drawing the revolver, grabbing her arm, shouting, knocking the syringe away, calling her by her real name) stops the injection: set `julius_saved = true` (+20). Miles, covered by Vinny's revolver (or by Jimmy's), drops his pistol. Jimmy runs to call the police and an ambulance.
  - A passive action (waiting, talking about something else, leaving) lets her inject: Julius dies, and the scene continues as in the late finale but here, in the room (Miles tries to pin everything on the delivery man).
  - **Unmasking**: if Vinny calls her Ada Costello or Adèle Morel and explains the swindle, the woman freezes; Miles, cornered, breaks down and confesses to save his own skin ("It was her idea! The mayor, the papers, everything!"). If the player has not yet worked out who she is, Miles blurts it out anyway ("It's over, Adèle!") and Vinny understands. Set `ada_unmasked = true` (+15) and `identity_deduced = true` if it was not already (+15): victory (good ending if `julius_saved = true`).
- **The finale, late** (`shots_fired = true`): Julius in the armchair, his head thrown back, eyes wide open, the syringe on the armrest; on the carpet Miles Harrington and the little man, both dead, pistols a few centimetres from their hands; a bullet hole in the wall; a typewriter on the table with anonymous letters in the same style as Jeanne's. Windows closed, nobody seen leaving the building. The scene tells a convenient story (the little man shot first and missed, Miles fired back, the dying man fired again), but Vinny knows that someone has staged it. No violent details: smell of gunpowder, silence. The truth is behind the door of 1B, in the corridor.
