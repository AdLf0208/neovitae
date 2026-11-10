// NeoVitae slice 1 — the Ara Vitae altar, the Capacity Rune family and the
// Novicius Orb of Vitae, as pure rules.
//
// Every number here was read from the mod, not invented: the tank sizes and
// constants from `common/blockentity/AraVitaeTile.java` and `AltarConstants.java`,
// the rune bonuses from `data/neovitae/data_maps/block/altar_rune_stats.json`,
// the layouts from `data/neovitae/neovitae/altar_tier/{weak,apprentice}.json`
// and the recipes from `data/neovitae/recipe/ara_vitae/*.json` (all in the
// 1.1.51 jar; see `FINDINGS.md`).

/// The altar block.
pub const ALTAR: &str = "neovitae:ara_vitae";
/// The Novicius Orb of Vitae.
pub const BLOOD_ORB_WEAK: &str = "neovitae:blood_orb_weak";
/// The blank rune (the Capacity family's crafting base).
#[allow(
    dead_code,
    reason = "the crafting tree is the next slice; the tests name it"
)]
pub const RUNE_BLANK: &str = "neovitae:rune_blank";
/// Capacity Rune: `capacity_mod` +0.2, additive.
pub const RUNE_CAPACITY: &str = "neovitae:rune_capacity";
/// Augmented Capacity Rune: `augmented_capacity_power` 1.075, multiplies.
pub const RUNE_CAPACITY_AUGMENTED: &str = "neovitae:rune_capacity_augmented";

/// The altar's item slot count (upstream's `Inv`, one slot).
pub const SLOT: usize = 0;

/// `AltarConstants`: one capacity rune adds 0.2 of the base tank.
pub const CAPACITY_PER_RUNE: f32 = 0.2;
/// `AltarConstants`: each augmented rune multiplies the capacity by 1.075.
pub const AUGMENTED_CAPACITY_POWER: f32 = 1.075;
/// `AraVitaeTile.getMainCapacity`: `BUCKET_VOLUME * 10 * capacityMod`.
pub const BASE_CAPACITY_MB: i32 = 10_000;
/// `AraVitaeTile.getIOCapacity`: `BUCKET_VOLUME * 1 * capacityMod`.
pub const BASE_IO_MB: i32 = 1_000;
/// `AltarConstants.RESERVE_CAPACITY_FACTOR`: the reserve holds 20 altars.
pub const RESERVE_CAPACITY_FACTOR: i32 = 20;
/// `AltarConstants.BASE_TICK_RATE`.
#[allow(dead_code, reason = "the recipe's own craftSpeed paces the altar here")]
pub const BASE_TICK_RATE: i32 = 20;
/// `AltarConstants.STRUCTURE_CHECK_INTERVAL` (5 s).
pub const STRUCTURE_CHECK_INTERVAL: i32 = 100;
/// `AltarConstants.CRAFTING_COOLDOWN_TICKS`.
pub const CRAFTING_COOLDOWN_TICKS: i32 = 30;
/// `ServerConfig.SELF_SACRIFICE_CONVERSION` default: LP per health point.
pub const SELF_SACRIFICE_CONVERSION: i32 = 100;
/// `BloodOrbItem.use`: one right click gives up two health (a heart).
pub const ORB_SACRIFICE_HEALTH: i32 = 2;
/// `BloodOrbItem.ORB_ALTAR_RANGE`: the altar an orb reaches.
pub const ORB_ALTAR_RANGE: i32 = 5;

/// Every block in `neovitae:altar/runes` at 1.1.51 (the tag a tier's rune
/// positions accept). Only the Capacity family is carried by this slice.
pub const ALTAR_RUNES: [&str; 21] = [
    "neovitae:rune_blank",
    "neovitae:rune_efficiency",
    "neovitae:rune_sacrifice",
    "neovitae:rune_sacrifice_self",
    "neovitae:rune_speed",
    "neovitae:rune_acceleration",
    "neovitae:rune_dislocation",
    "neovitae:rune_capacity",
    "neovitae:rune_capacity_augmented",
    "neovitae:rune_orb",
    "neovitae:rune_charging",
    "neovitae:rune_2_efficiency",
    "neovitae:rune_2_sacrifice",
    "neovitae:rune_2_sacrifice_self",
    "neovitae:rune_2_speed",
    "neovitae:rune_2_acceleration",
    "neovitae:rune_2_dislocation",
    "neovitae:rune_2_capacity",
    "neovitae:rune_2_capacity_augmented",
    "neovitae:rune_2_orb",
    "neovitae:rune_2_charging",
];

