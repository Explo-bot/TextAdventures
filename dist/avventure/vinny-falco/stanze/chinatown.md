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
