// A right click with an item on the altar (API 13).
fn observe_use_item_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    face: [i32; 3],
    claimed: &mut bool,
) {
    crate::game::neovitae_state::on_use_item_on(game, stack, pos, face, claimed);
}