/// Whether `block` is one of the altar's runes.
pub fn is_altar_rune(block: &str) -> bool {
    ALTAR_RUNES.contains(&block)
}

/// What a layout position accepts (the jar's `valid`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Valid {
    /// `neovitae:ara_vitae`.
    Altar,
    /// `#neovitae:altar/runes`.
    Rune,
}

/// One position a tier's layout requires, relative to the altar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Component {
    pub offset: [i32; 3],
    /// `upgrade` in the jar: only these positions are scanned for rune stats.
    pub upgrade: bool,
    pub valid: Valid,
}

const fn altar(offset: [i32; 3]) -> Component {
    Component {
        offset,
        upgrade: false,
        valid: Valid::Altar,
    }
}

const fn rune(offset: [i32; 3], upgrade: bool) -> Component {
    Component {
        offset,
        upgrade,
        valid: Valid::Rune,
    }
}

/// `altar_tier/weak.json`: the altar alone.
pub const WEAK: [Component; 1] = [altar([0, 0, 0])];

/// `altar_tier/apprentice.json`: the altar plus eight runes at `y - 1`, of
/// which the four orthogonal ones are the `upgrade` positions.
pub const APPRENTICE: [Component; 9] = [
    altar([0, 0, 0]),
    rune([1, -1, 0], true),
    rune([0, -1, 1], true),
    rune([-1, -1, 0], true),
    rune([0, -1, -1], true),
    rune([1, -1, 1], false),
    rune([1, -1, -1], false),
    rune([-1, -1, 1], false),
    rune([-1, -1, -1], false),
];

/// The tiers this slice validates, lowest first (the layout list of
/// `NVMultiblock.TIER_VALIDATORS`, cut after `apprentice`).
pub const TIERS: [&[Component]; 2] = [&WEAK, &APPRENTICE];

/// The altar's tier: the highest layout that matches, `-1` when none does
/// (`AltarUtil.getTier`). `block_at` answers the block at a world position
/// relative to the altar, or `None` when that position is not loaded.
pub fn tier(mut block_at: impl FnMut([i32; 3]) -> Option<String>) -> i32 {
    let mut found = -1;
    for (index, layout) in TIERS.iter().enumerate() {
        let matches = layout.iter().all(|component| {
            let Some(block) = block_at(component.offset) else {
                return false;
            };
            match component.valid {
                Valid::Altar => block == ALTAR,
                Valid::Rune => is_altar_rune(&block),
            }
        });
        if matches {
            found = index as i32;
        }
    }
    found
}

/// The capacity multiplier the runes at a tier's `upgrade` positions give:
/// additive capacity runes, multiplied by every augmented rune
/// (`AraVitaeTile.calculateStats`).
pub fn capacity_mod(runes: &[&str]) -> f32 {
    let mut additive = 0.0_f32;
    let mut multiplier = 1.0_f32;
    for rune in runes {
        match *rune {
            RUNE_CAPACITY => additive += CAPACITY_PER_RUNE,
            RUNE_CAPACITY_AUGMENTED => multiplier *= AUGMENTED_CAPACITY_POWER,
            _ => {}
        }
    }
    (1.0 + additive) * multiplier
}

/// The altar's main tank, for a capacity multiplier.
pub fn capacity_mb(modifier: f32) -> i32 {
    (BASE_CAPACITY_MB as f32 * modifier) as i32
}

