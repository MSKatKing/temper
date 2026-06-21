use bevy_ecs::prelude::World;
use temper_command_infra::CommandRegistry;

#[test]
fn default_commands_register_new_tp_metadata() {
    temper_default_commands::init();

    let registry = CommandRegistry::from_static_commands();
    let paths = registry.paths_for_player(World::new().spawn_empty().id());

    assert!(paths.iter().any(|path| path.root == "tp"));
}
