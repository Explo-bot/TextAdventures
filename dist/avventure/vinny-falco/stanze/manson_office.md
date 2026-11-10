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