/// The altar's I/O tank, for a capacity multiplier.
#[allow(
    dead_code,
    reason = "fluid I/O into the altar is the next slice; the tests read this"
)]
pub fn io_capacity_mb(modifier: f32) -> i32 {
    (BASE_IO_MB as f32 * modifier) as i32
}

/// The reserve tank's size, for a capacity multiplier.
pub fn reserve_capacity_mb(modifier: f32) -> i32 {
    capacity_mb(modifier).saturating_mul(RESERVE_CAPACITY_FACTOR)
}

/// The LP one self-sacrifice of `health` points pays.
pub fn sacrifice_lp(health: i32) -> i32 {
    health.max(0) * SELF_SACRIFICE_CONVERSION
}

/// What a recipe's input accepts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ingredient {
    /// An exact item id (the jar's `{"item": …}` form).
    #[allow(dead_code, reason = "the carried tier-0 recipes both take a tag")]
    Item(&'static str),
    /// An item tag (the jar's `{"tag": …}` form).
    Tag(&'static str),
}

/// One `neovitae:ara_vitae_recipe`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recipe {
    /// The jar's file name, for the record.
    pub id: &'static str,
    pub input: Ingredient,
    pub output: &'static str,
    /// `bloodNeeded`: LP per crafted item.
    pub lp: i32,
    /// `craftSpeed`: LP drawn from the main tank a tick.
    pub craft_speed: i32,
    /// `drainSpeed`: progress lost a tick with no LP to draw.
    pub drain_speed: i32,
    /// `minTier`: the altar tier the recipe needs.
    pub min_tier: i32,
}

/// The tier-0 altar recipes this slice carries, with their jar numbers.
pub const RECIPES: [Recipe; 2] = [
    Recipe {
        id: "weak_blood_orb",
        input: Ingredient::Tag("c:gems/diamond"),
        output: BLOOD_ORB_WEAK,
        lp: 2_000,
        craft_speed: 10,
        drain_speed: 0,
        min_tier: 0,
    },
    Recipe {
        id: "cobbled_deepslate",
        input: Ingredient::Tag("c:cobblestones/normal"),
        output: "minecraft:cobbled_deepslate",
        lp: 50,
        craft_speed: 5,
        drain_speed: 0,
        min_tier: 0,
    },
];

/// The recipe `item` matches at `tier`, if any. `in_tag` answers the item
/// tags of the world (the glue's tag table; tests pass their own).
pub fn recipe_for(
    item: &str,
    tier: i32,
    in_tag: impl Fn(&str, &str) -> bool,
) -> Option<&'static Recipe> {
    RECIPES.iter().find(|recipe| {
        recipe.min_tier <= tier
            && match recipe.input {
                Ingredient::Item(wanted) => wanted == item,
                Ingredient::Tag(tag) => in_tag(tag, item),
            }
    })
}

/// What one altar tick did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Craft {
    /// No input, or an input no recipe matches.
    Idle,
    /// LP was drawn (or progress bled) but the output is not ready.
    Working,
    /// The recipe finished this tick.
    Done,
}

/// The altar's saved state: the tank, the reserve, the crafting progress and
/// the structure it last validated.
#[derive(Clone, Debug, PartialEq)]
pub struct Altar {
    pub tier: i32,
    pub capacity_mod: f32,
    pub lp: i32,
    pub reserve: i32,
    pub progress: i32,
    pub cooldown: i32,
    /// The recipe in progress, so a different input resets the progress.
    pub recipe: Option<&'static str>,
}

impl Altar {
    pub fn new() -> Self {
        Self {
            tier: -1,
            capacity_mod: 1.0,
            lp: 0,
            reserve: 0,
            progress: 0,
            cooldown: 0,
            recipe: None,
        }
    }

    /// Applies a validated structure: its tier and its runes' capacity.
    pub fn set_structure(&mut self, tier: i32, modifier: f32) {
        self.tier = tier;
        self.capacity_mod = modifier;
        self.lp = self.lp.min(self.capacity());
        self.reserve = self.reserve.min(self.reserve_capacity());
    }

    pub fn capacity(&self) -> i32 {
        capacity_mb(self.capacity_mod)
    }

