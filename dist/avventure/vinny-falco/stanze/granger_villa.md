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
