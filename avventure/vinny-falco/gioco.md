+++
titolo = "Vinny Falco: The Devil's Slide Affair"
descrizione_breve = "San Francisco, 1928. A rich colonel, a son in trouble, a French widow with too perfect a story. A hard-boiled noir."
autore = "Inspired by the novel \"Vinny Falco Detective\" by Peter A. Ray (Studio Gronk!)"
versione = "1.0"
lingua = "en"
stanza_iniziale = "office"
max_turni = 100
inventario_iniziale = ["revolver"]
spostamenti_liberi = false

[flag_iniziali]
case_accepted = false
colonel_met = false
julius_seen = false
following_julius = false
den_found = false
julius_on_ferry = false
julius_door_open = false
heroin_found = false
julius_report_done = false
jeanne_visited = false
violets_noticed = false
violets_linked = false
dinner_invited = false
cliff_dinner_done = false
binder_read = false
will_known = false
telegram_sent = false
telegram_sent_turn = 0
telegram_read = false
identity_deduced = false
finale_started = false
finale_turn = 0
shots_fired = false
julius_saved = false
ada_unmasked = false

[oggetti]
revolver = "your .38 Colt Police Positive revolver"
lockpicks = "a roll of lock picks wrapped in chamois leather"
heroin_kit = "a metal box with syringes, blackened spoons and a pouch of whitish powder"
anonymous_letter = "a typewritten anonymous letter, with no signature"
manson_invitation = "a note from lawyer Manson on headed paper: \"Dinner at the Cliff House, seven o'clock. We must talk.\""
miles_folder = "a mustard-coloured folder on Miles Harrington"
case_binder = "a thick red-banded binder with every report on the Granger case"
telegram = "a telegram from the New York detective agency"
+++

## Introduction
San Francisco, October 1928. Prohibition is the law, bootleg bourbon is the custom, and Herbert Hoover promises a chicken in every pot. You are **Vinny Falco**, private eye of *Falco & Associates*. The "Associates" is a bit of advertising: the only partner is you, but the gilt letters on the door give the place a touch of respectability. The peeling plaster on the landing tells another story.

It is a lazy afternoon. King Oliver's cornet moans from your new valve radio, the gift of a lucky day at the races, and you are dozing with your feet on the desk. Then the telephone rings.

## Setting
San Francisco at the end of the Roaring Twenties: fog rolling in from the bay, cable cars, speakeasies, Chinatown's red lanterns, the rotten heart of the Tenderloin, the old Montgomery Block where artists and nobodies live behind brass name plates. South of the city the coast road climbs to **Devil's Slide**, where the Granger villa hangs over the ocean.

The characters:
- **Vinny Falco** (the player): a weary, honest, ironic private eye. Only lawyer Manson and his mother call him "Vincenzo". Drives a Ford Roadster, owns one jacket.
- **Arthur Manson**: the Grangers' lawyer. Precise, silky voice, maniacally tidy files, distrustful by profession. Calls Vinny "Vincenzo".
- **Colonel Elias Granger**: old, paralysed, in a wheelchair, eyes that pierce you. Made his fortune smuggling arms and trading with the Orient. Widower of Désirée Fontenot, a Creole beauty who died giving birth to their second son.
- **Miss Maria Granger**: the colonel's spinster sister. She raised the boys. Few words, wet eyes.
- **Anthony Granger**: the elder son, a hero. Volunteer pilot in France, shot down in 1918 a few days before the armistice.
- **Julius Granger**: the younger son, 26, would-be poet, kept by his father. Lives in the Montgomery Block. Asks for more and more money and refuses to explain why.
- **"Jeanne Roche"**: a beautiful, sophisticated Frenchwoman with dark red hair and a violet perfume. A year ago she appeared claiming to be Anthony's widow: married in France in 1918, at the field hospital where she was a nurse. Papers in order, signature authentic. The colonel recognised her with a generous annuity but cut her out of his will.
- **Miles Harrington**: from a respectable Baltimore family, a war correspondent wounded in 1918, witness at the wedding. Limps slightly. Now lives in San Francisco, engaged to Jeanne, and has become the favourite of Miss Maria.
- **Jimmy Delaney**: a young private eye trying to make his way. Vinny, half a lifetime ago.
- **Madame Sybilla**: the Chinatown fortune teller who reads the future in tea leaves. Theatrical, but she sees more than you would think.

## Secrets
- Julius is a heroin addict: that is where the money goes.
- The Harrington family is ruined: the father lost almost everything investing in dry oil fields. Only the name and appearances remain.

