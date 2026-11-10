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
