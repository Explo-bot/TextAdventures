# 06 — Avventure di esempio: "The Punta Nera Lighthouse" e "Star Trucker and the Space Chicken of Doom"

**Modello consigliato:** Claude Sonnet 5.5 (`claude-sonnet-5-5`). I contenuti sono già scritti qui sotto: il lavoro è creare i file, validarli e collaudarli.
**Durata indicativa:** 10–20 minuti.
**Prerequisiti:** script 02–05 completati (serve l'eseguibile per `valida`).

---

## Istruzioni per Claude Code

Crea nella radice le cartelle di **due** avventure con i file riportati sotto, **copiandone il contenuto esattamente** (sono già conformi al §3 di `01_SPECIFICHE.md`):

1. `avventure/faro-punta-nera/` — *The Punta Nera Lighthouse* / *Il Faro di Punta Nera* (racconto marinaresco, 6 stanze, ~40 turni);
2. `avventure/space-chicken/` — *Star Trucker and the Space Chicken of Doom* (commedia sci-fi da B-movie, 10 stanze, ~80 turni).

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

## Controlli da eseguire

Per **ciascuna** delle due avventure:

1. `avventura valida --avventura <id>` (usa `motore/target/release/avventura` o `cargo run --release --manifest-path motore/Cargo.toml -- …`). Deve terminare con codice 0 e **senza avvisi** (anche le traduzioni devono coprire tutte le stanze). Annota il contesto consigliato che `valida` riporta (§5.2). Se ci sono **errori**, correggi i file dell'avventura (non il motore, a meno che il bug sia evidentemente del motore: in quel caso spiegalo e correggi con test).
2. `avventura elenca`: entrambe devono comparire con titolo e descrizione breve. Nel menu di `avventura gioca --lingua it` titolo e descrizione sono quelli italiani di `traduzioni/it.toml`; con `--lingua en` quelli inglesi; con `--lingua fr`, `es`, `de` quelli dei rispettivi file.
3. Partita offline con `--profilo prova`, passando dal menu di avvio, **in italiano** (`--lingua it`): introduzione e nomi delle stanze in italiano, comandi italiani anche se le uscite sono scritte in inglese (`nord` trova `north`), `/uscite` con direzioni e descrizioni italiane, `/inventario` con i nomi italiani. L'AI simulata riporta le descrizioni delle stanze come sono scritte (in inglese): è normale, un modello reale le racconta in italiano.
   - **Faro**: da `cliff` si va a `boathouse` (est), si prende la tanica, si torna, si entra in `entrance` (nord); `su` dalle `stairs` è bloccato senza chiave.
   - **Space Chicken**: dalla `mess_hall` si prendono padella, tonno e guscio; `nord` al corridoio; `est` verso il laboratorio è bloccato senza tessera; `giù` in stiva si prende la mozzarella; `est` verso l'hangar è bloccato finché l'autodistruzione non è attiva.
   - Ripeti un percorso breve **in inglese** (`--lingua en`) con i comandi inglesi (`east`, `take the can`, `go north`, `look`…).
   - Ripeti il percorso del Faro **in francese**, **spagnolo** e **tedesco** (`--lingua fr` / `es` / `de`): `est`, `prends le bidon`, `ouest`, `nord`, `/sorties`, `monte`; `este`, `coge la lata`, `oeste`, `norte`, `/salidas`, `sube`; `osten`, `nimm den kanister`, `westen`, `norden`, `/ausgänge`, `hoch`. Titolo, introduzione, nomi di stanze e oggetti e `/uscite` devono essere nella lingua scelta.
4. Verifica che le avventure abbiano stati e salvataggi separati in `partite/` (un `/save prova` in ciascuna).
5. Se lo script 05 ha già creato `dist/`, copia le avventure anche in `dist/avventure/` (togliendo eventuali cartelle `*-en` di versioni precedenti) e rigenera lo zip.
6. **Facoltativo (solo se l'utente ha un'AI reale configurata — chiave cloud o modello locale — e acconsente):** gioca la soluzione di riferimento di ciascuna avventura, in italiano (e, se l'utente vuole, in un'altra lingua), tramite input da file e riporta se si arriva alla vittoria, quanti turni servono, se la narrazione è nella lingua scelta e usa i nomi del glossario e dove l'AI ha sbagliato (meccaniche ignorate, oggetti inventati, uscite non rispettate, JSON non valido). Suggerisci eventuali ritocchi ai testi delle Meccaniche, senza applicarli senza conferma.

### Report finale
Elenco file creati (per avventura), esito della validazione, contesto consigliato, esito delle prove.
