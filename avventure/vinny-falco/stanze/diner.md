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
