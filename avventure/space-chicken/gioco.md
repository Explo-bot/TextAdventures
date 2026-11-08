+++
titolo = "Star Trucker and the Space Chicken of Doom"
descrizione_breve = "A space freighter, alien eggs and monstrous chickens. A B-movie sci-fi comedy with mozzarella."
autore = "Example for the Avventura engine"
versione = "1.0"
lingua = "en"
stanza_iniziale = "mess_hall"
max_turni = 80
inventario_iniziale = []
spostamenti_liberi = false

[flag_iniziali]
grate_open = false
locker_open = false
code_known = false
weakness_found = false
conspiracy_revealed = false
self_destruct_active = false
self_destruct_turn = 0
ambushes_suffered = 0
seatbelt_fastened = false
monster_held_off = false
monster_frozen = false
monster_ejected = false
shuttle_launched = false

[oggetti]
frying_pan = "a dented non-stick galley frying pan"
tuna_can = "a can of space tuna in brine"
eggshell = "a fragment of alien eggshell covered in greenish slime"
cat_carrier = "a cat carrier labelled \"MR. WHISKERS — DO NOT OPEN\""
mr_whiskers = "Mr. Whiskers, the ship's ginger cat, deeply offended, in the carrier"
captain_card = "Captain Hal Brannigan's magnetic key card"
captain_note = "a greasy note with the self-destruct code written on it"
megacorp_dossier = "a printout of MegaCorp Unlimited's classified files"
sour_mozzarella = "a soured Mizardian buffalo mozzarella with a devastating smell"
wrench = "a giant wrench"
rubber_hose = "a flexible rubber hose"
duct_tape = "a roll of duct tape (the true hero of space)"
coolant_tank = "a tank of cryogenic coolant, icy to the touch"
cryo_cannon = "an improvised cryo cannon: tank, hose and lots of duct tape"
+++

## Introduction
Year 2122. The space freighter **Rusty Badger** was on its way back to Earth with a load of Mizardian buffalo mozzarella when AUNTIE, the ship's computer, woke the crew because of a signal from the asteroid Mold-13. On the asteroid there was an alien wreck full of giant eggs. Somebody had the brilliant idea of bringing a few on board.
While the captain was suggesting an omelette and the science officer insisted on taking them back to Earth, the eggs hatched. Dozens of chicks with teeth vanished into the air ducts. Since then they have grown. A lot. And they are hungry.

You are **Molly**, the ship's mechanic. You have just come out of the mess hall pantry, where you were hiding. The ship is silent. Too silent. You are the only survivor… apart, maybe, from Mr. Whiskers, the ship's cat.

## Setting
An affectionate parody of 1970s–80s sci-fi horror films and B-movies: corridors full of steam for no reason, flashing lights, a passive-aggressive ship's computer and ridiculous but deadly monsters.
The Rusty Badger is an old, heavily patched freighter. AUNTIE is a motherly, sarcastic artificial intelligence who talks like a caring and slightly scatterbrained aunt ("Sweetie, have you eaten? Oh, no, right, they're eating you.").
The creatures, which the player can call whatever they like (the crew called them "Space Chickens"), are a cross between chicks and velociraptors: feathers, teeth, a "PEEOK!" call, very fast, growing before your eyes.
The failure of the cargo hold's cryogenic system made the mozzarella go sour: the smell is unbearable for humans and **terrifies the creatures**, which is why they avoid the cargo hold. No crew member noticed it in time.
The conspiracy: MegaCorp Unlimited knew about the eggs and diverted the ship to Mold-13 to recover them and breed genetically modified giant chickens for the galactic fast-food chain **BFB - Betelgeuse Fried Blobchicken**. The crew was considered "expendable".

## Goals
1. Survive the creature invasion.
2. Find items and build an improvised weapon (the cryo cannon).
3. Discover the creatures' weak point (sour dairy products) by analysing a biological sample in the laboratory.
4. Uncover MegaCorp's conspiracy (classified files on the bridge).
5. Save Mr. Whiskers.
6. Activate the self-destruct and escape with the rescue shuttle, defeating the last monster.

