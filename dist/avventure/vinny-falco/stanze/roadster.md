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
