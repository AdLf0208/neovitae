// A right click with the orb in hand (API 13).
fn observe_use_item(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    claimed: &mut bool,
) {
    crate::game::neovitae_state::on_use_item(game, stack, claimed);
}