## Victory conditions
Victory when `monster_ejected = true` and `shuttle_launched = true`.
Final narration: the shuttle flies away while the Rusty Badger explodes in the background in a huge fireball (vaguely) shaped like a chicken. Molly records the ship's log: "If anyone finds more alien eggs, please… make yourselves a carbonara and leave them where they are." Last shot: Mr. Whiskers silently stares at a single alien feather hidden under the seat… maybe the nightmare isn't over.
If `conspiracy_revealed = true`, Molly adds in the log that she will hand the MegaCorp dossier to the press: "Betelgeuse Fried Blobchicken? More like Betelgeuse Fried Bull*chicken*."

## Defeat conditions
- `ambushes_suffered` reaches 2 (see World rules): Molly is overwhelmed → `esito = "sconfitta"`.
- The self-destruct is active and 10 turns have passed since `self_destruct_turn` without `shuttle_launched` being true → the ship explodes with Molly on board → `esito = "sconfitta"`.
- Molly opens the shuttle's emergency hatch without `seatbelt_fastened = true` → she is sucked out into space together with the monster → `esito = "sconfitta"`.
- The engine declares defeat at turn 80 (the creatures, fully grown by now, invade every corner of the ship): the AI does not need to do it.
Defeats must be narrated in a comic but clear way, with a last line from AUNTIE.

## Tone and style
B-movie sci-fi comedy: horror-film tension constantly ruined by absurd details. Fast pace, short lines, written sound effects ("PEEOK!", "SBLAM!", "fssssss"). AUNTIE often chimes in from the loudspeakers with motherly, sarcastic comments. Never gory: when the creatures "eliminate" someone, only feathers and a pair of boots are left. Descriptions still stay precise about items and exits, because the player must be able to reason.

## World rules
- **Creature growth** (mention it when one is seen or heard): turns 1–25 as big as hens with teeth; turns 26–50 as big as turkeys and bolder; turns 51–80 as big as ostriches, and they can be heard walking above the ceiling.
- **Ambushes**: in the rooms listed in their Mechanics (air ducts, engine room) a creature attacks. If Molly has `sour_mozzarella` in the inventory, the creature sniffs the air, goes "PEEOK?!" in disgust and runs away: no harm done. Otherwise Molly manages to escape to the room she came from, loses the turn and `ambushes_suffered` increases by 1 (at the second ambush suffered: defeat). The `frying_pan` can fend off **one** attack without increasing `ambushes_suffered`, but the pan bends and is lost (remove it from the inventory).
- **Self-destruct**: when it is activated set `self_destruct_active = true` and `self_destruct_turn` = current turn. From then on AUNTIE announces the countdown every turn ("Sweetie, 7 minutes to the explosion, did you take your jumper?"). After 10 turns without `shuttle_launched` → defeat.
- **Combinations**: the player can combine items. Valid combination: `coolant_tank` + `rubber_hose` + `duct_tape` → `cryo_cannon` (remove the tank and the hose; the duct tape stays, it is infinite like in every film). Other sensible combinations can be narrated as funny attempts but do not create new items.
- **Mr. Whiskers** can only be in the carrier: `mr_whiskers` can enter the inventory only if `cat_carrier` is there.
- **Discovering the weak point**: the player can also work it out on their own (the cargo hold is the only place with no trace of the creatures); `weakness_found` is for the score and for AUNTIE to comment on the discovery, it is not required to use the mozzarella.
- **Score**: +5 key card, +5 Mr. Whiskers in the carrier, +10 sample analysis (`weakness_found`), +15 conspiracy revealed, +10 cryo cannon, +10 self-destruct activated, +15 monster frozen, +15 monster ejected, +15 shuttle launch.
- Few, precise items: do not invent weapons, flamethrowers, suits or other usable tools. Furniture can be described but not taken.
