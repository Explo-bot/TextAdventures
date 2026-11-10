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
