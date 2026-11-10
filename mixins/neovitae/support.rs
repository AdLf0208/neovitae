// Game-side glue for NeoVitae slice 1.
//
// Injected into `crate::game` as `neovitae_state`; it needs the game, so it is
// not part of the standalone library (only `runtime.rs` is).
//
// The Ara Vitae keeps upstream's state in the block entity's saved compound:
// the validated tier and capacity multiplier, the main tank (`lp`), the reserve
// tank, the crafting progress and the crafting cooldown. Its one slot holds the
// input (or the finished output), as upstream's `Inv`.
//
// Read from Neo Vitae 1.1.51 (`AraVitaeTile.tick/tickRecipeCrafting/
// calculateStats/addSacrificeEV`, `AraVitaeBlock.useItemOn`, `BloodOrbItem.use`,
// `AltarUtil.getTier/scanForRunes`). See `../../FINDINGS.md`.

use horizon_client::containers::{BlockEntity, ModBlockEntity, ModContainer};
use horizon_client::items::ItemStack;
use horizon_client::text::Text;
use horizon_world::nbt::{Compound, JavaString, Tag};

use crate::game::Game;
use crate::game::native_server::{NativeCommand, NativeServerPlayer};
use crate::game::neovitae::{
    ALTAR, Altar, BLOOD_ORB_WEAK, Craft, ORB_ALTAR_RANGE, ORB_SACRIFICE_HEALTH, SLOT,
    STRUCTURE_CHECK_INTERVAL, capacity_mod, recipe_for, sacrifice_lp, tier,
};

/// Saved-compound keys (upstream's `saveAdditional`, shortened).
const KEY_TIER: &str = "tier";
const KEY_CAP: &str = "cap";
const KEY_LP: &str = "lp";
const KEY_RESERVE: &str = "reserve";
const KEY_PROGRESS: &str = "progress";
const KEY_COOLDOWN: &str = "cooldown";
const KEY_RECIPE: &str = "recipe";
const KEY_CHECK: &str = "check";
// Harness counters: how many clicks reached each hook on this altar.
const KEY_USES: &str = "uses";
const KEY_RIGHT: &str = "right";

// ---------------------------------------------------------------------------
// Declaration hooks
// ---------------------------------------------------------------------------

