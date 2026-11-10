// One server tick of the altar: structure, sacrifice, crafting (API 17).
fn observe_block_entity_tick(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    entity: &mut horizon_client::containers::ModBlockEntity,
) {
    crate::game::neovitae_state::on_tick(game, pos, entity);
}
