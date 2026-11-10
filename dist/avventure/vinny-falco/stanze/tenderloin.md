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