**The truth (the player must reconstruct it):** the real Jeanne Roche died of Spanish flu after the war. Her best friend and fellow nurse **Adèle Morel**, the bride's witness, married an American soldier named Costello, followed him to the States and reinvented herself as **Ada Costello**, a nightclub singer. Years later she met Miles again by chance: together they planned the swindle. In February 1927 they sailed to Europe; with forged papers Adèle became "Jeanne", they recovered the marriage certificate, and Miles killed the only person who could recognise the fraud, the mayor of Valduremont who had celebrated the wedding (found drowned, "an accident"). The colonel changed his will, so the pair decided to remove Julius: with no blood heir, "Anthony's widow" could claim the bonds and even the presidency of the charitable trust that will receive the whole fortune. Under her real name Ada rents **apartment 1B of the Montgomery Block**, next door to Julius. She sends herself anonymous letters, delivered by a small-time crook, so that they will look like Julius's work, and she hires a young detective (Jimmy) to tail the delivery man and be the unwitting witness. On the chosen day, when Julius comes to his flat escorted by Miles, Ada knocks him out with chloroform and injects him with a lethal dose of heroin; then the staged shoot-out pins everything on the delivery man. But Ada does not want to share: she also kills Miles.

## Goals
1. Accept lawyer Manson's job and meet the colonel at Devil's Slide.
2. Find out where Julius's money goes (the Tenderloin den) and confirm it by searching his flat.
3. Report to the colonel.
4. Listen to "Jeanne Roche" and examine her anonymous letter.
5. Investigate Miles Harrington for Manson: the folder, the binder of the old investigation, the will.
6. Have a telegram sent to the New York agency and read the answer.
7. Understand who "Jeanne Roche" really is and stop her, possibly before Julius dies.

## Victory conditions
Victory when `ada_unmasked = true` (Vinny confronts the woman calling her by her true identity, Ada Costello / Adèle Morel, in front of witnesses or the police).
- If `julius_saved = true`: final narration of the **good ending**. Julius wakes up in the hospital, Miles confesses everything to save his own skin, Ada is taken away in handcuffs without a word, with the last smile of an actress who has lost her audience. At the villa Manson adds the final line: the real Jeanne and Anthony married because she was expecting a child; the boy grew up in an orphanage in a small French village. The Grangers have an heir. And Manson, for the first time, calls him "Vinny".
- Otherwise: final narration of the **bitter ending**. Behind the door of 1B a single shot: the leading lady has refused to speak her last line. The Chronicle puts Jimmy's smiling photo on the crime page, among articles celebrating Hoover's victory. At the villa Vinny tells the two old people the whole story; Manson reveals the heir in France. Outside, the cold wind smells of Christmas, and Manson called him "Vinny".

## Defeat conditions
- Vinny accepts or confirms the staged version of the shooting (the delivery man killed everyone, or Julius wrote the letters) in front of the police and walks away, or accuses the wrong person as the mastermind (Julius, Maria, Manson, Jimmy, the delivery man alone) to the inspector: the case is closed the wrong way, "Jeanne" inherits → `esito = "sconfitta"`.
- Vinny shoots someone who is not threatening him → `esito = "sconfitta"` (his licence ends up in the bay).
- The engine declares defeat at turn 100 (the woman gets away with it): the AI does not need to do it.
Defeats are narrated in a dry, bitter noir style.

## Tone and style
Hard-boiled noir in the style of the 1920s–30s pulps: short sentences, cynical similes with a smile ("The city opened in front of me like a knife through rancid butter"), smells of coffee, fog, gun oil and cheap tobacco. Second person, present tense, but with Vinny's ironic inner voice. Food matters (the diner's beef pot pie, oysters at the Cliff House). Violence is never gory: it is told through details, silence and the smell of gunpowder. Drug addiction is treated seriously, without moralising and without complacency.

## World rules
- **Time** passes by days: travelling across the city or along the coast takes a few hours, stakeouts eat whole mornings. Mention the changing weather (warm October, then November fog and wind).
- **Clues**: the AI never solves the case for the player. Vinny can comment, reason aloud and ask questions, but the decisive connections (Ada Costello = "Jeanne Roche"; the violet perfume; the trip to Europe) must come from the player. When the player states a deduction that the clues support, the narration confirms it with a noir flourish.
- **Deducing the identity** (`identity_deduced = true`, +15): only after `telegram_read = true`, when the player explicitly connects Ada Costello (apartment 1B, next door to Julius) with "Jeanne Roche" (better still if they also mention Adèle Morel, the vanished bride's witness, or the violet perfume).
- **The finale** (see the diner and Montgomery Block rooms): it starts when Vinny arrives at the diner with `telegram_read = true`. From that moment the clock runs: what happens depends on how many turns pass after `finale_turn`.
- **Wrong accusations before the finale**: if Vinny accuses someone without proof, the characters react (Manson cools down, the colonel throws him out, Maria cries), but there is no defeat: only the accusation in front of the police in the finale counts.
- **Violence**: the `revolver` is for threatening and defending oneself. Vinny draws it only if the player asks. Nobody dies at Vinny's hand except as a last resort in self-defence.
- **Score**: +5 colonel met, +5 den found, +10 heroin found, +10 report to the colonel, +5 violets noticed, +5 dinner at the Cliff House, +5 binder read, +5 will understood, +10 telegram read, +15 identity deduced, +15 Ada unmasked, +20 Julius saved.
- Few, precise items: do not invent weapons, keys, documents or other usable tools. Furniture, food and drinks can be described but not taken.
