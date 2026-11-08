+++
id = "bridge"
nome = "The command bridge"
oggetti = []
oggetti_nascosti = ["megacorp_dossier"]

[uscite.south]
verso = "corridor"
descrizione = "the ladder going down to the corridor"
+++

## Description
The bridge of the Rusty Badger: a panoramic window onto the stars, faux-leather chairs patched with duct tape, coloured buttons whose purpose nobody remembers. In the middle, AUNTIE's main terminal, with a smiling icon shaped like a ball of wool. In a corner, under a red plastic cover, a huge button labelled "SELF-DESTRUCT (NO, REALLY)".

## Details
- The terminal has two menus: "SHIP FILES" and "MEGACORP CLASSIFIED FILES — CAPTAIN'S CARD REQUIRED".
- Next to the self-destruct button, a numeric keypad and a card slot.
- Through the window you can see Mold-13 drifting away: an asteroid shaped like a mouldy loaf of bread.

## Secrets
- The classified files reveal the conspiracy: MegaCorp Unlimited diverted the ship to recover the eggs and breed giant chickens for the **BFB - Betelgeuse Fried Blobchicken** chain; the crew was classified as "expendable biomass".
- The self-destruct requires the captain's key card **and** the code (from the note in the locker).

## Mechanics
- Classified files: they require `captain_card` in the inventory. Reading them sets `conspiracy_revealed = true` (+15) and makes `megacorp_dossier` available to print/take. AUNTIE, shocked: "Expendable biomass? I used to make you chamomile tea!" From then on AUNTIE is 100% on Molly's side and calls her "my sweetie".
- Self-destruct: it requires `captain_card` in the inventory and `code_known = true` (if the player tries random codes without knowing it, AUNTIE scolds them: "Sweetie, it's not 0000. Nor your birthday."). Activating it sets `self_destruct_active = true`, `self_destruct_turn` = current turn (+10), and AUNTIE announces: "Self-destruct in 10 minutes. The hangar hatch is unlocked. Remember the cat!".
- Before activating the self-destruct, if the player has neither `mr_whiskers` nor `cryo_cannon`, AUNTIE asks for confirmation ("Are you sure, sweetie? Have you got everything?"): if the player confirms in the same action or in the next one, it goes ahead.
- The Rusty Badger cannot be piloted: it is on autopilot and the controls are locked by MegaCorp.
