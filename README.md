# neovitae (Horizon port)

A Horizon port of [neovitae](breakinblocks/NeoVitae) — MIT.

**Status: first slices.** What is below is what the port actually does today.

## Implemented so far

- The altar** `neovitae:ara_vitae` (content template `minecraft:carved_pumpkin`:
- a horizontal `facing` block with no menu, so a right click reaches the use
- hooks; hardness 2 / blast 5 / stone sounds; collision 16×12×16, occludes
- nothing). It has a block entity (`block_entity`, API 15), ticks
- (`block_entity_ticks` + `block_entity_tick`, API 15/17) and one slot
- (`block_entity_container`, API 16, no menu).
- Structure → tier**, data-driven from the jar's two layouts: `weak` (the altar
- alone, tier 0) and `apprentice` (altar + 8 runes at `y-1`, tier 1). The tier is

## Not yet ported

- Altar tiers `mage`/`master`/`archmage`/`transcendent` (pillars, t3–t6 capstones,
- tier effects), the 21-rune family beyond Capacity (speed, efficiency, sacrifice,
- self-sacrifice, acceleration, dislocation, orb, charging and their reinforced
- versions), the augmented-capacity *charging* capacity, the altar's fluid I/O
- (input/output/charging tanks), the reserve auto-move, the tier-0 bootstrap leech
- (`leechNearbyPlayers`), redstone signal/comparator output, the altar's particles
- and sounds, the block-entity renderer (the altar model is static here), the
- `.hmod` crafting tree above the altar (vitae stone, tabula rasa/robur/animata/

## Install

Build the bundle with a native-mod capable launcher, or add the file from the
registry once it is listed. Pinned to Horizon `1d63e0a` and horizon-mixins
`5a64af93` for game 26.3.

## Build from source

```sh
cargo test
python3 tools/horizon.py check --horizon <horizon checkout>
python3 tools/horizon.py build --horizon <horizon checkout>
python3 tools/package.py
```

## Credits and licence

Original mod: breakinblocks/NeoVitae. Port: AdLf0208. Licence: MIT — see [LICENSE](LICENSE);
the original notices are preserved.
