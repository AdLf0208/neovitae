// ... and ticks on the server (API 15).
fn observe_block_entity_ticks(block: &str, ticks: &mut bool) {
    crate::game::neovitae_state::on_block_entity_ticks(block, ticks);
}
