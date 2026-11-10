// An empty-hand right click takes the altar's stack (API 21).
fn observe_right_click_block(
    game: &mut crate::game::Game<'_>,
    pos: [i32; 3],
    face: [i32; 3],
    canceled: &mut bool,
) {
    crate::game::neovitae_state::on_right_click_block(game, pos, face, canceled);
}
