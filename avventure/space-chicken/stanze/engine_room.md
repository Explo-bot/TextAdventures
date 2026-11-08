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
