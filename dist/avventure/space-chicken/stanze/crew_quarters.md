+++
id = "crew_quarters"
nome = "The crew quarters"
oggetti = ["cat_carrier"]
oggetti_nascosti = ["mr_whiskers", "captain_card", "captain_note"]

[uscite.east]
verso = "corridor"
descrizione = "the passage to the main corridor"

[uscite.up]
verso = "air_ducts"
descrizione = "a ventilation grate in the ceiling, above the top bunk"
richiede_flag = "grate_open"
nascosta = true
+++

## Description
Six narrow bunks, crumpled blankets, posters of space bands and socks everywhere. At the back, the captain's personal locker, with a sticker "DO NOT TOUCH — CAPTAIN". On one bunk there is an empty cat carrier with its door open. From under a bunk comes a threatening hiss.

## Details
- Under the bunk is Mr. Whiskers, the ship's ginger cat: fur puffed up, huge eyes, deeply offended with the universe.
- The captain's locker is closed with a cheap padlock: it opens with a well-aimed blow or simply by pulling hard.
- Above the top bunk there is a ventilation grate held by four loose screws.

## Secrets
- In the captain's locker are the `captain_card` and the `captain_note` with the self-destruct code ("Self-destruct code: 1-2-3-4. Don't tell anyone. P.S. buy bait").
- The grate leads into the air ducts, which connect the crew quarters to the laboratory: an alternative (and dangerous) way to get in without the key card.

## Mechanics
- Opening/forcing the locker: set `locker_open = true` and make `captain_card` and `captain_note` visible. Reading the note (or taking it) sets `code_known = true`. Taking the key card: +5.
- Mr. Whiskers: trying to grab him bare-handed → a scratch and he runs under another bunk (no change of state, turn lost). Opening the `tuna_can` or offering it to him makes him come out (make `mr_whiskers` visible; the can is used up, remove it from the inventory). To take him the `cat_carrier` must be in the inventory (+5). Without the carrier, the cat runs away again.
- Unscrewing or forcing the grate: set `grate_open = true` and mention the exit upwards.
- AUNTIE, if spoken to here: "Mr. Whiskers hasn't eaten since yesterday. Like everybody else, really. Well, almost everybody."
