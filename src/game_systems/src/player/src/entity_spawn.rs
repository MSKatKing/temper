use bevy_ecs::prelude::*;
use temper_components::player::position::Position;
use temper_components::player::rotation::Rotation;
use temper_entities::MobBundle;
use temper_messages::{SpawnEntityCommand, SpawnMobBundle};
use tracing::warn;

/// Processes `/spawn` command messages by turning them into mob bundle spawns.
pub fn spawn_command_processor(
    mut spawn_commands: MessageReader<SpawnEntityCommand>,
    query: Query<(&Position, &Rotation)>,
    mut mob_bundle_events: MessageWriter<SpawnMobBundle>,
) {
    for command in spawn_commands.read() {
        let Ok((pos, rot)) = query.get(command.player_entity) else {
            warn!(
                "Failed to get position for entity {:?}",
                command.player_entity
            );
            continue;
        };

        let spawn_pos = pos.offset_forward(rot, 2.0);
        mob_bundle_events.write(SpawnMobBundle {
            bundle: MobBundle::new(command.entity_type, spawn_pos),
            persist: true,
        });
    }
}