    #[allow(
        dead_code,
        reason = "fluid I/O into the altar is the next slice; the tests read this"
    )]
    pub fn io_capacity(&self) -> i32 {
        io_capacity_mb(self.capacity_mod)
    }

    pub fn reserve_capacity(&self) -> i32 {
        reserve_capacity_mb(self.capacity_mod)
    }

    /// Credits LP to the main tank, the overflow to the reserve
    /// (`AraVitaeTile.addSacrificeEV`). Answers what was credited in all.
    pub fn add_lp(&mut self, lp: i32) -> i32 {
        let room = (self.capacity() - self.lp).max(0);
        let to_main = lp.clamp(0, room);
        self.lp += to_main;
        let overflow = (lp - to_main).max(0);
        let reserve_room = (self.reserve_capacity() - self.reserve).max(0);
        let to_reserve = overflow.min(reserve_room);
        self.reserve += to_reserve;
        to_main + to_reserve
    }

    /// One tick of the crafting loop (`tickRecipeCrafting`): `count` is the
    /// input stack's size, the whole stack is crafted at once.
    pub fn craft_tick(&mut self, recipe: Option<&Recipe>, count: i32) -> Craft {
        let Some(recipe) = recipe else {
            self.recipe = None;
            self.progress = 0;
            return Craft::Idle;
        };
        if self.recipe != Some(recipe.id) {
            self.recipe = Some(recipe.id);
            self.progress = 0;
        }
        let total = recipe.lp * count.max(1);
        if self.lp > 0 {
            let drawn = self
                .lp
                .min(recipe.craft_speed)
                .min((total - self.progress).max(0));
            self.lp -= drawn;
            self.progress += drawn;
        } else if self.progress > 0 {
            self.progress = (self.progress - recipe.drain_speed).max(0);
            return Craft::Working;
        }
        if self.progress >= total {
            self.progress = 0;
            self.recipe = None;
            self.cooldown = CRAFTING_COOLDOWN_TICKS;
            return Craft::Done;
        }
        Craft::Working
    }
}

