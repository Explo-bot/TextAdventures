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
