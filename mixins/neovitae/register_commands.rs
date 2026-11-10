// Declare `/neovitae altar|lp` (API 8).
fn observe_register(commands: &mut Vec<NativeCommand>) {
    crate::game::neovitae_state::on_register_commands(commands);
}