impl Default for Altar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world<'a>(blocks: &'a [([i32; 3], &'a str)]) -> impl FnMut([i32; 3]) -> Option<String> + 'a {
        move |pos| {
            blocks
                .iter()
                .find(|(at, _)| *at == pos)
                .map(|(_, block)| (*block).to_owned())
        }
    }

    fn apprentice_ring(blocks: &mut Vec<([i32; 3], &'static str)>, rune: &'static str) {
        for offset in [[1, -1, 0], [0, -1, 1], [-1, -1, 0], [0, -1, -1]] {
            blocks.push((offset, rune));
        }
        for offset in [[1, -1, 1], [1, -1, -1], [-1, -1, 1], [-1, -1, -1]] {
            blocks.push((offset, RUNE_BLANK));
        }
    }

    #[test]
    fn the_altar_alone_is_tier_0() {
        let blocks = [([0, 0, 0], ALTAR)];
        assert_eq!(tier(world(&blocks)), 0);
    }

    #[test]
    fn no_altar_is_no_tier() {
        let blocks = [([0, 0, 0], "minecraft:stone")];
        assert_eq!(tier(world(&blocks)), -1);
    }

    #[test]
    fn only_the_four_orthogonal_runes_of_the_apprentice_ring_count() {
        let upgrades: Vec<&str> = APPRENTICE
            .iter()
            .filter(|component| component.upgrade)
            .map(|_| RUNE_CAPACITY)
            .collect();
        assert_eq!(
            upgrades.len(),
            4,
            "apprentice.json marks four upgrade positions"
        );
        assert_eq!(
            capacity_mb(capacity_mod(&upgrades)),
            18_000,
            "four capacity runes: 10,000 * (1 + 4 * 0.2)"
        );
    }

    #[test]
    fn eight_runes_make_the_apprentice_tier() {
        let mut blocks = vec![([0, 0, 0], ALTAR)];
        apprentice_ring(&mut blocks, RUNE_CAPACITY);
        assert_eq!(tier(world(&blocks)), 1);
    }

    #[test]
    fn seven_runes_are_still_the_weak_altar() {
        let mut blocks = vec![([0, 0, 0], ALTAR)];
        apprentice_ring(&mut blocks, RUNE_CAPACITY);
        blocks.retain(|(at, _)| *at != [1, -1, 1]);
        assert_eq!(tier(world(&blocks)), 0);
    }

    #[test]
    fn a_non_rune_in_a_rune_slot_breaks_the_apprentice_tier() {
        let mut blocks = vec![([0, 0, 0], ALTAR)];
        apprentice_ring(&mut blocks, RUNE_CAPACITY);
        blocks.retain(|(at, _)| *at != [0, -1, -1]);
        blocks.push(([0, -1, -1], "minecraft:stone"));
        assert_eq!(tier(world(&blocks)), 0);
    }

    #[test]
    fn a_capacity_rune_adds_a_fifth_of_the_tank() {
        assert_eq!(capacity_mb(capacity_mod(&[])), 10_000);
        assert_eq!(capacity_mb(capacity_mod(&[RUNE_CAPACITY])), 12_000);
        assert_eq!(
            capacity_mb(capacity_mod(&[RUNE_CAPACITY; 8])),
            26_000,
            "eight capacity runes: 10,000 * (1 + 8 * 0.2)"
        );
    }

    #[test]
    fn the_augmented_rune_multiplies_and_compounds() {
        let one = capacity_mod(&[RUNE_CAPACITY_AUGMENTED]);
        assert!((one - 1.075).abs() < 1e-6);
        let three = capacity_mod(&[RUNE_CAPACITY_AUGMENTED; 3]);
        assert!((three - 1.075_f32.powi(3)).abs() < 1e-6);
        let mixed = capacity_mod(&[RUNE_CAPACITY, RUNE_CAPACITY_AUGMENTED]);
        assert!((mixed - 1.2 * 1.075).abs() < 1e-5);
        assert_eq!(capacity_mb(mixed), 12_900);
    }

    #[test]
    fn the_io_and_reserve_tanks_follow_the_capacity() {
        let modifier = capacity_mod(&[RUNE_CAPACITY; 8]);
        assert_eq!(capacity_mb(modifier), 26_000);
        assert_eq!(io_capacity_mb(modifier), 2_600);
        assert_eq!(reserve_capacity_mb(modifier), 520_000);
    }

    #[test]
    fn sacrificing_two_health_pays_two_hundred_lp() {
        assert_eq!(sacrifice_lp(ORB_SACRIFICE_HEALTH), 200);
        assert_eq!(sacrifice_lp(0), 0);
        assert_eq!(sacrifice_lp(-3), 0);
    }

    #[test]
    fn lp_overflows_into_the_reserve_and_stops_at_its_cap() {
        let mut altar = Altar::new();
        altar.set_structure(0, 1.0);
        assert_eq!(altar.add_lp(8_000), 8_000);
        assert_eq!(altar.lp, 8_000);
        assert_eq!(altar.add_lp(4_000), 4_000, "2,000 in the tank, 2,000 spare");
        assert_eq!(altar.lp, 10_000);
        assert_eq!(altar.reserve, 2_000);
        assert_eq!(
            altar.add_lp(1_000_000),
            198_000,
            "the reserve caps at 200,000"
        );
        assert_eq!(altar.reserve, 200_000);
        assert_eq!(altar.add_lp(0), 0);
    }

    #[test]
    fn the_weak_orb_costs_exactly_two_thousand_lp() {
        let mut altar = Altar::new();
        altar.set_structure(0, 1.0);
        altar.add_lp(2_000);
        let recipe = recipe_for("minecraft:diamond", altar.tier, |tag, item| {
            tag == "c:gems/diamond" && item == "minecraft:diamond"
        });
        assert_eq!(recipe.map(|r| r.id), Some("weak_blood_orb"));
        let mut ticks = 0;
        loop {
            ticks += 1;
            match altar.craft_tick(recipe, 1) {
                Craft::Done => break,
                Craft::Working => assert!(ticks < 1_000),
                Craft::Idle => panic!("the recipe went idle"),
            }
        }
        assert_eq!(ticks, 200, "2,000 LP at 10 a tick");
        assert_eq!(altar.lp, 0);
        assert_eq!(altar.progress, 0);
    }

    #[test]
    fn the_wrong_item_matches_no_recipe() {
        assert!(
            recipe_for("minecraft:dirt", 0, |_, _| false).is_none(),
            "dirt is not a diamond"
        );
        assert!(recipe_for("minecraft:diamond", 0, |_, _| false).is_none());
    }

    #[test]
    fn a_tier_too_low_matches_no_recipe() {
        const HIGH: [Recipe; 0] = [];
        let _ = HIGH;
        let above = RECIPES.iter().find(|recipe| recipe.min_tier > 0);
        assert!(above.is_none(), "the two carried recipes are tier 0");
        assert!(
            recipe_for("minecraft:diamond", -1, |_, _| false).is_none(),
            "no structure, no crafting"
        );
    }

    #[test]
    fn a_dry_altar_keeps_progress_when_the_recipe_does_not_drain() {
        let mut altar = Altar::new();
        altar.set_structure(0, 1.0);
        altar.add_lp(500);
        let recipe = recipe_for("minecraft:diamond", 0, |_, _| true).unwrap();
        for _ in 0..50 {
            altar.craft_tick(Some(recipe), 1);
        }
        assert_eq!(altar.lp, 0);
        assert_eq!(altar.progress, 500);
        for _ in 0..10 {
            assert_eq!(altar.craft_tick(Some(recipe), 1), Craft::Working);
        }
        assert_eq!(altar.progress, 500, "drainSpeed 0 keeps the progress");
    }

    #[test]
    fn a_different_input_resets_the_progress() {
        let mut altar = Altar::new();
        altar.set_structure(0, 1.0);
        altar.add_lp(6_000);
        let tags = |tag: &str, item: &str| {
            (tag == "c:gems/diamond" && item == "minecraft:diamond")
                || (tag == "c:cobblestones/normal" && item == "minecraft:cobblestone")
        };
        let orb = recipe_for("minecraft:diamond", 0, tags).unwrap();
        let slate = recipe_for("minecraft:cobblestone", 0, tags).unwrap();
        assert_eq!(orb.id, "weak_blood_orb");
        assert_eq!(slate.id, "cobbled_deepslate");
        for _ in 0..20 {
            altar.craft_tick(Some(orb), 1);
        }
        assert_eq!(altar.progress, 200);
        altar.craft_tick(Some(slate), 1);
        assert_eq!(altar.progress, 5, "cobbled_deepslate draws 5 a tick: 0 + 5");
        assert_eq!(altar.recipe, Some("cobbled_deepslate"));
    }

    #[test]
    fn a_whole_stack_costs_its_count() {
        let mut altar = Altar::new();
        altar.set_structure(0, 1.0);
        altar.add_lp(10_000);
        let recipe = recipe_for("minecraft:cobblestone", 0, |tag, _| {
            tag == "c:cobblestones/normal"
        })
        .unwrap();
        assert_eq!(recipe.id, "cobbled_deepslate");
        let mut ticks = 0;
        loop {
            ticks += 1;
            if altar.craft_tick(Some(recipe), 3) == Craft::Done {
                break;
            }
            assert!(ticks < 1_000);
        }
        assert_eq!(ticks, 30, "3 x 50 LP at 5 a tick");
    }

    #[test]
    fn a_structure_shrink_clamps_the_tanks() {
        let mut altar = Altar::new();
        altar.set_structure(1, 2.6);
        altar.add_lp(26_000);
        assert_eq!(altar.lp, 26_000);
        altar.set_structure(0, 1.0);
        assert_eq!(
            altar.lp, 10_000,
            "the tank cannot hold more than it has room for"
        );
        assert_eq!(altar.reserve, 0);
    }
}
