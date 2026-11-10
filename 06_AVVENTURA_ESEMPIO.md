# 06 — Avventure di esempio: "The Punta Nera Lighthouse", "Star Trucker and the Space Chicken of Doom" e "Vinny Falco: The Devil's Slide Affair"

**Modello consigliato:** Claude Sonnet 5.5 (`claude-sonnet-5-5`). I contenuti sono già scritti qui sotto: il lavoro è creare i file, validarli e collaudarli.
**Durata indicativa:** 10–20 minuti.
**Prerequisiti:** script 02–05 completati (serve l'eseguibile per `valida`).

---

## Istruzioni per Claude Code

Crea nella radice le cartelle di **tre** avventure con i file riportati sotto, **copiandone il contenuto esattamente** (sono già conformi al §3 di `01_SPECIFICHE.md`):

1. `avventure/faro-punta-nera/` — *The Punta Nera Lighthouse* / *Il Faro di Punta Nera* (racconto marinaresco, 6 stanze, ~40 turni);
2. `avventure/space-chicken/` — *Star Trucker and the Space Chicken of Doom* (commedia sci-fi da B-movie, 10 stanze, ~80 turni);
3. `avventure/vinny-falco/` — *Vinny Falco: The Devil's Slide Affair* / *Vinny Falco: il caso di Devil's Slide* (noir hard-boiled nella San Francisco del 1928, ispirato al romanzo *Vinny Falco Detective* di Peter A. Ray; 11 stanze, ~100 turni).

Le avventure sono scritte **in inglese**, id di stanze, oggetti e flag e direzioni (`north`, `up`…) compresi: il prompt per l'AI è così interamente in inglese, e i modelli lo seguono meglio (soprattutto quelli locali piccoli). Si giocano comunque **in italiano, inglese, francese, spagnolo o tedesco**: il narratore AI scrive nella lingua dell'interfaccia del giocatore (§7.6) e i file `traduzioni/it.toml`, `fr.toml`, `es.toml`, `de.toml` (§3.4) danno la versione italiana, francese, spagnola e tedesca dei testi che il motore mostra da sé (titolo, descrizione breve, introduzione, nomi di stanze e oggetti, descrizioni delle uscite). Il glossario dei nomi nella lingua del giocatore finisce anche nel prompt, perché l'AI chiami stanze e oggetti sempre allo stesso modo.

Se una delle cartelle esiste già (per esempio creata da un'esecuzione precedente), **non sovrascriverla**: confronta i file con quelli di questo script e, se differiscono, mostra le differenze all'utente e chiedi se aggiornarli. Se esistono le cartelle `faro-punta-nera-en/` o `space-chicken-en/` di una versione precedente (quando ogni storia aveva una versione italiana e una inglese), chiedi all'utente se eliminarle. Poi esegui i controlli in fondo.

Ogni blocco di codice qui sotto, preceduto dal percorso del file, è il contenuto completo di quel file (senza le tre righe di backtick).

---

## Avventura 1 — "The Punta Nera Lighthouse" (in italiano "Il Faro di Punta Nera")

Racconto marinaresco: ottobre 1893, il faro è spento e un brigantino punta verso gli scogli; bisogna riaccendere la lampada e rimettere in rotazione la lente, e possibilmente soccorrere il guardiano Anselmo.

### Struttura

```
avventure/faro-punta-nera/
├── gioco.md
├── stanze/
│   ├── cliff.md
│   ├── boathouse.md
│   ├── entrance.md
│   ├── cellar.md
│   ├── stairs.md
│   └── lantern_room.md
└── traduzioni/
    ├── it.toml
    ├── fr.toml
    ├── es.toml
    └── de.toml
```

### Mappa (per riferimento, non è un file)

```
                 [lantern_room]
                     | up (needs brass_key)
                  [stairs]
                     | up
   [cliff] --north--> [entrance] --down (needs trapdoor_open)--> [cellar]
      | east
  [boathouse]
```

---

### `avventure/faro-punta-nera/gioco.md`

```markdown
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
```

---

### `avventure/faro-punta-nera/stanze/cliff.md`

```markdown
+++
id = "cliff"
nome = "The cliff below the lighthouse"
oggetti = []
oggetti_nascosti = []

[uscite.north]
verso = "entrance"
descrizione = "the paved path climbing to the lighthouse door"

[uscite.east]
verso = "boathouse"
descrizione = "a wooden boathouse, built against the rock"
+++

## Description
You are on the path that runs along the cliff. The rain lashes your face and the waves explode on the rocks a few metres below. To the north, the black shape of the lighthouse stands out against the sky: at the top, where the light should be turning, there is only darkness. To the east, a plank boathouse bangs its door in the wind. Out at sea, the lights of a brigantine.

## Details
- The lighthouse door, at the top of the path, is ajar: Anselmo never leaves it like that.
- On the path there are muddy footprints going towards the lighthouse and not coming back.
- Looking at the sea you can see the brigantine's navigation lights (see World rules for the distance).

## Secrets
None. It is the starting point and helps the player find their bearings.

## Mechanics
- If the player tries to climb down onto the rocks or to signal the ship by hand: it is useless and dangerous; narrate that the waves drive them back, no change of state.
- If the player tries to run to the village for help: it is too far, they realise it after a few steps; they stay here (the turn is wasted).
```

---

### `avventure/faro-punta-nera/stanze/boathouse.md`

```markdown
+++
id = "boathouse"
nome = "The boathouse"
oggetti = ["empty_can"]
oggetti_nascosti = ["brass_key"]

[uscite.west]
verso = "cliff"
descrizione = "the banging door, towards the path"
+++

## Description
A narrow, damp boathouse that smells of tar and seaweed. A small rowing boat lies upside down on two trestles. Against the wall there are tangled nets, a box of rusty tools and a tin can. Through the door, to the west, the rain blows in.

## Details
- The tin can is empty but intact, with a screw cap: good for carrying liquids.
- The tool box only holds rusty nails and a hammer with a broken handle: useless.
- Under the upturned boat, looking closely, something glints.

## Secrets
Anselmo hides the key to the lantern room (`brass_key`) under the upturned boat, hanging from a nail in the hull, because he is afraid of oil thieves.

## Mechanics
- If the player lifts, turns over, flips or looks under the boat: set `boat_turned = true` and make `brass_key` visible by adding it to the room items (`oggetti_stanza_aggiungi`). If they explicitly take it, put it in the inventory (+10 points).
- Taking the key in the same turn in which the boat is turned over is allowed.
- The nets and the tools are not items that can be taken.
```

---

### `avventure/faro-punta-nera/stanze/entrance.md`

```markdown
+++
id = "entrance"
nome = "The lighthouse entrance"
oggetti = ["oilskin", "logbook"]
oggetti_nascosti = ["matches"]

[uscite.south]
verso = "cliff"
descrizione = "the half-open door you came in through"

[uscite.up]
verso = "stairs"
descrizione = "the foot of the spiral staircase climbing into the tower"

[uscite.down]
verso = "cellar"
descrizione = "a wooden trapdoor in the floor"
richiede_flag = "trapdoor_open"
nascosta = true
+++

## Description
The ground-floor room of the lighthouse: whitewashed stone walls, a cold cast-iron stove, a table with the lighthouse logbook lying open, an old worn rug on the floor. From a hook beside the door hangs a yellow sailor's oilskin, still dripping. A spiral staircase climbs into the darkness of the tower.

## Details
- The dripping oilskin is Anselmo's: he came back not long ago, so he is somewhere around here.
- The lighthouse logbook (`logbook`), on its last line, in Anselmo's shaky handwriting: "Oil almost gone. Going down to fill the can."
- The rug is crumpled and crooked, as if it had been moved in a hurry and put back badly.
- From under the floor, if the player listens carefully, comes a muffled groan.

## Secrets
- In the inside pocket of the oilskin are the matches, wrapped in oilcloth and therefore dry.
- Under the rug is the trapdoor to the cellar, where Anselmo is.

## Mechanics
- If the player searches, examines or puts on the oilskin, or checks its pockets: make `matches` visible (add it to the room items) and, if they take them, put them in the inventory (+10 points).
- If the player moves/lifts the rug or searches the floor: they discover the trapdoor; if they open it (even in the same action) set `trapdoor_open = true` and mention the exit downwards.
- Shouting or calling Anselmo: a weak answer is heard coming "from below", without explicitly revealing the trapdoor.
- Lighting the stove is useless and uses up a match: discourage it in the narration but respect the choice.
```

---

### `avventure/faro-punta-nera/stanze/cellar.md`

```markdown
+++
id = "cellar"
nome = "The cellar"
oggetti = []
oggetti_nascosti = ["crank"]

[uscite.up]
verso = "entrance"
descrizione = "the ladder leading back up to the trapdoor"
+++

## Description
You climb down a slippery ladder into total darkness. The air smells of rancid oil, wet stone and brine. You cannot see anything. Somewhere, not far away, someone is breathing with difficulty.

## Details
(Visible only with `match_lit = true`.)
- Anselmo is lying at the foot of the ladder, pale, one ankle swollen and twisted.
- At the back, a large oil barrel with a brass tap.
- One of the rungs of the ladder is broken: that is how Anselmo fell.

## Secrets
- Anselmo has in his pocket the `crank` used to wind the lens's clockwork mechanism: without it the lens does not turn and the light stays fixed and useless as a signal.
- The barrel is the lighthouse's only supply of oil.

## Mechanics
- In the dark the player cannot fill the can or see Anselmo, but can talk to him. Anselmo, in the dark, says: "Nicola! Strike a match, for God's sake… the lamp… the oil is here."
- Striking a match: decrease `matches_left`, set `match_lit = true`. With the light the player sees everything (Details). If `matches_left` is 0, they cannot strike one.
- Helping Anselmo (talking to him and helping him get comfortable, splinting his ankle, giving him the oilskin…): set `anselmo_helped = true` (+15 points); Anselmo hands over the `crank` (add it to the inventory) and says: "Without this the lens won't turn. Wind the mechanism all the way, then light the lamp."
- Just by talking to Anselmo the player gets the crank, but `anselmo_helped` becomes true only if they do something concrete to help him.
- Filling the can at the barrel: requires `empty_can` in the inventory and light (`match_lit = true` in this turn, or in the same turn in which they strike one). Remove `empty_can` and add `full_can` to the inventory.
- Anselmo cannot move and cannot climb: if the player tries to carry him up, they cannot do it alone; Anselmo insists that the lighthouse comes first.
- Bringing the lit match close to the open barrel → fire → `esito = "sconfitta"` (warn first with a moment of hesitation if the intention is ambiguous).
```

---

### `avventure/faro-punta-nera/stanze/stairs.md`

```markdown
+++
id = "stairs"
nome = "The spiral staircase"
oggetti = []
oggetti_nascosti = []

[uscite.down]
verso = "entrance"
descrizione = "the steps going down to the entrance"

[uscite.up]
verso = "lantern_room"
descrizione = "a locked iron door at the top of the stairs"
richiede_oggetto = "brass_key"
+++

## Description
A hundred and twenty stone steps wind around the central pillar of the tower. The arrow slits let in gusts of rain. Halfway up, a small window looks out to sea. At the top, an iron door.

## Details
- From the window you can see the brigantine (distance according to the World rules).
- One of the steps halfway up is cracked and wobbly.
- The iron door at the top has a big lock; without the key it will not open.

## Secrets
The cracked step gives way if someone runs or jumps over it.

## Mechanics
- If the player says they run/rush up/jump the steps: the step gives way, they fall and are slightly hurt; they stay in `stairs` and lose the turn. If at that moment they have `full_can`, the oil spills: replace `full_can` with `empty_can` (they will have to go back to the cellar).
- Climbing normally: no problem.
- Without `brass_key` the door does not open (the engine prevents the move): narrate the vain attempt and hint that Anselmo keeps the key "in a safe place".
- Forcing the door is impossible.
```

---

### `avventure/faro-punta-nera/stanze/lantern_room.md`

```markdown
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
```

---

### `avventure/faro-punta-nera/traduzioni/it.toml`

```toml
# Traduzione italiana dei testi che il motore mostra direttamente al giocatore.
# La narrazione la scrive l'AI in italiano; questi nomi le vengono passati
# perché li usi sempre uguali. Tutto ciò che manca resta in inglese.

titolo = "Il Faro di Punta Nera"
descrizione_breve = "Una notte di tempesta, un faro spento, una nave che punta dritta verso gli scogli."
introduzione = """
Ottobre 1893. Sei Nicola, diciassette anni, aiutante del vecchio guardiano Anselmo al faro di Punta Nera.
Sei sceso al paese nel pomeriggio per le provviste; ora è notte, il vento urla e la pioggia arriva di traverso.
Risalendo il sentiero ti accorgi di una cosa che ti gela il sangue: **il faro è spento**.
Laggiù, oltre le onde, brillano le luci di posizione di un brigantino che punta dritto verso gli scogli.
"""

[stanze]
cliff = "La scogliera sotto il faro"
entrance = "L'ingresso del faro"
cellar = "La cantina"
boathouse = "La rimessa delle barche"
stairs = "La scala a chiocciola"
lantern_room = "La sala della lanterna"

[oggetti]
brass_key = "una chiave d'ottone annerita"
empty_can = "una tanica di latta vuota"
full_can = "una tanica di latta piena d'olio"
oilskin = "la cerata gialla del guardiano"
matches = "una scatola di fiammiferi avvolta nella tela cerata"
crank = "una manovella di ferro"
logbook = "il diario di bordo del faro"

[uscite]
"cliff.north" = "il sentiero lastricato che sale alla porta del faro"
"cliff.east" = "una rimessa per barche di legno, addossata alla roccia"
"entrance.south" = "la porta socchiusa da cui sei entrato"
"entrance.up" = "l'inizio della scala a chiocciola che sale nella torre"
"entrance.down" = "una botola di legno nel pavimento"
"cellar.up" = "la scala a pioli che risale alla botola"
"boathouse.west" = "la porta che sbatte, verso il sentiero"
"stairs.down" = "i gradini che scendono all'ingresso"
"stairs.up" = "una porta di ferro chiusa a chiave, in cima alla scala"
"lantern_room.down" = "la porta di ferro e la scala a chiocciola"
```

---

### `avventure/faro-punta-nera/traduzioni/fr.toml`

```toml
# Traduzione francese dei testi che il motore mostra direttamente al giocatore.
# La narrazione la scrive l'AI in francese; questi nomi le vengono passati
# perché li usi sempre uguali. Tutto ciò che manca resta in inglese.

titolo = "Le Phare de Punta Nera"
descrizione_breve = "Une nuit de tempête, un phare éteint, un navire qui fonce droit sur les rochers."
introduzione = """
Octobre 1893. Vous êtes Nicola, dix-sept ans, aide du vieux gardien Anselmo au phare de Punta Nera.
Vous êtes descendu au village dans l'après-midi pour les provisions ; maintenant il fait nuit, le vent hurle et la pluie tombe de biais.
En remontant le sentier, vous remarquez une chose qui vous glace le sang : **le phare est éteint**.
Là-bas, au-delà des vagues, brillent les feux de position d'un brick qui fonce droit sur les rochers.
"""

[stanze]
cliff = "La falaise sous le phare"
entrance = "L'entrée du phare"
cellar = "La cave"
boathouse = "Le hangar à bateaux"
stairs = "L'escalier en colimaçon"
lantern_room = "La salle de la lanterne"

[oggetti]
brass_key = "une clé en laiton noircie"
empty_can = "un bidon en fer-blanc vide"
full_can = "un bidon en fer-blanc plein d'huile"
oilskin = "le ciré jaune du gardien"
matches = "une boîte d'allumettes enveloppée dans de la toile cirée"
crank = "une manivelle en fer"
logbook = "le journal de bord du phare"

[uscite]
"cliff.north" = "le sentier pavé qui monte jusqu'à la porte du phare"
"cliff.east" = "un hangar à bateaux en bois, adossé à la roche"
"entrance.south" = "la porte entrouverte par laquelle vous êtes entré"
"entrance.up" = "le bas de l'escalier en colimaçon qui monte dans la tour"
"entrance.down" = "une trappe en bois dans le plancher"
"cellar.up" = "l'échelle qui remonte vers la trappe"
"boathouse.west" = "la porte qui claque, vers le sentier"
"stairs.down" = "les marches qui redescendent vers l'entrée"
"stairs.up" = "une porte en fer fermée à clé, en haut de l'escalier"
"lantern_room.down" = "la porte en fer et l'escalier en colimaçon"
```

---

### `avventure/faro-punta-nera/traduzioni/es.toml`

```toml
# Traduzione spagnola dei testi che il motore mostra direttamente al giocatore.
# La narrazione la scrive l'AI in spagnolo; questi nomi le vengono passati
# perché li usi sempre uguali. Tutto ciò che manca resta in inglese.

titolo = "El Faro de Punta Nera"
descrizione_breve = "Una noche de tormenta, un faro apagado, un barco que va directo hacia las rocas."
introduzione = """
Octubre de 1893. Eres Nicola, diecisiete años, ayudante del viejo farero Anselmo en el faro de Punta Nera.
Bajaste al pueblo por la tarde a por provisiones; ahora es de noche, el viento aúlla y la lluvia cae de lado.
Al subir de nuevo por el sendero te das cuenta de algo que te hiela la sangre: **el faro está apagado**.
Allá abajo, más allá de las olas, brillan las luces de navegación de un bergantín que va directo hacia las rocas.
"""

[stanze]
cliff = "El acantilado bajo el faro"
entrance = "La entrada del faro"
cellar = "El sótano"
boathouse = "El cobertizo de las barcas"
stairs = "La escalera de caracol"
lantern_room = "La sala de la linterna"

[oggetti]
brass_key = "una llave de latón ennegrecida"
empty_can = "una lata de hojalata vacía"
full_can = "una lata de hojalata llena de aceite"
oilskin = "el impermeable amarillo del farero"
matches = "una caja de cerillas envuelta en hule"
crank = "una manivela de hierro"
logbook = "el cuaderno de bitácora del faro"

[uscite]
"cliff.north" = "el sendero empedrado que sube hasta la puerta del faro"
"cliff.east" = "un cobertizo de madera para barcas, pegado a la roca"
"entrance.south" = "la puerta entornada por la que has entrado"
"entrance.up" = "el arranque de la escalera de caracol que sube a la torre"
"entrance.down" = "una trampilla de madera en el suelo"
"cellar.up" = "la escalera de mano que sube hasta la trampilla"
"boathouse.west" = "la puerta que golpea con el viento, hacia el sendero"
"stairs.down" = "los peldaños que bajan a la entrada"
"stairs.up" = "una puerta de hierro cerrada con llave, en lo alto de la escalera"
"lantern_room.down" = "la puerta de hierro y la escalera de caracol"
```

---

### `avventure/faro-punta-nera/traduzioni/de.toml`

```toml
# Traduzione tedesca dei testi che il motore mostra direttamente al giocatore.
# La narrazione la scrive l'AI in tedesco; questi nomi le vengono passati
# perché li usi sempre uguali. Tutto ciò che manca resta in inglese.

titolo = "Der Leuchtturm von Punta Nera"
descrizione_breve = "Eine stürmische Nacht, ein dunkler Leuchtturm, ein Schiff, das direkt auf die Klippen zuhält."
introduzione = """
Oktober 1893. Du bist Nicola, siebzehn Jahre alt, Gehilfe des alten Leuchtturmwärters Anselmo am Leuchtturm von Punta Nera.
Am Nachmittag bist du ins Dorf hinuntergegangen, um Vorräte zu holen; jetzt ist es Nacht, der Wind heult und der Regen peitscht von der Seite.
Als du den Pfad wieder hinaufsteigst, bemerkst du etwas, das dir das Blut in den Adern gefrieren lässt: **Der Leuchtturm ist dunkel**.
Dort draußen, jenseits der Wellen, leuchten die Positionslichter einer Brigg, die direkt auf die Klippen zuhält.
"""

[stanze]
cliff = "Die Klippe unter dem Leuchtturm"
entrance = "Der Eingang des Leuchtturms"
cellar = "Der Keller"
boathouse = "Das Bootshaus"
stairs = "Die Wendeltreppe"
lantern_room = "Der Laternenraum"

[oggetti]
brass_key = "ein angelaufener Messingschlüssel"
empty_can = "ein leerer Blechkanister"
full_can = "ein Blechkanister voller Öl"
oilskin = "das gelbe Ölzeug des Wärters"
matches = "eine in Wachstuch gewickelte Streichholzschachtel"
crank = "eine eiserne Kurbel"
logbook = "das Logbuch des Leuchtturms"

[uscite]
"cliff.north" = "der gepflasterte Pfad, der zur Tür des Leuchtturms hinaufführt"
"cliff.east" = "ein hölzernes Bootshaus, an den Fels gebaut"
"entrance.south" = "die angelehnte Tür, durch die du hereingekommen bist"
"entrance.up" = "der Fuß der Wendeltreppe, die in den Turm hinaufführt"
"entrance.down" = "eine hölzerne Falltür im Boden"
"cellar.up" = "die Leiter, die zur Falltür hinaufführt"
"boathouse.west" = "die klappernde Tür, zum Pfad hin"
"stairs.down" = "die Stufen, die zum Eingang hinunterführen"
"stairs.up" = "eine verschlossene Eisentür oben an der Treppe"
"lantern_room.down" = "die Eisentür und die Wendeltreppe"
```

---

## Avventura 2 — "Star Trucker and the Space Chicken of Doom"

Commedia sci-fi da B-movie: anno 2122, cargo *Rusty Badger*, polli alieni giganti. Molly deve salvare il gatto, scoprire il complotto della MegaCorp, attivare l'autodistruzione e scappare con la navetta.

### Struttura

```
avventure/space-chicken/
├── gioco.md
├── stanze/
│   ├── mess_hall.md
│   ├── corridor.md
│   ├── crew_quarters.md
│   ├── air_ducts.md
│   ├── laboratory.md
│   ├── bridge.md
│   ├── cargo_hold.md
│   ├── engine_room.md
│   ├── hangar.md
│   └── shuttle.md
└── traduzioni/
    ├── it.toml
    ├── fr.toml
    ├── es.toml
    └── de.toml
```

### Mappa (per riferimento, non è un file)

```
                      [bridge]
                         | north
 [crew_quarters] <-west- [corridor] --east (needs captain_card)--> [laboratory]
     | up (grate_open)    |     | south                                  | up
 [air_ducts] -------------|-----|------------ east ------------------> (air_ducts)
                          | [mess_hall]
                          | down
 [engine_room] <-west- [cargo_hold] --east (needs self_destruct_active)--> [hangar] --in--> [shuttle]
```

---

### `avventure/space-chicken/gioco.md`

```markdown
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
- **Discovering the weak point**: the player may suspect it on their own (the cargo hold is the only place with no trace of the creatures), but Molly only takes a sour mozzarella once she knows what it is for, that is after the analysis in the laboratory (`weakness_found = true`). Before that, she refuses to pick one up with a disgusted excuse about the smell (see the cargo hold Mechanics), without revealing that it would be useful.
- **Score**: +5 key card, +5 Mr. Whiskers in the carrier, +10 sample analysis (`weakness_found`), +15 conspiracy revealed, +10 cryo cannon, +10 self-destruct activated, +15 monster frozen, +15 monster ejected, +15 shuttle launch.
- Few, precise items: do not invent weapons, flamethrowers, suits or other usable tools. Furniture can be described but not taken.
```

---

### `avventure/space-chicken/stanze/mess_hall.md`

```markdown
+++
id = "mess_hall"
nome = "The mess hall"
oggetti = ["frying_pan", "tuna_can", "eggshell"]
oggetti_nascosti = []

[uscite.north]
verso = "corridor"
descrizione = "a sliding door that opens onto the main corridor with a hydraulic sigh"
+++

## Description
The Rusty Badger's mess hall: a steel table, overturned chairs, a motivational poster ("A HAPPY CREW IS A PRODUCTIVE CREW — MegaCorp Unlimited"). On the table, a frying pan with cold butter in it: somebody really wanted to make that omelette. All around, giant eggshells broken from the inside and a carpet of feathers. On the counter, a can of tuna. From the pantry behind you still comes the smell of your hiding place. To the north, the door to the corridor.

## Details
- The shells are as thick as ceramic, sticky with a greenish slime that bubbles. One fragment is small enough to carry.
- On the floor, three-toed footprints lead to a ripped-off ventilation grate in the ceiling, too high to reach.
- Next to the table, Captain Brannigan's cap and a pair of boots. Nothing else.
- AUNTIE, from the loudspeakers: "Oh, sweetie, you're alive! How lovely. The others… well, we'll talk about that later."

## Secrets
The eggshell is the biological sample that, analysed in the laboratory, reveals the creatures' weak point.

## Mechanics
- The frying pan, the can and the eggshell can be taken freely.
- If the player asks AUNTIE what happened, she sums up (comically) the hatching and confirms that the sensors only see Molly and "a small, furry, indignant life form in the crew quarters" (Mr. Whiskers).
- If the player asks AUNTIE how to stop the creatures: "I don't know, sweetie, I'm not a vet. But the lab has a biological analyser."
- Going back into the pantry: nothing useful, Molly loses the turn shaking among the flour sacks.
```

---

### `avventure/space-chicken/stanze/corridor.md`

```markdown
+++
id = "corridor"
nome = "The main corridor"
oggetti = []
oggetti_nascosti = []

[uscite.south]
verso = "mess_hall"
descrizione = "the mess hall door"

[uscite.west]
verso = "crew_quarters"
descrizione = "the passage to the crew quarters"

[uscite.north]
verso = "bridge"
descrizione = "the ladder climbing to the command bridge"

[uscite.east]
verso = "laboratory"
descrizione = "the armoured laboratory door, with a card reader blinking red"
richiede_oggetto = "captain_card"

[uscite.down]
verso = "cargo_hold"
descrizione = "a freight lift going down to the cargo hold and the cold rooms"
+++

## Description
A long corridor of grey metal, lit by red emergency lights that blink for no apparent reason. Puffs of steam come out of the walls at regular intervals, because that is how it's done on spaceships. You can hear little feet scurrying in the ducts above your head. To the south the mess hall, to the west the crew quarters, to the north the ladder to the bridge, to the east the armoured laboratory door, and a freight lift goes down to the cargo hold.

## Details
- The laboratory door reader displays: "RESTRICTED ACCESS — AUTHORISED PERSONNEL ONLY (AND THE CAPTAIN)".
- A trail of feathers leads to the freight lift… and stops abruptly, as if the creatures had changed their minds.
- Fresh graffiti on the wall, made with a claw: a drawing that vaguely looks like a fried chicken. Disturbing.

## Secrets
The trail of feathers that stops in front of the freight lift is the first clue: the creatures avoid the cargo hold (because of the smell of the sour mozzarella).

## Mechanics
- Without `captain_card` the laboratory door does not open (the engine blocks the move): AUNTIE suggests that the captain "always kept everything in his locker, together with his fishing magazines".
- Forcing the laboratory door: impossible ("it's armoured, sweetie, it cost us an arm and a leg").
- After turn 40, if Molly stays in the corridor for two turns in a row without `sour_mozzarella`, a creature drops from the ceiling: apply the ambush rule.
```

---

### `avventure/space-chicken/stanze/crew_quarters.md`

```markdown
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
```

---

### `avventure/space-chicken/stanze/air_ducts.md`

```markdown
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
```

---

### `avventure/space-chicken/stanze/laboratory.md`

```markdown
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
```

---

### `avventure/space-chicken/stanze/bridge.md`

```markdown
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
```

---

### `avventure/space-chicken/stanze/cargo_hold.md`

```markdown
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
```

---

### `avventure/space-chicken/stanze/engine_room.md`

```markdown
+++
id = "engine_room"
nome = "The engine room"
oggetti = ["wrench", "rubber_hose", "duct_tape"]
oggetti_nascosti = []

[uscite.east]
verso = "cargo_hold"
descrizione = "the watertight door back to the cargo hold"
+++

## Description
The noisy heart of the Rusty Badger: a humming reactor, pipes leaking steam, a workbench full of tools. It is your kingdom: here you spent years fixing the unfixable. On the bench are your giant wrench, a coiled rubber hose and a roll of duct tape. Behind the reactor, something big is moving.

## Details
- Behind the reactor, a creature as big as a turkey is pecking at an electric cable. It turns towards you. "PEEOK."
- The rubber hose has a fitting that would match a tank valve perfectly.
- On the bench, a sticker with your name: "MOLLY — TOUCH IT AND YOU'RE DEAD".

## Secrets
The engine room creature attacks anyone who comes in, unless it smells the sour mozzarella.

## Mechanics
- **Ambush on entry** (every time Molly comes in, until the creature has been chased away): apply the ambush rule. With `sour_mozzarella` in the inventory the creature flees into a duct and never comes back (record it with a room note).
- Once the creature has been chased away, the items on the bench can be taken freely.
- If Molly suffered the ambush and fled, the items stay here.
- Repairing the reactor or the ship: impossible, "it would take six months and a miracle".
```

---

### `avventure/space-chicken/stanze/hangar.md`

```markdown
+++
id = "hangar"
nome = "The shuttle hangar"
oggetti = []
oggetti_nascosti = []

[uscite.west]
verso = "cargo_hold"
descrizione = "the hatch back to the cargo hold"

[uscite.in]
verso = "shuttle"
descrizione = "the open hatch of the rescue shuttle \"Little Badger\""
+++

## Description
A narrow hangar lit by rotating orange lights. In the middle, the rescue shuttle **Little Badger**: small, stubby, with a fin painted with flames. The hatch is open. From the loudspeakers, AUNTIE counts down. On the floor, traces of fresh slime leading… inside the shuttle.

## Details
- The slime trails are huge: whatever left them must be twice your height.
- Next to the hatch, a sign: "CAPACITY: 1 ADULT + 1 PET. NO CHICKENS."

## Secrets
The biggest monster, now over three metres tall, is hiding in the shuttle.

## Mechanics
- If the player hesitates or wastes time, remind them of the countdown.
- Getting into the shuttle leads to the final showdown (see `shuttle`).
```

---

### `avventure/space-chicken/stanze/shuttle.md`

```markdown
+++
id = "shuttle"
nome = "The rescue shuttle"
oggetti = []
oggetti_nascosti = []

[uscite.out]
verso = "hangar"
descrizione = "the hatch to the hangar"
+++

## Description
The cockpit of the Little Badger: two seats with seatbelts, a panel with the big green "LAUNCH" button and, at the back, a red emergency hatch with a "QUICK RELEASE" lever. And then there is **him**: a creature more than three metres tall, hunched under the ceiling, with a tuft of feathers on its head and a beak full of teeth. It looks at you. It licks its beak. "…PEEOK."

## Details
- The monster is standing right in front of the emergency hatch.
- The seatbelts are sturdy, the four-point kind.
- The launch button has a safety cover and a warning light: "HATCHES CLOSED: NO".

## Secrets
- To win you have to: hold the monster off (sour mozzarella), freeze it (cryo cannon), fasten your seatbelt, open the emergency hatch (the frozen monster is sucked out into space), close it again and take off with Mr. Whiskers.

## Mechanics
- **Holding the monster off**: showing/throwing/waving the `sour_mozzarella` → the monster backs away in disgust against the hatch: `monster_held_off = true` (the mozzarella stays in the inventory if it is only shown, it is removed if thrown: in both cases the effect lasts until the end of the fight).
- Without `monster_held_off`, any action other than fleeing outside makes the monster attack: apply the ambush rule (at the second ambush suffered: defeat).
- **Freezing**: with `cryo_cannon` and `monster_held_off = true` → `monster_frozen = true` (+15): the monster becomes a chicken-shaped ice lolly. With only the `coolant_tank` (no hose) the jet goes everywhere and is not enough: no effect, turn lost.
- **Seatbelt**: fastening it sets `seatbelt_fastened = true`.
- **Emergency hatch**: opening it with `monster_frozen = true` and `seatbelt_fastened = true` → the monster is sucked out into space: `monster_ejected = true` (+15). Without the seatbelt → defeat (Molly flies away together with the ice lolly). With the monster not frozen → the monster clings on with its claws and stays inside: ambush.
- **Launch**: pressing "LAUNCH" requires `monster_ejected = true` (the hatch must be closed first: the AI closes it automatically in the narration). If Mr. Whiskers is not in the inventory, Molly stops: "I'm not leaving without Mr. Whiskers." and AUNTIE reminds her where the cat was (the player can go back for him if time allows). With Mr. Whiskers: `shuttle_launched = true` (+15) → victory (see gioco.md).
```

---

### `avventure/space-chicken/traduzioni/it.toml`

```toml
# Traduzione italiana dei testi che il motore mostra direttamente al giocatore.
# La narrazione la scrive l'AI in italiano; questi nomi le vengono passati
# perché li usi sempre uguali. Tutto ciò che manca resta in inglese.

titolo = "Star Trucker and the Space Chicken of Doom"
descrizione_breve = "Un cargo spaziale, uova aliene e polli mostruosi. Commedia sci-fi da B-movie con mozzarelle."
introduzione = """
Anno 2122. Il cargo spaziale **Rusty Badger** stava tornando sulla Terra con un carico di mozzarelle di bufale mizardiane quando AUNTIE, il computer di bordo, ha svegliato l'equipaggio per un segnale proveniente dall'asteroide Mold-13. Sull'asteroide c'era un relitto alieno pieno di uova giganti. Qualcuno ha avuto la brillante idea di portarne a bordo qualcuna.
Mentre il capitano proponeva una frittata e la responsabile scientifica insisteva per riportarle sulla Terra, le uova si sono schiuse. Decine di pulcini con i denti sono spariti nei condotti di aerazione. Da allora sono cresciuti. Molto. E hanno fame.

Tu sei **Molly**, la meccanica di bordo. Sei appena uscita dalla dispensa della mensa, dove ti eri nascosta. La nave è silenziosa. Troppo silenziosa. Sei l'unica sopravvissuta… a parte, forse, Mr. Whiskers, il gatto di bordo.
"""

[stanze]
mess_hall = "La mensa"
corridor = "Il corridoio principale"
crew_quarters = "Gli alloggi dell'equipaggio"
air_ducts = "I condotti di aerazione"
laboratory = "Il laboratorio scientifico"
bridge = "Il ponte di comando"
cargo_hold = "La stiva e le celle frigorifere"
engine_room = "La sala macchine"
hangar = "L'hangar della navetta"
shuttle = "La navetta di salvataggio"

[oggetti]
frying_pan = "una padella antiaderente da galea, ammaccata"
tuna_can = "una scatoletta di tonno spaziale al naturale"
eggshell = "un frammento di guscio d'uovo alieno coperto di bava verdastra"
cat_carrier = "un trasportino per gatti con la scritta «MR. WHISKERS — NON APRIRE»"
mr_whiskers = "Mr. Whiskers, il gatto rosso di bordo, molto offeso, nel trasportino"
captain_card = "la tessera magnetica del capitano Hal Brannigan"
captain_note = "un biglietto unto con scritto il codice di autodistruzione"
megacorp_dossier = "una stampa dei file riservati della MegaCorp Unlimited"
sour_mozzarella = "una mozzarella di bufala mizardiana inacidita, dall'odore devastante"
wrench = "una chiave inglese gigante"
rubber_hose = "un tubo di gomma flessibile"
duct_tape = "un rotolo di nastro adesivo americano (il vero eroe dello spazio)"
coolant_tank = "una bombola di refrigerante criogenico, gelida al tatto"
cryo_cannon = "un cannone criogenico improvvisato: bombola, tubo e tanto nastro adesivo"

[uscite]
"mess_hall.north" = "una porta scorrevole che si apre con un sospiro idraulico sul corridoio principale"
"corridor.south" = "la porta della mensa"
"corridor.west" = "il passaggio verso gli alloggi dell'equipaggio"
"corridor.north" = "la scaletta che sale al ponte di comando"
"corridor.east" = "la porta blindata del laboratorio, con un lettore di tessere che lampeggia di rosso"
"corridor.down" = "un montacarichi che scende alla stiva e alle celle frigorifere"
"crew_quarters.east" = "il passaggio verso il corridoio principale"
"crew_quarters.up" = "una grata di aerazione nel soffitto, sopra la cuccetta più alta"
"air_ducts.down" = "la grata aperta sopra gli alloggi"
"air_ducts.east" = "un tratto di condotto che finisce su una grata sopra il laboratorio"
"laboratory.west" = "la porta blindata verso il corridoio (dall'interno si apre con un pulsante)"
"laboratory.up" = "la grata di aerazione nel soffitto"
"bridge.south" = "la scaletta che scende al corridoio"
"cargo_hold.up" = "il montacarichi che risale al corridoio"
"cargo_hold.west" = "una porta stagna verso la sala macchine, da cui arriva un ronzio"
"cargo_hold.east" = "il portellone dell'hangar della navetta di salvataggio, con la scritta «SOLO EMERGENZE»"
"engine_room.east" = "la porta stagna che torna alla stiva"
"hangar.west" = "il portellone che torna alla stiva"
"hangar.in" = "il portello aperto della navetta di salvataggio «Little Badger»"
"shuttle.out" = "il portello verso l'hangar"
```

---

### `avventure/space-chicken/traduzioni/fr.toml`

```toml
# Traduzione francese dei testi che il motore mostra direttamente al giocatore.
# La narrazione la scrive l'AI in francese; questi nomi le vengono passati
# perché li usi sempre uguali. Tutto ciò che manca resta in inglese.

titolo = "Star Trucker and the Space Chicken of Doom"
descrizione_breve = "Un cargo spatial, des œufs extraterrestres et des poulets monstrueux. Comédie de science-fiction façon série B, avec mozzarellas."
introduzione = """
Année 2122. Le cargo spatial **Rusty Badger** rentrait sur Terre avec une cargaison de mozzarella de bufflonnes mizardiennes quand AUNTIE, l'ordinateur de bord, a réveillé l'équipage à cause d'un signal provenant de l'astéroïde Mold-13. Sur l'astéroïde se trouvait une épave extraterrestre pleine d'œufs géants. Quelqu'un a eu la brillante idée d'en monter quelques-uns à bord.
Pendant que le capitaine proposait une omelette et que la responsable scientifique insistait pour les rapporter sur Terre, les œufs ont éclos. Des dizaines de poussins à dents ont disparu dans les conduits d'aération. Depuis, ils ont grandi. Beaucoup. Et ils ont faim.

Vous êtes **Molly**, la mécanicienne de bord. Vous venez de sortir du garde-manger du mess, où vous vous étiez cachée. Le vaisseau est silencieux. Trop silencieux. Vous êtes la seule survivante… à part, peut-être, Mr. Whiskers, le chat du bord.
"""

[stanze]
mess_hall = "Le mess"
corridor = "Le couloir principal"
crew_quarters = "Les quartiers de l'équipage"
air_ducts = "Les conduits d'aération"
laboratory = "Le laboratoire scientifique"
bridge = "La passerelle de commandement"
cargo_hold = "La soute et les chambres froides"
engine_room = "La salle des machines"
hangar = "Le hangar de la navette"
shuttle = "La navette de sauvetage"

[oggetti]
frying_pan = "une poêle antiadhésive de cuisine de bord, cabossée"
tuna_can = "une boîte de thon spatial au naturel"
eggshell = "un fragment de coquille d'œuf extraterrestre couvert de bave verdâtre"
cat_carrier = "une caisse de transport pour chat marquée « MR. WHISKERS — NE PAS OUVRIR »"
mr_whiskers = "Mr. Whiskers, le chat roux du bord, très vexé, dans sa caisse de transport"
captain_card = "la carte magnétique du capitaine Hal Brannigan"
captain_note = "un billet graisseux où est écrit le code d'autodestruction"
megacorp_dossier = "une impression des dossiers confidentiels de MegaCorp Unlimited"
sour_mozzarella = "une mozzarella de bufflonne mizardienne tournée, à l'odeur dévastatrice"
wrench = "une clé à molette géante"
rubber_hose = "un tuyau en caoutchouc souple"
duct_tape = "un rouleau de ruban adhésif toilé (le vrai héros de l'espace)"
coolant_tank = "une bouteille de fluide cryogénique, glacée au toucher"
cryo_cannon = "un canon cryogénique improvisé : bouteille, tuyau et beaucoup de ruban adhésif"

[uscite]
"mess_hall.north" = "une porte coulissante qui s'ouvre avec un soupir hydraulique sur le couloir principal"
"corridor.south" = "la porte du mess"
"corridor.west" = "le passage vers les quartiers de l'équipage"
"corridor.north" = "l'échelle qui monte à la passerelle de commandement"
"corridor.east" = "la porte blindée du laboratoire, avec un lecteur de cartes qui clignote en rouge"
"corridor.down" = "un monte-charge qui descend à la soute et aux chambres froides"
"crew_quarters.east" = "le passage vers le couloir principal"
"crew_quarters.up" = "une grille d'aération au plafond, au-dessus de la couchette la plus haute"
"air_ducts.down" = "la grille ouverte au-dessus des quartiers"
"air_ducts.east" = "un tronçon de conduit qui aboutit à une grille au-dessus du laboratoire"
"laboratory.west" = "la porte blindée vers le couloir (de l'intérieur, elle s'ouvre avec un bouton)"
"laboratory.up" = "la grille d'aération au plafond"
"bridge.south" = "l'échelle qui descend au couloir"
"cargo_hold.up" = "le monte-charge qui remonte au couloir"
"cargo_hold.west" = "une porte étanche vers la salle des machines, d'où vient un bourdonnement"
"cargo_hold.east" = "le sas du hangar de la navette de sauvetage, marqué « URGENCES UNIQUEMENT »"
"engine_room.east" = "la porte étanche qui retourne à la soute"
"hangar.west" = "le sas qui retourne à la soute"
"hangar.in" = "l'écoutille ouverte de la navette de sauvetage « Little Badger »"
"shuttle.out" = "l'écoutille vers le hangar"
```

---

### `avventure/space-chicken/traduzioni/es.toml`

```toml
# Traduzione spagnola dei testi che il motore mostra direttamente al giocatore.
# La narrazione la scrive l'AI in spagnolo; questi nomi le vengono passati
# perché li usi sempre uguali. Tutto ciò che manca resta in inglese.

titolo = "Star Trucker and the Space Chicken of Doom"
descrizione_breve = "Un carguero espacial, huevos alienígenas y pollos monstruosos. Comedia de ciencia ficción de serie B, con mozzarellas."
introduzione = """
Año 2122. El carguero espacial **Rusty Badger** volvía a la Tierra con un cargamento de mozzarella de búfala mizardiana cuando AUNTIE, el ordenador de a bordo, despertó a la tripulación por una señal procedente del asteroide Mold-13. En el asteroide había un pecio alienígena lleno de huevos gigantes. A alguien se le ocurrió la brillante idea de subir unos cuantos a bordo.
Mientras el capitán proponía una tortilla y la oficial científica insistía en llevarlos a la Tierra, los huevos eclosionaron. Decenas de polluelos con dientes desaparecieron por los conductos de ventilación. Desde entonces han crecido. Mucho. Y tienen hambre.

Eres **Molly**, la mecánica de a bordo. Acabas de salir de la despensa del comedor, donde te habías escondido. La nave está en silencio. Demasiado en silencio. Eres la única superviviente… aparte, quizá, de Mr. Whiskers, el gato de a bordo.
"""

[stanze]
mess_hall = "El comedor"
corridor = "El pasillo principal"
crew_quarters = "Los camarotes de la tripulación"
air_ducts = "Los conductos de ventilación"
laboratory = "El laboratorio científico"
bridge = "El puente de mando"
cargo_hold = "La bodega y las cámaras frigoríficas"
engine_room = "La sala de máquinas"
hangar = "El hangar de la lanzadera"
shuttle = "La lanzadera de salvamento"

[oggetti]
frying_pan = "una sartén antiadherente de cocina de a bordo, abollada"
tuna_can = "una lata de atún espacial al natural"
eggshell = "un trozo de cáscara de huevo alienígena cubierto de baba verdosa"
cat_carrier = "un transportín para gatos con el rótulo «MR. WHISKERS — NO ABRIR»"
mr_whiskers = "Mr. Whiskers, el gato pelirrojo de a bordo, muy ofendido, dentro del transportín"
captain_card = "la tarjeta magnética del capitán Hal Brannigan"
captain_note = "una nota grasienta con el código de autodestrucción"
megacorp_dossier = "una copia impresa de los archivos confidenciales de MegaCorp Unlimited"
sour_mozzarella = "una mozzarella de búfala mizardiana agria, de olor devastador"
wrench = "una llave inglesa gigante"
rubber_hose = "una manguera de goma flexible"
duct_tape = "un rollo de cinta americana (la verdadera heroína del espacio)"
coolant_tank = "una bombona de refrigerante criogénico, helada al tacto"
cryo_cannon = "un cañón criogénico improvisado: bombona, manguera y mucha cinta americana"

[uscite]
"mess_hall.north" = "una puerta corredera que se abre con un suspiro hidráulico hacia el pasillo principal"
"corridor.south" = "la puerta del comedor"
"corridor.west" = "el paso hacia los camarotes de la tripulación"
"corridor.north" = "la escalerilla que sube al puente de mando"
"corridor.east" = "la puerta blindada del laboratorio, con un lector de tarjetas que parpadea en rojo"
"corridor.down" = "un montacargas que baja a la bodega y a las cámaras frigoríficas"
"crew_quarters.east" = "el paso hacia el pasillo principal"
"crew_quarters.up" = "una rejilla de ventilación en el techo, sobre la litera más alta"
"air_ducts.down" = "la rejilla abierta sobre los camarotes"
"air_ducts.east" = "un tramo de conducto que acaba en una rejilla sobre el laboratorio"
"laboratory.west" = "la puerta blindada hacia el pasillo (desde dentro se abre con un botón)"
"laboratory.up" = "la rejilla de ventilación del techo"
"bridge.south" = "la escalerilla que baja al pasillo"
"cargo_hold.up" = "el montacargas que sube al pasillo"
"cargo_hold.west" = "una puerta estanca hacia la sala de máquinas, de donde llega un zumbido"
"cargo_hold.east" = "la compuerta del hangar de la lanzadera de salvamento, con el rótulo «SOLO EMERGENCIAS»"
"engine_room.east" = "la puerta estanca que vuelve a la bodega"
"hangar.west" = "la compuerta que vuelve a la bodega"
"hangar.in" = "la escotilla abierta de la lanzadera de salvamento «Little Badger»"
"shuttle.out" = "la escotilla hacia el hangar"
```

---

### `avventure/space-chicken/traduzioni/de.toml`

```toml
# Traduzione tedesca dei testi che il motore mostra direttamente al giocatore.
# La narrazione la scrive l'AI in tedesco; questi nomi le vengono passati
# perché li usi sempre uguali. Tutto ciò che manca resta in inglese.

titolo = "Star Trucker and the Space Chicken of Doom"
descrizione_breve = "Ein Raumfrachter, außerirdische Eier und monströse Hühner. Science-Fiction-Komödie im B-Movie-Stil, mit Mozzarella."
introduzione = """
Im Jahr 2122. Der Raumfrachter **Rusty Badger** war mit einer Ladung Mozzarella von mizardianischen Büffeln auf dem Rückweg zur Erde, als AUNTIE, der Bordcomputer, die Besatzung wegen eines Signals vom Asteroiden Mold-13 weckte. Auf dem Asteroiden lag ein außerirdisches Wrack voller riesiger Eier. Jemand hatte die glänzende Idee, ein paar davon an Bord zu bringen.
Während der Kapitän ein Omelett vorschlug und die Wissenschaftsoffizierin darauf bestand, sie zur Erde zu bringen, sind die Eier geschlüpft. Dutzende Küken mit Zähnen verschwanden in den Lüftungsschächten. Seitdem sind sie gewachsen. Sehr. Und sie haben Hunger.

Du bist **Molly**, die Bordmechanikerin. Du bist gerade aus der Vorratskammer der Messe gekommen, in der du dich versteckt hattest. Das Schiff ist still. Zu still. Du bist die einzige Überlebende … abgesehen vielleicht von Mr. Whiskers, der Bordkatze.
"""

[stanze]
mess_hall = "Die Messe"
corridor = "Der Hauptkorridor"
crew_quarters = "Die Mannschaftsquartiere"
air_ducts = "Die Lüftungsschächte"
laboratory = "Das Wissenschaftslabor"
bridge = "Die Kommandobrücke"
cargo_hold = "Der Laderaum und die Kühlkammern"
engine_room = "Der Maschinenraum"
hangar = "Der Hangar der Rettungsfähre"
shuttle = "Die Rettungsfähre"

[oggetti]
frying_pan = "eine verbeulte antihaftbeschichtete Kombüsenpfanne"
tuna_can = "eine Dose Weltraum-Thunfisch im eigenen Saft"
eggshell = "ein Stück außerirdischer Eierschale voller grünlichem Schleim"
cat_carrier = "eine Katzentransportbox mit der Aufschrift „MR. WHISKERS — NICHT ÖFFNEN“"
mr_whiskers = "Mr. Whiskers, der rote Bordkater, sehr beleidigt, in der Transportbox"
captain_card = "die Magnetkarte von Kapitän Hal Brannigan"
captain_note = "ein fettiger Zettel mit dem Selbstzerstörungscode"
megacorp_dossier = "ein Ausdruck der vertraulichen Akten der MegaCorp Unlimited"
sour_mozzarella = "ein sauer gewordener Mozzarella vom mizardianischen Büffel mit verheerendem Geruch"
wrench = "ein riesiger Schraubenschlüssel"
rubber_hose = "ein biegsamer Gummischlauch"
duct_tape = "eine Rolle Panzerband (der wahre Held des Weltraums)"
coolant_tank = "eine Flasche Kryo-Kühlmittel, eiskalt beim Anfassen"
cryo_cannon = "eine improvisierte Kryokanone: Flasche, Schlauch und jede Menge Panzerband"

[uscite]
"mess_hall.north" = "eine Schiebetür, die sich mit einem hydraulischen Seufzer zum Hauptkorridor öffnet"
"corridor.south" = "die Tür der Messe"
"corridor.west" = "der Durchgang zu den Mannschaftsquartieren"
"corridor.north" = "die Leiter, die zur Kommandobrücke hinaufführt"
"corridor.east" = "die Panzertür des Labors mit einem rot blinkenden Kartenleser"
"corridor.down" = "ein Lastenaufzug, der zum Laderaum und zu den Kühlkammern hinunterfährt"
"crew_quarters.east" = "der Durchgang zum Hauptkorridor"
"crew_quarters.up" = "ein Lüftungsgitter in der Decke, über der obersten Koje"
"air_ducts.down" = "das offene Gitter über den Quartieren"
"air_ducts.east" = "ein Schachtabschnitt, der an einem Gitter über dem Labor endet"
"laboratory.west" = "die Panzertür zum Korridor (von innen öffnet sie sich per Knopfdruck)"
"laboratory.up" = "das Lüftungsgitter in der Decke"
"bridge.south" = "die Leiter, die zum Korridor hinunterführt"
"cargo_hold.up" = "der Lastenaufzug, der zum Korridor hinauffährt"
"cargo_hold.west" = "eine Schottentür zum Maschinenraum, aus dem ein Summen dringt"
"cargo_hold.east" = "das Schott des Hangars der Rettungsfähre mit der Aufschrift „NUR FÜR NOTFÄLLE“"
"engine_room.east" = "die Schottentür zurück zum Laderaum"
"hangar.west" = "das Schott zurück zum Laderaum"
"hangar.in" = "die offene Luke der Rettungsfähre „Little Badger“"
"shuttle.out" = "die Luke zum Hangar"
```

---

## Avventura 3 — "Vinny Falco: The Devil's Slide Affair" (in italiano "Vinny Falco: il caso di Devil's Slide")

Noir hard-boiled: San Francisco, ottobre–novembre 1928. Il detective privato Vinny Falco indaga per il colonnello Granger sul figlio Julius, poi sulla vedova francese del figlio morto in guerra e sul testimone delle nozze, fino al finale al Montgomery Block (due finali vincenti: Julius salvato o finale amaro). Ispirata ai personaggi e alla trama del racconto *Vinny Falco Detective* di Peter A. Ray (Studio Gronk!), disponibile gratuitamente su https://gronk.it; i testi dell'avventura sono originali.

### Struttura

```
avventure/vinny-falco/
├── gioco.md
├── stanze/
│   ├── office.md
│   ├── market_street.md
│   ├── roadster.md
│   ├── granger_villa.md
│   ├── cliff_house.md
│   ├── manson_office.md
│   ├── diner.md
│   ├── chinatown.md
│   ├── tenderloin.md
│   ├── montgomery_block.md
│   └── julius_apartment.md
└── traduzioni/
    ├── it.toml
    ├── fr.toml
    ├── es.toml
    └── de.toml
```

### Mappa (per riferimento, non è un file)

```
 [office]
    | down
 [market_street] --east--> [manson_office]
    |  |   \ in
    |  |    [roadster] --south (case_accepted)--> [granger_villa]
    |  |               --west (dinner_invited)--> [cliff_house]
    |  | west
    |  [tenderloin] <-south- [chinatown]
    | north                     | east
 [diner] -------west----------->+
    | in
 [montgomery_block] --east (julius_door_open)--> [julius_apartment]
```

---

### `avventure/vinny-falco/gioco.md`

```markdown
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
```

---

### `avventure/vinny-falco/stanze/office.md`

```markdown
+++
id = "office"
nome = "The office of Falco & Associates"
oggetti = []
oggetti_nascosti = ["lockpicks"]

[uscite.down]
verso = "market_street"
descrizione = "the creaking stairs down to Market Street"
+++

## Description
A small office on the third floor of a building that has seen better days. Gilt letters on the frosted glass of the door: FALCO & ASSOCIATES. A scratched desk with a black telephone, an overflowing ashtray and a racing paper; a coat stand holding your one and only jacket; a filing cabinet; the valve radio playing jazz. Through the window, the roofs of Market Street and a slice of grey sky. The stairs lead down to the street.

## Details
- The desk has three drawers; the bottom one sticks and has to be pulled hard.
- The filing cabinet contains old cases, unpaid bills and a bottle of rye that officially does not exist.
- The radio: King Oliver, then the race results.

## Secrets
- In the bottom drawer, under some bills, is a roll of `lockpicks`: a souvenir of a burglar who paid his fee in kind.

## Mechanics
- **The phone call** (start of the game): the telephone is ringing. If Vinny answers, it is lawyer Manson ("Good afternoon, Vincenzo"): a wealthy client, Colonel Elias Granger, wants to know why his son Julius asks for more and more money. Little effort, a lot of money. Accepting sets `case_accepted = true` and Manson gives directions to the villa at Devil's Slide, south along the coast road. If the player ignores the phone, it keeps ringing stubbornly until Vinny answers.
- Opening the bottom drawer (forcing it): make `lockpicks` visible.
- **Jeanne Roche's visit**: the first time Vinny comes back to the office with `julius_report_done = true` and `jeanne_visited = false`, it is a late-October afternoon of fog; three sharp knocks, and in comes a stunning woman: dark red hair under a wide-brimmed hat, black skirt, white blouse, a man-cut jacket, a violet perfume. "Jeanne Roche. You have probably heard of me." She is unmistakably the nurse in the photograph at the colonel's villa, only more mature and sophisticated. She plays with an unlit cigarette like an actress waiting for her cue. She receives anonymous letters accusing her of an "indecent life" after her husband's death; she is engaged to Miles Harrington and if the letters reached him it would be the end. She wants Vinny to find the author and the reason. Something does not add up: no demands, no money, only noise. Whatever Vinny answers, she does not insist; leaving, she leaves one letter on the desk ("in case you change your mind"). Set `jeanne_visited = true` and add `anonymous_letter` to the office items. Shortly after, a messenger boy brings a note from Manson: add `manson_invitation` to the inventory and set `dinner_invited = true`.
- **Examining the anonymous letter**: typewritten, vague and vulgar accusations, no signature, no demands. Held close, the paper gives off a faint scent of violets, the same perfume Jeanne wore. Set `violets_noticed = true` (+5). Vinny can wonder aloud why anyone would write to her and perfume the letter like that, but he does not draw the conclusion for the player.
- **The telegram**: if `telegram_sent = true` and at least 3 turns have passed since `telegram_sent_turn`, when Vinny is in the office the telegram has arrived under the door: add `telegram` to the office items (only once).
- **Reading the telegram**: set `telegram_read = true` (+10). Text, in telegraphic style: "HARRINGTON MILES SAILED NEW YORK LE HAVRE FEBRUARY 1927 ON SS PARIS STOP TRAVELLED WITH MRS ADA COSTELLO STOP RETURNED NEW YORK SAME SHIP AS MRS JEANNE ROCHE STOP NO ADA COSTELLO ON ANY RETURN LIST STOP". Vinny stares at the name for a long time: it rings a bell. Do not say where he read it: the player must remember (or go back to the Montgomery Block).
- Calling someone on the phone: Manson answers (he refers to what is in his files); the villa's butler answers for the Grangers. Nobody else is useful.
```

---

### `avventure/vinny-falco/stanze/market_street.md`

```markdown
+++
id = "market_street"
nome = "Market Street"
oggetti = []
oggetti_nascosti = []

[uscite.up]
verso = "office"
descrizione = "the door of your building and the stairs up to your office"

[uscite.in]
verso = "roadster"
descrizione = "your Ford Roadster, parked at the kerb"

[uscite.north]
verso = "diner"
descrizione = "Montgomery Street, towards the diner opposite the Montgomery Block"

[uscite.east]
verso = "manson_office"
descrizione = "the granite building that houses lawyer Manson's practice"

[uscite.west]
verso = "tenderloin"
descrizione = "Turk Street, towards the Tenderloin"
+++

## Description
The city's main artery: streetcars clanging, horns, newsboys shouting the headlines, office workers with crooked ties, a cop directing traffic as if conducting an orchestra of madmen. Your Ford Roadster waits at the kerb. Up there, the window of your office. To the north, Montgomery Street; to the east, the granite building of Manson's practice; to the west, Turk Street descends towards the Tenderloin.

## Details
- The newsboys shout about the presidential race: Hoover against Al Smith.
- A shoeshine boy offers his services and the latest gossip, all for a nickel.
- In front of a drugstore, a sign: "SODA FOUNTAIN — NO ALCOHOL SERVED" (wink included).

## Mechanics
- The shoeshine boy knows the streets: if asked, he says that the Tenderloin is where you go "for things you can't buy at the drugstore" and that the cops there "look the other way, for a fee".
- After `finale_started = true` there is nothing new here: Vinny feels that the game is being played at the Montgomery Block.
```

---

### `avventure/vinny-falco/stanze/roadster.md`

```markdown
+++
id = "roadster"
nome = "Your Ford Roadster"
oggetti = []
oggetti_nascosti = []

[uscite.out]
verso = "market_street"
descrizione = "the kerb of Market Street"

[uscite.south]
verso = "granger_villa"
descrizione = "the coast road south, towards Devil's Slide and the Granger villa"
richiede_flag = "case_accepted"

[uscite.west]
verso = "cliff_house"
descrizione = "the road west across the city to the ocean and the Cliff House"
richiede_flag = "dinner_invited"
+++

## Description
A black Ford Roadster with a dented mudguard and a hood that smells of oil and freedom. The seat creaks like an old dog. From here you can drive south along the coast road towards Devil's Slide, or west across the city to the ocean, or get out onto Market Street.

## Details
- In the glove box: a city map, a pack of Lucky Strikes and a parking ticket you have decided to forget.
- The coast road south runs high between bare hills and cliffs, with the ocean appearing and disappearing beyond the rocks.

## Mechanics
- Driving takes time: narrate the journey briefly (traffic, the salty wind, the light of the day) when Vinny leaves for the villa or the Cliff House.
- Without `case_accepted` Vinny has no reason to go to Devil's Slide; without `dinner_invited` nobody expects him at the Cliff House (the engine blocks those exits).
```

---

### `avventure/vinny-falco/stanze/granger_villa.md`

```markdown
+++
id = "granger_villa"
nome = "The Granger villa at Devil's Slide"
oggetti = []
oggetti_nascosti = []

[uscite.out]
verso = "roadster"
descrizione = "the gravel drive where your Ford is parked"
+++

## Description
A modern villa perched on the cliff, built more to keep intruders out than to welcome guests. A faceless butler leads you down a corridor that smells of wax and expensive bric-a-brac to a drawing room that is too big: Chinese vases, bronze statues with empty eyes, a Buddha smiling for no reason, a wide window open on the ocean. In the middle, in a wheelchair, sits **Colonel Elias Granger**. Behind him, her hands on the back of the chair, his sister **Maria**. On a cabinet, old photographs. Outside, your Ford waits on the gravel.

## Details
- The photographs: children playing on a beach; two young men posing in front of a biplane; a painting of an elegant young woman with a newborn in her arms (Désirée, the wife); and a photo of a young aviator, Anthony, next to a young nurse with the field plane in the background. "That's them," says the colonel. "Miles took it, in those days."
- Looked at carefully, the aviator and the nurse stand side by side but a little apart: polite, not tender. There is no wedding photo, although Miles photographed everything.
- The colonel speaks in a hoarse voice, as if every word cost him effort, but he has not lost his taste for command. Maria says little; her eyes fill with tears when Anthony is mentioned.

## Secrets
- The nurse in the photo is Adèle Morel, not the real Jeanne Roche: the woman who calls herself Jeanne looks like her because she *is* her.

## Mechanics
- **First visit** (`colonel_met = false`): the colonel tells the story (Désirée dead in childbirth, Anthony the hero shot down in France, Julius the would-be poet who asks for too much money). He gives Julius's address: **628 Montgomery Street, the Montgomery Block, first floor** (the diner opposite is a perfect observation point) and, from the butler, a useless faded photo of Julius as a boy. As Vinny is leaving, the colonel says that he and his sister have almost nobody left. "Almost?" Then he tells about **Jeanne Roche**: a year ago a Frenchwoman claimed to be Anthony's widow, married in 1918 at the field hospital, witnesses Miles Harrington (a reporter from a good Baltimore family, who confirmed everything) and a nurse colleague who could not be traced; the mayor who married them died before he could be questioned. Papers in order, Anthony's signature authentic. On Manson's advice the colonel made a new will that excludes her, but gave her a generous annuity. Set `colonel_met = true` (+5).
- **Report** (`heroin_found = true` and `julius_report_done = false`): lawyer Manson is there too. When Vinny reports the heroin, Maria turns pale and the colonel clenches his fists. Manson proposes a discreet clinic on Lake Geneva; until then Julius will stay locked up in the villa with a doctor and a nurse, Christmas with the family, departure in the new year. "And if he refuses?" "The colonel knows people who can persuade quietly." Vinny's job is done and the fee is generous. Set `julius_report_done = true` (+10). If Vinny reports without having found proof (only the den), the colonel wants certainty: "Bring me proof, Falco."
- **Later visits** (after the report): Julius is at the villa, pale, sedated and surly, under the eye of a nurse. Miss Maria speaks fondly of Miles Harrington, "such a dear boy", who visits almost every day and sometimes takes Julius to town to fetch clothes or books. If Vinny meets Miles here (only after `cliff_dinner_done = true`): polite, a slight limp, evasive charm; about France he says only that "the war is something one prefers to forget".
- Accusing someone in front of the Grangers without proof: the colonel has him shown out (no change of state).
```

---

### `avventure/vinny-falco/stanze/cliff_house.md`

```markdown
+++
id = "cliff_house"
nome = "The Cliff House restaurant"
oggetti = []
oggetti_nascosti = ["miles_folder"]

[uscite.out]
verso = "roadster"
descrizione = "the car park where your Ford is waiting"
+++

## Description
The Cliff House stands on the rocks at the end of the city, its lit windows piercing the dark like golden eyes. Inside, white tablecloths, silent waiters and the noise of the surf breaking down below. At a table by the window, his back as straight as a ruler, **lawyer Manson** is waiting for you. Outside, your Ford waits in the car park.

## Details
- The menu: fish in every sauce, from halibut to Dungeness crab. Oysters on ice.
- No bourbon on the table: with Manson, the Eighteenth Amendment is respected.
- The wind from the ocean lashes the windows.

## Secrets
- Manson has received a remorseful letter from a Baltimore colleague: the Harringtons are ruined (oil land as dry as the desert). Only the name and appearances remain.

## Mechanics
- **The dinner** (`cliff_dinner_done = false`): Manson tells how Julius is doing (docile, locked in the villa, a few trips to town escorted by the chauffeur or by Miles Harrington, now a regular visitor and Maria's favourite: he says it without looking up). Then he gets to the point: the letter from Baltimore, the ruined family, the engagement with Jeanne. He has a feeling that the first investigation was "only the first layer of a much darker picture" and he wants Vinny to find out who Miles really is. He hands over the folder: make `miles_folder` visible, Vinny can take it. Set `cliff_dinner_done = true` (+5). If Vinny mentions Jeanne's visit, Manson raises an eyebrow: "Interesting. She did not tell me."
- **Reading the folder on Miles**: a couple of years older than Anthony; studied law like his father, but became a war correspondent; at the front in France from 1914 for a small Baltimore paper; wounded by shrapnel, met Jeanne and Anthony in the field hospital and was a witness at their wedding; went home before the end of the war; learned of Anthony's death from Jeanne's letters, which soon stopped. No news for almost ten years, until Manson contacted him. He now lives modestly in San Francisco on a few jobs for Baltimore investors and a legacy from an aunt. Tailing him gives nothing: no vices, few outings, and, oddly, he rarely sees his fiancée. Vinny feels he is at a dead end: the answers are in the old investigation (Manson's practice).
- After the dinner there is nothing else to do here.
```

---

### `avventure/vinny-falco/stanze/manson_office.md`

```markdown
+++
id = "manson_office"
nome = "Lawyer Manson's practice"
oggetti = []
oggetti_nascosti = ["case_binder"]

[uscite.west]
verso = "market_street"
descrizione = "the lift and the lobby that lead back to Market Street"
+++

## Description
Tenth floor of a granite building. A secretary with a severe bun, a waiting room that smells of leather and ink, and beyond a frosted door the office of **Arthur Manson**: dark wood, shelves of law books, files labelled with maniacal precision. The lawyer sits at his desk bent over a thick volume. The lift goes back down to Market Street.

## Details
- On a shelf next to the desk, among the labelled files, a thick binder tied with a red band.
- Manson adjusts his glasses on his nose before every important sentence.

## Mechanics
- Before `cliff_dinner_done = true`: Manson receives Vinny politely but has nothing to add; if Vinny asks about Jeanne Roche, he says only that "the matter has been settled, with every guarantee".
- **The old investigation** (after `cliff_dinner_done = true`): if Vinny asks for the complete documentation, Manson points to the red-banded binder: make `case_binder` visible.
- **Reading the binder** (carefully, it takes hours): set `binder_read = true` (+5). The key facts: the bride's witness, **Adèle Morel**, a friend and fellow nurse of Jeanne, vanished after the war in an area where the Spanish flu killed more than the war; no relatives, no trace. The mayor of **Valduremont**, who celebrated the wedding, went fishing at dawn one morning in **March 1927** and was found drowned days later near the lock, with a suspicious bruise on his head: filed as an accident. Jeanne arrived in New York with an American passport issued by the consulate as the widow of a citizen, so she declared nothing to immigration; two weeks later she was in San Francisco, renting a small flat in a decent working-class district and waiting tables, clearly with money of her own. Why did she wait ten years?
- **The will** (asking Manson): set `will_known = true` (+5). The villa and safe bonds go to Julius, who can only live on the interest; Maria has an annuity and the right to live in the villa; Jeanne's annuity is paid from those bonds. The immense fortune goes to a charitable trust in memory of the wife and son (hospitals for women in childbirth, war widows). If Julius died before his father without children, Jeanne could claim the bonds and, as Anthony's widow, perhaps even the presidency of the trust: "An immense power, Vincenzo." And the colonel does not have long to live.
- **Jeanne's English**: if asked, Manson says that at first her English was hesitant, "or at least she made it seem so". **Miles**: when he was contacted in Baltimore, he already planned to come to San Francisco on business. "A nice coincidence, indeed."
- **The telegram** (after `binder_read = true`): if Vinny proposes to telegraph the New York agency that did the first investigation, to check whether Harrington has recently travelled to Europe (passenger lists), Manson thinks for a moment, drumming his fingers, then: "Very well, Falco." The secretary sends it at once. Set `telegram_sent = true` and `telegram_sent_turn` = current turn. The answer will be delivered to Vinny's office. If Vinny proposes it before reading the binder, Manson asks him to study the papers first.
```

---

### `avventure/vinny-falco/stanze/diner.md`

```markdown
+++
id = "diner"
nome = "The diner on Montgomery Street"
oggetti = []
oggetti_nascosti = []

[uscite.south]
verso = "market_street"
descrizione = "Montgomery Street, back down towards Market Street"

[uscite.in]
verso = "montgomery_block"
descrizione = "across the street, the arched entrance of the Montgomery Block"

[uscite.west]
verso = "chinatown"
descrizione = "the streets that climb west into Chinatown"
+++

## Description
A diner with a wide, clean window: perfect for watching both the entrance of the **Montgomery Block** across the street and the first-floor windows. Inside, the air smells of scrambled eggs, old grease, coffee and crumpled newspapers. A tired waitress with her hair pinned up any old way, two old men keeping each other company in silence, a corner table with a strategic view. Across the street, the Montgomery Block; to the west, Chinatown; to the south, Market Street.

## Details
- The house speciality is the beef pot pie: hot, fragrant, better than expected. It holds the stomach and patience together.
- The waitress takes orders without looking at you. The coffee is black, scalding and not the worst in San Francisco.
- From the corner table you can see the first-floor window of Julius Granger's flat.

## Mechanics
- **Stakeout** (after `colonel_met = true`, while `julius_report_done = false`): if Vinny waits and watches the entrance, the morning passes slowly, then Julius comes out: tall, thin, long brown hair, hands in his pockets, heading west towards Chinatown with the step of someone who knows exactly where he is going. Set `julius_seen = true`. If Vinny follows him (going west), set `following_julius = true`.
- **The trip to Angel Island** (after `den_found = true`, while `heroin_found = false`): on another morning of stakeout, Julius comes out early with a rucksack and a camera, meets a friend of his set and walks towards the Ferry Building, to the ferry for Angel Island: he will be away all day. Set `julius_on_ferry = true`. The opportunity is obvious: his flat is empty. If Vinny follows them instead, the day is wasted (set `julius_on_ferry = false`).
- **Start of the finale** (`telegram_read = true` and `finale_started = false`): a damp November afternoon. On the other side of the street, hat pulled down and hands in a trench coat too big for him, stands **Jimmy Delaney**, a young private eye. If Vinny mentions Jeanne Roche, Jimmy stiffens, then gives in: she put on the same act for him, and he found the little man who delivers the anonymous letters to her home, a petty crook known to half the police stations of the city, who always comes by the Montgomery Block before each delivery. While they talk, **Miles Harrington** enters the Montgomery Block: he knows exactly where to go. Behind the curtains of Julius's window, something moves. Set `finale_started = true`, `finale_turn` = current turn and `julius_door_open = true`. Vinny feels that the curtain is about to fall.
- **The clock of the finale**: if Vinny dawdles (orders, chats, waits) or moves away from the Montgomery Block, time passes. At the end of the turn in which `turno` exceeds `finale_turn` + 3 with `julius_saved = false` and `shots_fired = false`, three dry shots echo from inside the Montgomery Block: set `shots_fired = true`. If Vinny is far away from Montgomery Street at that moment (not in the diner, the Montgomery Block or Julius's flat), he learns of the shooting from the evening papers: the police have already closed the case as a shoot-out caused by the little man, and "Jeanne" is untouchable → `esito = "sconfitta"`.
- The first time Vinny tries to move away from the Montgomery Block after `finale_started = true`, his instinct warns him ("not now, Vinny"); if the player insists, let him go.
- Jimmy follows Vinny without asking questions once the finale has started.
```

---

### `avventure/vinny-falco/stanze/chinatown.md`

```markdown
+++
id = "chinatown"
nome = "Chinatown and Madame Sybilla's parlour"
oggetti = []
oggetti_nascosti = []

[uscite.east]
verso = "diner"
descrizione = "the street that goes back down to the diner on Montgomery Street"

[uscite.south]
verso = "tenderloin"
descrizione = "Turk Street, descending towards the Tenderloin"
+++

## Description
Red lanterns hang over the streets like tired eyes. Steaming laundries, grocers with dried fish hanging in the doorways, the clatter of mah-jong tiles from a first floor. Between a herbalist and a tailor, a bead curtain and a hand-painted sign: **MADAME SYBILLA — THE FUTURE IN THE TEA LEAVES**. To the east, the street goes back down towards the diner; to the south, Turk Street descends towards the Tenderloin.

## Details
- Madame Sybilla: an indefinite age, a turban, rings on every finger, a voice of smoke. She reads the tea leaves and the dreams of her clients, for a dollar (Vinny can pay; it changes nothing in the state).
- The parlour smells of incense and jasmine tea. A cat sleeps on a crystal ball.

## Mechanics
- **Following Julius** (`following_julius = true`): Julius crosses Chinatown without slowing down and turns onto Turk Street, towards the Tenderloin. If Vinny goes south straight away he keeps him in sight; if he stops (Sybilla, shopping, chatting) he loses him: set `following_julius = false` (he can try again another morning from the diner).
- **Madame Sybilla's prophecies** (the game's hint system): when Vinny has his future read, she gives **one** cryptic but useful hint, theatrical and ironic, based on the first missing step in this order:
  1. `case_accepted` false: "A bell rings in your empty office… someone wants to pay you. Answer, cara."
  2. `colonel_met` false: "An old soldier waits where the land ends and the devil slides into the sea."
  3. `den_found` false: "Watch the boy's door from where the coffee is hot, and follow his feet: they go where the sun never stops."
  4. `heroin_found` false: "The leaves show a boat… and an empty room. A key that is not a key, in your bottom drawer."
  5. `julius_report_done` false: "The father is waiting for the truth, even if it hurts."
  6. `cliff_dinner_done` false: "A woman with violets came to you. Then a man will invite you to dinner over the ocean."
  7. `binder_read` false: "The answers sleep in a red ribbon, in the lawyer's house."
  8. `telegram_read` false: "A ship crossed the ocean twice. Ask the wire who was on board."
  9. `identity_deduced` false: "Two names, one perfume. The brass plates do not lie, Vinny."
  10. otherwise: "Run, Vinny. The last act is played behind a door you have already knocked on."
  Never repeat the same hint twice in a row with the same words.
```

---

### `avventure/vinny-falco/stanze/tenderloin.md`

```markdown
+++
id = "tenderloin"
nome = "An alley in the Tenderloin"
oggetti = []
oggetti_nascosti = []

[uscite.north]
verso = "chinatown"
descrizione = "Turk Street, climbing back up towards Chinatown"

[uscite.east]
verso = "market_street"
descrizione = "the street that leads back to Market Street"
+++

## Description
The rotten heart of the city, where the sun never stops for long and the police, when they are there, just watch. An alley that smells of urine, mould and despair. A boarding house with a faded sign and windows boarded up crookedly, wedged between a brothel that does not bother to hide and a Chinese laundry with steamed-up windows. Steam rises from a grating in the pavement. On the corner, leaning on a lamp post, a cop as big as an ox. To the north, Turk Street climbs back towards Chinatown; to the east, a street leads back to Market Street.

## Details
- In ten minutes at least six people go in and out of the boarding house: the same look, dull, nervous, hungry. One clutches a package wrapped in brown paper.
- The cop on the corner: shiny uniform, hands in his pockets. He does not patrol and protects nobody: he looks more like a lookout.
- Inside the boarding house: a sweetish smoke that sticks to your clothes, and a thin Chinese clerk with apathetic eyes behind the counter.

## Mechanics
- **Following Julius** (`following_julius = true`): Julius walks into the boarding house. When he comes out, a little later, he goes back towards town. If Vinny goes in pretending to look for a room (or watches who goes in and out and then goes in), the clerk says there are no rooms, pretends not to understand, and his look says: you are not welcome. It is a narcotics den, and the boy is in it up to his neck. Set `den_found = true` (+5) and `following_julius = false`.
- Without `following_julius` (and with `den_found` false), the boarding house is just one of many squalid places: Vinny has no reason to connect it to Julius.
- Making trouble (threatening the clerk, drawing the revolver): the ox-like cop intervenes and Vinny ends up thrown out into the street with a bruise and a warning (turn lost, no other change).
```

---

### `avventure/vinny-falco/stanze/montgomery_block.md`

```markdown
+++
id = "montgomery_block"
nome = "The first floor of the Montgomery Block"
oggetti = []
oggetti_nascosti = []

[uscite.out]
verso = "diner"
descrizione = "the stairs down to Montgomery Street, opposite the diner"

[uscite.east]
verso = "julius_apartment"
descrizione = "the door with the brass plate JULIUS GRANGER"
richiede_flag = "julius_door_open"
+++

## Description
The Montgomery Block: thick walls, silent corridors and that air of discretion that suits people who do not want too many questions. On the first floor, a maze of corridors and doors with brass name plates. You read them as you walk: *1A — R. Whitcombe, Bookbinder*; *1B — Ada Costello*; *1C — Julius Granger*; *1D — The Golden Gate Poetry Circle*. A lift with a bored lift boy, and a caretaker who sweeps the same spot for half an hour. The door of 1C is to the east; the stairs go down to the street, opposite the diner.

## Details
- The door of 1B has a heavy, brand-new Yale lock, too good for a cheap flat and too good for your lock picks.
- The lift boy and the caretaker gossip for a quarter.

## Secrets
- Ada Costello, in 1B, is the woman who calls herself Jeanne Roche.

## Mechanics
- **Knocking at Julius's door** while he is at home (`julius_on_ferry = false` and `julius_report_done = false`): the door opens just enough. A tall, thin young man, uncombed, in purple pyjamas and a dressing gown with green parrots, eyes that have not yet decided whether to wake up. Vinny must improvise an excuse (the name on a nearby plate will do: "I'm looking for Ada Costello…" — "Can't you read? It's further along"). Through the gap he glimpses dishes, papers, clothes on a chair. Set `julius_seen = true`.
- **Picking the lock of 1C**: possible only with `lockpicks` in the inventory and only when the flat is empty (`julius_on_ferry = true`): the lock gives way easily, set `julius_door_open = true`. With Julius at home it would be madness (Vinny gives up). Without the picks the lock resists.
- **Gossip about 1B** (lift boy or caretaker): Mrs. Costello? Hardly ever seen, comes home late at night, a singer in some club, they say. A widow, or separated. Elegant, with a hat that hides her face. "And she always leaves the corridor smelling of violets, mister." Set `violets_linked = true`. If they are asked about Julius: a quiet tenant, out at strange hours.
- **Knocking at 1B before the finale**: nobody answers. Picking it: impossible (Yale lock).
- **The finale, late** (`shots_fired = true`, `ada_unmasked = false`): curious and frightened faces peer out of the doorways and point to 1C. A police inspector arrives with two officers and the reporters, and Jimmy (if he has come) bars the door to the crowd. The door of 1B is closed. If Vinny knocks at 1B and calls the woman by her name or by the name she has stolen ("Jeanne Roche!", "Ada Costello!", "Adèle Morel!", "open up, I know you're in there"), and possibly explains to the inspector the swindle and the staged scene: a second of silence, then a single dry shot behind the door. The officers break it down: the woman lies on the floor, the pistol still in her hand. Narrate it without gory details, with a bitter noir line. Set `ada_unmasked = true` (+15) and, if it was not already true, `identity_deduced = true` (+15): victory (bitter ending).
- If in the late finale Vinny confirms the staged version to the inspector or leaves without naming the woman in 1B: see the defeat conditions.
```

---

### `avventure/vinny-falco/stanze/julius_apartment.md`

```markdown
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
```

---

### `avventure/vinny-falco/traduzioni/it.toml`

```toml
# Traduzione italiana dei testi che il motore mostra direttamente al giocatore.
# La narrazione la scrive l'AI in italiano; questi nomi le vengono passati
# perché li usi sempre uguali. Tutto ciò che manca resta in inglese.

titolo = "Vinny Falco: il caso di Devil's Slide"
descrizione_breve = "San Francisco, 1928. Un ricco colonnello, un figlio nei guai, una vedova francese dalla storia troppo perfetta. Un noir hard-boiled."
introduzione = """
San Francisco, ottobre 1928. Il proibizionismo è la legge, il bourbon di contrabbando è l'abitudine, e Herbert Hoover promette un pollo in ogni pentola. Tu sei **Vinny Falco**, detective privato della *Falco e Associati*. «Associati» è solo una trovata pubblicitaria: l'unico socio sei tu, ma la scritta sulla porta dà un tocco di rispettabilità. L'intonaco scrostato del pianerottolo racconta tutta un'altra storia.

È un pomeriggio pigro. La cornetta di King Oliver si lamenta dalla tua nuova radio a valvole, regalo di una giornata fortunata alle corse, e tu sonnecchi con i piedi sulla scrivania. Poi il telefono squilla.
"""

[stanze]
office = "L'ufficio della Falco e Associati"
market_street = "Market Street"
roadster = "La tua Ford Roadster"
granger_villa = "La villa dei Granger a Devil's Slide"
cliff_house = "Il ristorante della Cliff House"
manson_office = "Lo studio dell'avvocato Manson"
diner = "La tavola calda di Montgomery Street"
chinatown = "Chinatown e il salottino di Madame Sybilla"
tenderloin = "Un vicolo del Tenderloin"
montgomery_block = "Il primo piano del Montgomery Block"
julius_apartment = "L'appartamento di Julius Granger"

[oggetti]
revolver = "il tuo revolver Colt Police Positive calibro 38"
lockpicks = "un rotolo di grimaldelli avvolto in pelle scamosciata"
heroin_kit = "una scatola di metallo con siringhe, cucchiaini anneriti e una bustina di polvere biancastra"
anonymous_letter = "una lettera anonima dattiloscritta, senza firma"
manson_invitation = "un biglietto dell'avvocato Manson su carta intestata: «Cena alla Cliff House, ore sette. Dobbiamo parlare.»"
miles_folder = "una cartelletta color senape su Miles Harrington"
case_binder = "un corposo raccoglitore con la fascetta rossa, con tutti i rapporti sul caso Granger"
telegram = "un telegramma dell'agenzia investigativa di New York"

[uscite]
"office.down" = "le scale cigolanti che scendono su Market Street"
"market_street.up" = "il portone del tuo palazzo e le scale che salgono all'ufficio"
"market_street.in" = "la tua Ford Roadster, parcheggiata accanto al marciapiede"
"market_street.north" = "Montgomery Street, verso la tavola calda di fronte al Montgomery Block"
"market_street.east" = "il palazzo di granito che ospita lo studio dell'avvocato Manson"
"market_street.west" = "Turk Street, verso il Tenderloin"
"roadster.out" = "il marciapiede di Market Street"
"roadster.south" = "la strada costiera verso sud, fino a Devil's Slide e alla villa dei Granger"
"roadster.west" = "la strada che attraversa la città verso ovest, fino all'oceano e alla Cliff House"
"granger_villa.out" = "il vialetto di ghiaia dove è parcheggiata la tua Ford"
"cliff_house.out" = "il parcheggio dove ti aspetta la tua Ford"
"manson_office.west" = "l'ascensore e l'atrio che riportano su Market Street"
"diner.south" = "Montgomery Street, che riscende verso Market Street"
"diner.in" = "dall'altra parte della strada, l'ingresso ad arco del Montgomery Block"
"diner.west" = "le strade che salgono a ovest verso Chinatown"
"chinatown.east" = "la strada che riscende alla tavola calda di Montgomery Street"
"chinatown.south" = "Turk Street, che scende verso il Tenderloin"
"tenderloin.north" = "Turk Street, che risale verso Chinatown"
"tenderloin.east" = "la strada che riporta su Market Street"
"montgomery_block.out" = "le scale che scendono su Montgomery Street, di fronte alla tavola calda"
"montgomery_block.east" = "la porta con la targhetta d'ottone JULIUS GRANGER"
"julius_apartment.west" = "la porta che dà sul corridoio del primo piano"
```

---

### `avventure/vinny-falco/traduzioni/fr.toml`

```toml
# Traduction française des textes que le moteur montre directement au joueur.
# La narration est écrite par l'IA en français ; ces noms lui sont transmis
# pour qu'elle les utilise toujours de la même façon. Ce qui manque reste en anglais.

titolo = "Vinny Falco : l'affaire de Devil's Slide"
descrizione_breve = "San Francisco, 1928. Un riche colonel, un fils en difficulté, une veuve française à l'histoire trop parfaite. Un polar hard-boiled."
introduzione = """
San Francisco, octobre 1928. La Prohibition est la loi, le bourbon de contrebande est l'habitude, et Herbert Hoover promet un poulet dans chaque marmite. Tu es **Vinny Falco**, détective privé de *Falco & Associés*. « Associés », c'est de la réclame : le seul associé, c'est toi, mais les lettres dorées sur la porte donnent une touche de respectabilité. Le plâtre écaillé du palier raconte une tout autre histoire.

C'est un après-midi paresseux. Le cornet de King Oliver gémit dans ta nouvelle radio à lampes, cadeau d'une journée de chance aux courses, et tu somnoles les pieds sur le bureau. Puis le téléphone sonne.
"""

[stanze]
office = "Le bureau de Falco & Associés"
market_street = "Market Street"
roadster = "Ta Ford Roadster"
granger_villa = "La villa des Granger à Devil's Slide"
cliff_house = "Le restaurant de la Cliff House"
manson_office = "Le cabinet de maître Manson"
diner = "Le diner de Montgomery Street"
chinatown = "Chinatown et le salon de Madame Sybilla"
tenderloin = "Une ruelle du Tenderloin"
montgomery_block = "Le premier étage du Montgomery Block"
julius_apartment = "L'appartement de Julius Granger"

[oggetti]
revolver = "ton revolver Colt Police Positive calibre 38"
lockpicks = "un rouleau de crochets de serrurier enveloppé dans une peau de chamois"
heroin_kit = "une boîte en métal avec des seringues, des cuillères noircies et un sachet de poudre blanchâtre"
anonymous_letter = "une lettre anonyme tapée à la machine, sans signature"
manson_invitation = "un mot de maître Manson sur papier à en-tête : « Dîner à la Cliff House, sept heures. Nous devons parler. »"
miles_folder = "une chemise couleur moutarde sur Miles Harrington"
case_binder = "un épais classeur au ruban rouge, avec tous les rapports sur l'affaire Granger"
telegram = "un télégramme de l'agence de détectives de New York"

[uscite]
"office.down" = "l'escalier qui grince et descend sur Market Street"
"market_street.up" = "la porte de ton immeuble et l'escalier qui monte au bureau"
"market_street.in" = "ta Ford Roadster, garée le long du trottoir"
"market_street.north" = "Montgomery Street, vers le diner en face du Montgomery Block"
"market_street.east" = "l'immeuble de granit qui abrite le cabinet de maître Manson"
"market_street.west" = "Turk Street, vers le Tenderloin"
"roadster.out" = "le trottoir de Market Street"
"roadster.south" = "la route côtière vers le sud, jusqu'à Devil's Slide et la villa des Granger"
"roadster.west" = "la route qui traverse la ville vers l'ouest, jusqu'à l'océan et la Cliff House"
"granger_villa.out" = "l'allée de gravier où ta Ford est garée"
"cliff_house.out" = "le parking où ta Ford t'attend"
"manson_office.west" = "l'ascenseur et le hall qui ramènent sur Market Street"
"diner.south" = "Montgomery Street, qui redescend vers Market Street"
"diner.in" = "de l'autre côté de la rue, l'entrée en arc du Montgomery Block"
"diner.west" = "les rues qui montent vers l'ouest, vers Chinatown"
"chinatown.east" = "la rue qui redescend au diner de Montgomery Street"
"chinatown.south" = "Turk Street, qui descend vers le Tenderloin"
"tenderloin.north" = "Turk Street, qui remonte vers Chinatown"
"tenderloin.east" = "la rue qui ramène à Market Street"
"montgomery_block.out" = "l'escalier qui descend sur Montgomery Street, en face du diner"
"montgomery_block.east" = "la porte à la plaque de laiton JULIUS GRANGER"
"julius_apartment.west" = "la porte qui donne sur le couloir du premier étage"
```

---

### `avventure/vinny-falco/traduzioni/es.toml`

```toml
# Traducción española de los textos que el motor muestra directamente al jugador.
# La narración la escribe la IA en español; estos nombres se le pasan para que
# los use siempre igual. Todo lo que falte queda en inglés.

titolo = "Vinny Falco: el caso de Devil's Slide"
descrizione_breve = "San Francisco, 1928. Un coronel rico, un hijo en apuros, una viuda francesa con una historia demasiado perfecta. Un noir hard-boiled."
introduzione = """
San Francisco, octubre de 1928. La Ley Seca es la ley, el bourbon de contrabando es la costumbre, y Herbert Hoover promete un pollo en cada olla. Eres **Vinny Falco**, detective privado de *Falco y Asociados*. Lo de «Asociados» es pura publicidad: el único socio eres tú, pero las letras doradas de la puerta dan un toque de respetabilidad. El yeso desconchado del rellano cuenta otra historia.

Es una tarde perezosa. La corneta de King Oliver gime en tu nueva radio de válvulas, regalo de un día de suerte en las carreras, y dormitas con los pies sobre el escritorio. Entonces suena el teléfono.
"""

[stanze]
office = "La oficina de Falco y Asociados"
market_street = "Market Street"
roadster = "Tu Ford Roadster"
granger_villa = "La villa de los Granger en Devil's Slide"
cliff_house = "El restaurante de la Cliff House"
manson_office = "El bufete del abogado Manson"
diner = "La cafetería de Montgomery Street"
chinatown = "Chinatown y el saloncito de Madame Sybilla"
tenderloin = "Un callejón del Tenderloin"
montgomery_block = "La primera planta del Montgomery Block"
julius_apartment = "El apartamento de Julius Granger"

[oggetti]
revolver = "tu revólver Colt Police Positive calibre 38"
lockpicks = "un rollo de ganzúas envuelto en piel de gamuza"
heroin_kit = "una caja de metal con jeringuillas, cucharillas ennegrecidas y una bolsita de polvo blanquecino"
anonymous_letter = "una carta anónima escrita a máquina, sin firma"
manson_invitation = "una nota del abogado Manson en papel con membrete: «Cena en la Cliff House, a las siete. Tenemos que hablar.»"
miles_folder = "una carpeta de color mostaza sobre Miles Harrington"
case_binder = "un grueso archivador con cinta roja, con todos los informes del caso Granger"
telegram = "un telegrama de la agencia de detectives de Nueva York"

[uscite]
"office.down" = "la escalera que cruje y baja a Market Street"
"market_street.up" = "el portal de tu edificio y la escalera que sube a la oficina"
"market_street.in" = "tu Ford Roadster, aparcado junto a la acera"
"market_street.north" = "Montgomery Street, hacia la cafetería frente al Montgomery Block"
"market_street.east" = "el edificio de granito donde está el bufete del abogado Manson"
"market_street.west" = "Turk Street, hacia el Tenderloin"
"roadster.out" = "la acera de Market Street"
"roadster.south" = "la carretera de la costa hacia el sur, hasta Devil's Slide y la villa de los Granger"
"roadster.west" = "la carretera que cruza la ciudad hacia el oeste, hasta el océano y la Cliff House"
"granger_villa.out" = "el camino de grava donde está aparcado tu Ford"
"cliff_house.out" = "el aparcamiento donde te espera tu Ford"
"manson_office.west" = "el ascensor y el vestíbulo que llevan de vuelta a Market Street"
"diner.south" = "Montgomery Street, que baja de nuevo hacia Market Street"
"diner.in" = "al otro lado de la calle, la entrada en arco del Montgomery Block"
"diner.west" = "las calles que suben hacia el oeste, hacia Chinatown"
"chinatown.east" = "la calle que baja de nuevo a la cafetería de Montgomery Street"
"chinatown.south" = "Turk Street, que baja hacia el Tenderloin"
"tenderloin.north" = "Turk Street, que sube de nuevo hacia Chinatown"
"tenderloin.east" = "la calle que lleva de vuelta a Market Street"
"montgomery_block.out" = "la escalera que baja a Montgomery Street, frente a la cafetería"
"montgomery_block.east" = "la puerta con la placa de latón JULIUS GRANGER"
"julius_apartment.west" = "la puerta que da al pasillo de la primera planta"
```

---

### `avventure/vinny-falco/traduzioni/de.toml`

```toml
# Deutsche Übersetzung der Texte, die das Programm dem Spieler direkt zeigt.
# Die Erzählung schreibt die KI auf Deutsch; diese Namen werden ihr übergeben,
# damit sie sie immer gleich verwendet. Was fehlt, bleibt auf Englisch.

titolo = "Vinny Falco: Der Fall Devil's Slide"
descrizione_breve = "San Francisco, 1928. Ein reicher Oberst, ein Sohn in Schwierigkeiten, eine französische Witwe mit einer zu perfekten Geschichte. Ein Hard-boiled-Noir."
introduzione = """
San Francisco, Oktober 1928. Die Prohibition ist das Gesetz, geschmuggelter Bourbon die Gewohnheit, und Herbert Hoover verspricht ein Huhn in jedem Topf. Du bist **Vinny Falco**, Privatdetektiv von *Falco & Partner*. Das „Partner“ ist reine Reklame: Der einzige Partner bist du, aber die goldenen Buchstaben an der Tür verleihen einen Hauch von Seriosität. Der abblätternde Putz im Treppenhaus erzählt eine ganz andere Geschichte.

Es ist ein träger Nachmittag. King Olivers Kornett jammert aus deinem neuen Röhrenradio, dem Geschenk eines Glückstags beim Pferderennen, und du döst mit den Füßen auf dem Schreibtisch. Dann klingelt das Telefon.
"""

[stanze]
office = "Das Büro von Falco & Partner"
market_street = "Market Street"
roadster = "Dein Ford Roadster"
granger_villa = "Die Villa der Grangers in Devil's Slide"
cliff_house = "Das Restaurant Cliff House"
manson_office = "Die Kanzlei von Anwalt Manson"
diner = "Das Diner in der Montgomery Street"
chinatown = "Chinatown und der Salon von Madame Sybilla"
tenderloin = "Eine Gasse im Tenderloin"
montgomery_block = "Der erste Stock des Montgomery Block"
julius_apartment = "Die Wohnung von Julius Granger"

[oggetti]
revolver = "dein Revolver, ein Colt Police Positive Kaliber .38"
lockpicks = "eine Rolle Dietriche, in Wildleder gewickelt"
heroin_kit = "eine Metalldose mit Spritzen, geschwärzten Löffeln und einem Beutel mit weißlichem Pulver"
anonymous_letter = "ein maschinengeschriebener anonymer Brief ohne Unterschrift"
manson_invitation = "eine Notiz von Anwalt Manson auf Briefpapier: „Abendessen im Cliff House, sieben Uhr. Wir müssen reden.“"
miles_folder = "eine senffarbene Mappe über Miles Harrington"
case_binder = "ein dicker Ordner mit rotem Band, mit allen Berichten zum Fall Granger"
telegram = "ein Telegramm der Detektei aus New York"

[uscite]
"office.down" = "die knarrende Treppe hinunter zur Market Street"
"market_street.up" = "die Haustür deines Gebäudes und die Treppe hinauf zum Büro"
"market_street.in" = "dein Ford Roadster, am Bordstein geparkt"
"market_street.north" = "die Montgomery Street, zum Diner gegenüber dem Montgomery Block"
"market_street.east" = "das Granitgebäude mit der Kanzlei von Anwalt Manson"
"market_street.west" = "die Turk Street, zum Tenderloin"
"roadster.out" = "der Bordstein der Market Street"
"roadster.south" = "die Küstenstraße nach Süden, bis Devil's Slide und zur Villa der Grangers"
"roadster.west" = "die Straße quer durch die Stadt nach Westen, bis zum Ozean und zum Cliff House"
"granger_villa.out" = "die Kieseinfahrt, in der dein Ford parkt"
"cliff_house.out" = "der Parkplatz, auf dem dein Ford wartet"
"manson_office.west" = "der Aufzug und die Eingangshalle zurück zur Market Street"
"diner.south" = "die Montgomery Street, zurück hinunter zur Market Street"
"diner.in" = "auf der anderen Straßenseite der Bogeneingang des Montgomery Block"
"diner.west" = "die Straßen, die nach Westen hinauf nach Chinatown führen"
"chinatown.east" = "die Straße zurück hinunter zum Diner in der Montgomery Street"
"chinatown.south" = "die Turk Street, hinunter zum Tenderloin"
"tenderloin.north" = "die Turk Street, wieder hinauf nach Chinatown"
"tenderloin.east" = "die Straße zurück zur Market Street"
"montgomery_block.out" = "die Treppe hinunter zur Montgomery Street, gegenüber dem Diner"
"montgomery_block.east" = "die Tür mit dem Messingschild JULIUS GRANGER"
"julius_apartment.west" = "die Tür zum Flur im ersten Stock"
```

---

## Controlli da eseguire

Per **ciascuna** delle tre avventure:

1. `avventura valida --avventura <id>` (usa `motore/target/release/avventura` o `cargo run --release --manifest-path motore/Cargo.toml -- …`). Deve terminare con codice 0 e **senza avvisi** (anche le traduzioni devono coprire tutte le stanze). Annota il contesto consigliato che `valida` riporta (§5.2). Se ci sono **errori**, correggi i file dell'avventura (non il motore, a meno che il bug sia evidentemente del motore: in quel caso spiegalo e correggi con test).
2. `avventura elenca`: tutte e tre devono comparire con titolo e descrizione breve. Nel menu di `avventura gioca --lingua it` titolo e descrizione sono quelli italiani di `traduzioni/it.toml`; con `--lingua en` quelli inglesi; con `--lingua fr`, `es`, `de` quelli dei rispettivi file.
3. Partita offline con `--profilo prova`, passando dal menu di avvio, **in italiano** (`--lingua it`): introduzione e nomi delle stanze in italiano, comandi italiani anche se le uscite sono scritte in inglese (`nord` trova `north`), `/uscite` con direzioni e descrizioni italiane, `/inventario` con i nomi italiani. L'AI simulata riporta le descrizioni delle stanze come sono scritte (in inglese): è normale, un modello reale le racconta in italiano.
   - **Faro**: da `cliff` si va a `boathouse` (est), si prende la tanica, si torna, si entra in `entrance` (nord); `su` dalle `stairs` è bloccato senza chiave.
   - **Space Chicken**: dalla `mess_hall` si prendono padella, tonno e guscio; `nord` al corridoio; `est` verso il laboratorio è bloccato senza tessera; `giù` in stiva si prende la mozzarella; `est` verso l'hangar è bloccato finché l'autodistruzione non è attiva.
   - **Vinny Falco**: nell'`office` l'inventario contiene solo il revolver (i grimaldelli sono nascosti nel cassetto: l'AI simulata non li rivela, un modello reale sì); `giù` su Market Street; `dentro` nella Ford, dove `sud` (villa) e `ovest` (Cliff House) sono bloccati finché i flag `case_accepted` e `dinner_invited` sono falsi; `fuori`, `nord` alla tavola calda, `dentro` al Montgomery Block, dove `est` verso l'appartamento di Julius è bloccato.
   - Ripeti un percorso breve **in inglese** (`--lingua en`) con i comandi inglesi (`east`, `take the can`, `go north`, `look`…).
   - Ripeti il percorso del Faro **in francese**, **spagnolo** e **tedesco** (`--lingua fr` / `es` / `de`): `est`, `prends le bidon`, `ouest`, `nord`, `/sorties`, `monte`; `este`, `coge la lata`, `oeste`, `norte`, `/salidas`, `sube`; `osten`, `nimm den kanister`, `westen`, `norden`, `/ausgänge`, `hoch`. Titolo, introduzione, nomi di stanze e oggetti e `/uscite` devono essere nella lingua scelta.
4. Verifica che le avventure abbiano stati e salvataggi separati in `partite/` (un `/save prova` in ciascuna).
5. Se lo script 05 ha già creato `dist/`, copia le avventure anche in `dist/avventure/` (togliendo eventuali cartelle `*-en` di versioni precedenti) e rigenera lo zip.
6. **Facoltativo (solo se l'utente ha un'AI reale configurata — chiave cloud o modello locale — e acconsente):** gioca la soluzione di riferimento di ciascuna avventura, in italiano (e, se l'utente vuole, in un'altra lingua), tramite input da file e riporta se si arriva alla vittoria, quanti turni servono, se la narrazione è nella lingua scelta e usa i nomi del glossario e dove l'AI ha sbagliato (meccaniche ignorate, oggetti inventati, uscite non rispettate, JSON non valido). Suggerisci eventuali ritocchi ai testi delle Meccaniche, senza applicarli senza conferma.

### Report finale
Elenco file creati (per avventura), esito della validazione, contesto consigliato, esito delle prove.
