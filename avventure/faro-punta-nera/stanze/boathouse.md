+++
id = "boathouse"
nome = "The boathouse"
oggetti = ["empty_can"]
oggetti_nascosti = ["brass_key"]

[uscite.west]
verso = "cliff"
descrizione = "the banging door, towards the path"
+++

## Description
A narrow, damp boathouse that smells of tar and seaweed. A small rowing boat lies upside down on two trestles. Against the wall there are tangled nets, a box of rusty tools and a tin can. Through the door, to the west, the rain blows in.

## Details
- The tin can is empty but intact, with a screw cap: good for carrying liquids.
- The tool box only holds rusty nails and a hammer with a broken handle: useless.
- Under the upturned boat, looking closely, something glints.

## Secrets
Anselmo hides the key to the lantern room (`brass_key`) under the upturned boat, hanging from a nail in the hull, because he is afraid of oil thieves.

## Mechanics
- If the player lifts, turns over, flips or looks under the boat: set `boat_turned = true` and make `brass_key` visible by adding it to the room items (`oggetti_stanza_aggiungi`). If they explicitly take it, put it in the inventory (+10 points).
- Taking the key in the same turn in which the boat is turned over is allowed.
- The nets and the tools are not items that can be taken.