/// The altar has a block entity (`native_block_entity`, API 15).
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if block == ALTAR {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

/// ... and ticks on the server (`native_block_entity_ticks`, API 15).
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_block_entity_ticks(block: &str, ticks: &mut bool) {
    if block == ALTAR {
        *ticks = true;
    }
}

/// The altar's one slot (`native_block_entity_container`, API 16/17).
/// Upstream's `Inv` is a plain `ItemStackHandler`, so every face reaches it;
/// the altar has no menu, so a right click still reaches `use_item_on`.
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_container(block: &str, container: &mut Option<ModContainer>) {
    if block != ALTAR {
        return;
    }
    *container = Some(ModContainer {
        slots: 1,
        faces: std::array::from_fn(|_| vec![SLOT]),
        inputs: vec![SLOT],
        outputs: vec![SLOT],
        menu: None,
    });
}

// ---------------------------------------------------------------------------
// The altar's tick
// ---------------------------------------------------------------------------

/// One server tick (`native_block_entity_tick`, API 17): settle a queued
/// self-sacrifice, revalidate the structure every five seconds, then run the
/// crafting loop (`AraVitaeTile.tick`).
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_tick(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    if entity.id != ALTAR {
        return;
    }
    if entity.items.len() <= SLOT {
        entity.items.resize(SLOT + 1, None);
    }
    settle_request(game, pos, entity);
    let mut altar = load(entity);
    settle_sacrifice(game, pos, &mut altar);
    let check = read_int(&entity.nbt, KEY_CHECK).wrapping_add(1);
    if check % STRUCTURE_CHECK_INTERVAL == 0 || altar.tier < 0 {
        let (found, modifier) = scan(game, pos);
        altar.set_structure(found, modifier);
    }
    if altar.cooldown > 0 {
        altar.cooldown -= 1;
    }
    let slot = entity.items[SLOT].clone();
    if let Some(stack) = &slot
        && altar.cooldown == 0
    {
        let recipe = recipe_for(&stack.item, altar.tier, |tag, item| in_tag(game, item, tag));
        if altar.craft_tick(recipe, i32::try_from(stack.count).unwrap_or(1)) == Craft::Done
            && let Some(recipe) = recipe
        {
            entity.items[SLOT] = Some(ItemStack::new(recipe.output, stack.count));
        }
    }
    store(entity, &altar, check);
    // `setChanged`: the chunk is saved with what this tick did.
    game.world.modified.insert((pos[0] >> 4, pos[2] >> 4));
}

/// The altar's tier and the capacity multiplier of the runes standing at its
/// layout's `upgrade` positions (`AltarUtil.getTier` + `scanForRunes` +
/// `calculateStats`).
fn scan(game: &Game<'_>, pos: [i32; 3]) -> (i32, f32) {
    let found = tier(|offset| Some(game.native_block(add(pos, offset))));
    let Ok(index) = usize::try_from(found) else {
        return (found, 1.0);
    };
    let Some(layout) = crate::game::neovitae::TIERS.get(index).copied() else {
        return (found, 1.0);
    };
    let runes: Vec<String> = layout
        .iter()
        .filter(|component| component.upgrade)
        .map(|component| game.native_block(add(pos, component.offset)))
        .collect();
    let borrowed: Vec<&str> = runes.iter().map(String::as_str).collect();
    (found, capacity_mod(&borrowed))
}

// ---------------------------------------------------------------------------
// Right clicks
// ---------------------------------------------------------------------------

/// A right click with an item on the altar puts one item in its slot, or takes
/// the stack back (`AraVitaeBlock.useItemOn`), `native_use_item_on` (API 13).
///
/// The slot move itself is queued for the altar's next tick: a block entity's
/// item slots written inside a use hook are not kept (the item is gone by the
/// next read, while a saved-compound write in the same hook survives), and the
/// tick is the path the engine persists (API 17).
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_use_item_on(
    game: &mut Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    _face: [i32; 3],
    claimed: &mut bool,
) {
    if game.native_block(pos) != ALTAR {
        return;
    }
    count_click(game, pos, KEY_USES);
    *claimed = true;
    let occupied = game
        .native_with_mod_block_entity(pos, |entity| {
            entity.items.get(SLOT).is_some_and(Option::is_some)
        })
        .unwrap_or(true);
    if occupied {
        // Upstream consumes the click: an occupied altar takes nothing more.
        return;
    }
    queue(Request::Insert {
        at: pos,
        item: stack.item.to_string(),
    });
    mark_insert(pos);
    let hand = game.held_slot();
    let emptied = match game.stack_in_mut(hand) {
        Some(held) => {
            held.count = held.count.saturating_sub(1);
            held.count == 0
        }
        None => false,
    };
    if emptied {
        game.stack_in_mut(hand).take();
    }
    game.hand_stack_replaced(hand);
}

/// An empty-hand right click on the altar takes its stack back
/// (`AraVitaeBlock.useItemOn`'s second branch), `native_right_click_block`
/// (API 21). The move is queued for the altar's tick, as the insert is.
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_right_click_block(
    game: &mut Game<'_>,
    pos: [i32; 3],
    _face: [i32; 3],
    canceled: &mut bool,
) {
    if game.native_block(pos) != ALTAR {
        return;
    }
    count_click(game, pos, KEY_RIGHT);
    // The engine retries a click with the other hand once the first claimed it;
    // that retry must not undo the insert it just asked for (upstream's insert
    // answers SUCCESS, so no retry follows there).
    if inserted_recently(pos) {
        return;
    }
    let hand = game.held_slot();
    if game.stack_in_mut(hand).is_some() {
        return;
    }
    let waiting = game
        .native_with_mod_block_entity(pos, |entity| {
            entity.items.get(SLOT).is_some_and(Option::is_some)
        })
        .unwrap_or(false);
    if !waiting {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    queue(Request::Take {
        at: pos,
        player: player.uuid,
    });
    *canceled = true;
}

/// One click that reached a hook on the altar, for the readout.
fn count_click(game: &mut Game<'_>, pos: [i32; 3], key: &str) {
    game.native_with_mod_block_entity(pos, |entity| {
        let count = read_int(&entity.nbt, key).wrapping_add(1);
        write_int(&mut entity.nbt, key, count);
    });
}

/// When an item last went into an altar from a click (for the retry guard).
fn recent_inserts() -> &'static std::sync::Mutex<Vec<([i32; 3], std::time::Instant)>> {
    static INSERTS: std::sync::OnceLock<std::sync::Mutex<Vec<([i32; 3], std::time::Instant)>>> =
        std::sync::OnceLock::new();
    INSERTS.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

/// Remembers that a click put an item into the altar at `at`.
fn mark_insert(at: [i32; 3]) {
    let mut inserts = recent_inserts()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    inserts.push((at, std::time::Instant::now()));
}

/// Whether a click put an item into the altar at `at` a moment ago.
fn inserted_recently(at: [i32; 3]) -> bool {
    let mut inserts = recent_inserts()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    inserts.retain(|(_, when)| when.elapsed() < std::time::Duration::from_millis(500));
    inserts.iter().any(|(other, _)| *other == at)
}

/// A slot move a click asked for, applied by the altar's next tick.
#[derive(Clone, Debug)]
enum Request {
    /// One item of this id into the free slot.
    Insert { at: [i32; 3], item: String },
    /// The slot's stack into this player's inventory.
    Take { at: [i32; 3], player: u128 },
}

fn requests() -> &'static std::sync::Mutex<Vec<Request>> {
    static REQUESTS: std::sync::OnceLock<std::sync::Mutex<Vec<Request>>> =
        std::sync::OnceLock::new();
    REQUESTS.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

fn queue(request: Request) {
    let at = match &request {
        Request::Insert { at, .. } | Request::Take { at, .. } => *at,
    };
    let mut waiting = requests().lock().unwrap_or_else(|error| error.into_inner());
    // One click's move per altar: a second click before the tick replaces it.
    waiting.retain(|other| match other {
        Request::Insert { at: other_at, .. } | Request::Take { at: other_at, .. } => {
            *other_at != at
        }
    });
    waiting.push(request);
}

/// Applies a queued insert or take for the altar at `pos`
/// (`AraVitaeBlock.useItemOn`'s slot moves).
fn settle_request(game: &mut Game<'_>, pos: [i32; 3], entity: &mut ModBlockEntity) {
    if entity.items.len() <= SLOT {
        entity.items.resize(SLOT + 1, None);
    }
    let request = {
        let mut waiting = requests().lock().unwrap_or_else(|error| error.into_inner());
        let Some(index) = waiting.iter().position(|request| match request {
            Request::Insert { at, .. } | Request::Take { at, .. } => *at == pos,
        }) else {
            return;
        };
        waiting.remove(index)
    };
    match request {
        Request::Insert { item, .. } => {
            if entity.items[SLOT].is_none() {
                entity.items[SLOT] = Some(ItemStack::new(&item, 1));
            }
        }
        Request::Take { player, .. } => {
            let Some(stack) = entity.items[SLOT].take() else {
                return;
            };
            if let Some(left) = game.native_give_stack(player, stack.clone()) {
                entity.items[SLOT] = Some(left);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// The orb
// ---------------------------------------------------------------------------

/// A right click with the weak orb (`BloodOrbItem.use`, `native_use_item`,
/// API 13): near an altar it trades health for LP. In creative it fills the
/// altar's room, as upstream's `instabuild` branch does; in survival the
/// sacrifice is queued and settled by the altar's next tick, because the pin
/// has no helper that hurts a player from inside a player's hook.
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_use_item(game: &mut Game<'_>, stack: &ItemStack, claimed: &mut bool) {
    if &*stack.item != BLOOD_ORB_WEAK {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    let at = [
        player.position[0].floor() as i32,
        player.position[1].floor() as i32,
        player.position[2].floor() as i32,
    ];
    let Some(altar_pos) = find_altar(game, at) else {
        return;
    };
    *claimed = true;
    if player.game_mode == 1 {
        // Upstream: a creative player pays nothing and fills the room.
        let filled = game
            .native_with_mod_block_entity(altar_pos, |entity| {
                if entity.id != ALTAR {
                    return false;
                }
                let mut altar = load(entity);
                let room = (altar.capacity() - altar.lp).max(0);
                altar.add_lp(room);
                store(entity, &altar, read_int(&entity.nbt, KEY_CHECK));
                true
            })
            .unwrap_or(false);
        if !filled {
            *claimed = false;
        }
        return;
    }
    queue_sacrifice(&player.name, altar_pos);
}

/// The altar within `ORB_ALTAR_RANGE` of `at`, as `AltarUtil.findAltar` scans.
fn find_altar(game: &Game<'_>, at: [i32; 3]) -> Option<[i32; 3]> {
    for dx in -ORB_ALTAR_RANGE..=ORB_ALTAR_RANGE {
        for dy in -ORB_ALTAR_RANGE..=ORB_ALTAR_RANGE {
            for dz in -ORB_ALTAR_RANGE..=ORB_ALTAR_RANGE {
                let pos = [at[0] + dx, at[1] + dy, at[2] + dz];
                if game.native_block(pos) == ALTAR {
                    return Some(pos);
                }
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// A queued self-sacrifice
// ---------------------------------------------------------------------------

/// A survival orb's sacrifice, waiting for the altar's next tick (one per
/// altar; a second click before the tick replaces the first).
#[derive(Clone, Debug)]
struct Sacrifice {
    at: [i32; 3],
    player: String,
}

fn sacrifices() -> &'static std::sync::Mutex<Vec<Sacrifice>> {
    static SACRIFICES: std::sync::OnceLock<std::sync::Mutex<Vec<Sacrifice>>> =
        std::sync::OnceLock::new();
    SACRIFICES.get_or_init(|| std::sync::Mutex::new(Vec::new()))
}

fn queue_sacrifice(player: &str, at: [i32; 3]) {
    let mut waiting = sacrifices().lock().unwrap_or_else(|error| error.into_inner());
    waiting.retain(|entry| entry.at != at);
    waiting.push(Sacrifice {
        at,
        player: player.to_owned(),
    });
}

/// Pays a queued sacrifice: `/damage <player> 2` as the console, then credit
/// the health actually lost (upstream's `takeSacrifice` measures the same).
fn settle_sacrifice(game: &mut Game<'_>, pos: [i32; 3], altar: &mut Altar) {
    let entry = {
        let mut waiting = sacrifices().lock().unwrap_or_else(|error| error.into_inner());
        let Some(index) = waiting.iter().position(|entry| entry.at == pos) else {
            return;
        };
        waiting.remove(index)
    };
    let before = health_of(game, &entry.player);
    let ran = game.native_run_command(&format!(
        "damage {} {} minecraft:magic",
        entry.player, ORB_SACRIFICE_HEALTH
    ));
    let after = health_of(game, &entry.player);
    let lost = match (before, after, ran) {
        (Some(before), Some(after), true) => (before - after).max(0.0),
        _ => 0.0,
    };
    let credited = sacrifice_lp(lost.round() as i32);
    altar.add_lp(credited);
}

fn health_of(game: &Game<'_>, name: &str) -> Option<f32> {
    game.native_server_players()
        .into_iter()
        .find(|player| player.name == name)
        .map(|player| player.health)
}

// ---------------------------------------------------------------------------
// The `/neovitae` harness controls
// ---------------------------------------------------------------------------

/// The commands (`native_register_commands`, API 8). Upstream has no commands:
/// these exist so the harness can read and set an altar without a GUI
/// (precedent `/charginggadgets info|energy`).
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_register_commands(commands: &mut Vec<NativeCommand>) {
    commands.push(NativeCommand::new("neovitae altar <pos:block_pos>"));
    commands.push(NativeCommand::op(
        "neovitae lp <pos:block_pos> <amount:integer>",
        2,
    ));
    commands.push(NativeCommand::op(
        "neovitae insert <pos:block_pos> <item:string>",
        2,
    ));
}

/// `/neovitae altar|lp` (`native_perform_command`, API 8).
#[allow(
    private_interfaces,
    reason = "the injected bundle is this crate's own module"
)]
pub fn on_perform_command(
    game: &mut Game<'_>,
    _player: &NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    let words: Vec<&str> = command.split_whitespace().collect();
    if words.first() != Some(&"neovitae") {
        return;
    }
    *ran = true;
    match words.get(1).copied() {
        Some("altar") => {
            let Some(pos) = parse_pos(&words[2..]) else {
                game.native_feedback(Text::literal("usage: /neovitae altar <x> <y> <z>"));
                return;
            };
            game.native_feedback(Text::literal(readout(game, pos)));
        }
        Some("lp") => {
            let Some((pos, amount)) = parse_pos_amount(&words[2..]) else {
                game.native_feedback(Text::literal("usage: /neovitae lp <x> <y> <z> <amount>"));
                return;
            };
            let Some(read) = load_at(game, pos) else {
                game.native_feedback(Text::literal(format!("{pos:?} is not an Ara Vitae.")));
                return;
            };
            let capacity = read.capacity();
            let wanted = amount.clamp(0, capacity);
            let changed = game
                .native_with_mod_block_entity(pos, |entity| {
                    if entity.id != ALTAR {
                        return false;
                    }
                    let mut altar = load(entity);
                    altar.lp = wanted;
                    store(entity, &altar, read_int(&entity.nbt, KEY_CHECK));
                    true
                })
                .unwrap_or(false);
            game.native_feedback(Text::literal(if changed {
                format!("Ara Vitae at {pos:?}: main tank set to {wanted}/{capacity}.")
            } else {
                format!("{pos:?} is not an Ara Vitae.")
            }));
        }
        Some("insert") => {
            let (Some(pos), Some(item)) = (parse_pos(&words[2..]), words.get(5).copied()) else {
                game.native_feedback(Text::literal(
                    "usage: /neovitae insert <x> <y> <z> <item>",
                ));
                return;
            };
            let placed = game
                .native_with_mod_block_entity(pos, |entity| {
                    if entity.id != ALTAR {
                        return false;
                    }
                    if entity.items.len() <= SLOT {
                        entity.items.resize(SLOT + 1, None);
                    }
                    entity.items[SLOT] = Some(ItemStack::new(item, 1));
                    true
                })
                .unwrap_or(false);
            game.native_feedback(Text::literal(format!(
                "Ara Vitae at {pos:?}: insert {item} -> {placed}"
            )));
        }
        _ => game.native_feedback(Text::literal("usage: /neovitae altar|lp|insert ...")),
    }
}

fn load_at(game: &Game<'_>, pos: [i32; 3]) -> Option<Altar> {
    match game.world.block_entities.get(&(pos[0], pos[1], pos[2])) {
        Some(BlockEntity::Mod(entity)) if entity.id == ALTAR => Some(load(entity)),
        _ => None,
    }
}

/// The altar's state, as `/neovitae altar` prints it.
fn readout(game: &Game<'_>, pos: [i32; 3]) -> String {
    let Some(BlockEntity::Mod(entity)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))
    else {
        return format!("{pos:?} is not a mod block entity.");
    };
    if entity.id != ALTAR {
        return format!("{pos:?} is a {}, not an Ara Vitae.", entity.id);
    }
    let altar = load(entity);
    let slots = entity.items.len();
    let slot = match entity.items.get(SLOT).and_then(Option::as_ref) {
        Some(stack) => format!("{} x{}", stack.item, stack.count),
        None => "empty".to_owned(),
    };
    format!(
        "Ara Vitae at {pos:?}: tier {}, lp {}/{}, reserve {}, progress {}, slots {}, slot {}, clicks {}/{}, t={}",
        altar.tier,
        altar.lp,
        altar.capacity(),
        altar.reserve,
        altar.progress,
        slots,
        slot,
        read_int(&entity.nbt, KEY_USES),
        read_int(&entity.nbt, KEY_RIGHT),
        game.world.game_time
    )
}

// ---------------------------------------------------------------------------
// Saved-compound reads
// ---------------------------------------------------------------------------

fn load(entity: &ModBlockEntity) -> Altar {
    Altar {
        tier: read_int(&entity.nbt, KEY_TIER),
        capacity_mod: read_float(&entity.nbt, KEY_CAP).max(0.0),
        lp: read_int(&entity.nbt, KEY_LP),
        reserve: read_int(&entity.nbt, KEY_RESERVE),
        progress: read_int(&entity.nbt, KEY_PROGRESS),
        cooldown: read_int(&entity.nbt, KEY_COOLDOWN),
        recipe: recipe_id(&entity.nbt),
    }
}

fn store(entity: &mut ModBlockEntity, altar: &Altar, check: i32) {
    write_int(&mut entity.nbt, KEY_TIER, altar.tier);
    write_float(&mut entity.nbt, KEY_CAP, altar.capacity_mod);
    write_int(&mut entity.nbt, KEY_LP, altar.lp);
    write_int(&mut entity.nbt, KEY_RESERVE, altar.reserve);
    write_int(&mut entity.nbt, KEY_PROGRESS, altar.progress);
    write_int(&mut entity.nbt, KEY_COOLDOWN, altar.cooldown);
    match altar.recipe {
        Some(id) => {
            entity.nbt.insert(JavaString::from(KEY_RECIPE), Tag::String(id.into()));
        }
        None => {
            entity.nbt.remove(&JavaString::from(KEY_RECIPE));
        }
    }
    write_int(&mut entity.nbt, KEY_CHECK, check);
}

fn in_tag(game: &Game<'_>, item: &str, tag: &str) -> bool {
    let stack = ItemStack::new(item, 1);
    game.native_server_item_in_tag(&stack, tag).unwrap_or(false)
}

fn add(pos: [i32; 3], offset: [i32; 3]) -> [i32; 3] {
    [pos[0] + offset[0], pos[1] + offset[1], pos[2] + offset[2]]
}

fn parse_pos(words: &[&str]) -> Option<[i32; 3]> {
    Some([
        words.first()?.parse().ok()?,
        words.get(1)?.parse().ok()?,
        words.get(2)?.parse().ok()?,
    ])
}

fn parse_pos_amount(words: &[&str]) -> Option<([i32; 3], i32)> {
    let pos = parse_pos(words)?;
    let amount = words.get(3)?.parse().ok()?;
    Some((pos, amount))
}

fn read_int(nbt: &Compound, key: &str) -> i32 {
    match nbt.get(&JavaString::from(key)) {
        Some(Tag::Int(value)) => *value,
        _ => 0,
    }
}

/// The recipe in progress, matched back to the table (`Altar.recipe` is a
/// borrowed id upstream too: a renamed recipe just resets the progress).
fn recipe_id(nbt: &Compound) -> Option<&'static str> {
    let Tag::String(id) = nbt.get(&JavaString::from(KEY_RECIPE))? else {
        return None;
    };
    let id = id.to_string().ok()?;
    crate::game::neovitae::RECIPES
        .iter()
        .find(|recipe| recipe.id == id)
        .map(|recipe| recipe.id)
}

fn read_float(nbt: &Compound, key: &str) -> f32 {
    match nbt.get(&JavaString::from(key)) {
        Some(Tag::Float(bits)) => f32::from_bits(*bits),
        _ => 1.0,
    }
}

fn write_int(nbt: &mut Compound, key: &str, value: i32) {
    nbt.insert(JavaString::from(key), Tag::Int(value));
}

fn write_float(nbt: &mut Compound, key: &str, value: f32) {
    nbt.insert(JavaString::from(key), Tag::Float(value.to_bits()));
}
