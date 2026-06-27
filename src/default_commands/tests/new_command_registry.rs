use bevy_ecs::prelude::World;
use temper_command_infra::{CommandPathSegment, CommandRegistry};

#[test]
fn default_commands_register_new_metadata() {
    temper_default_commands::init();

    let registry = CommandRegistry::from_static_commands();
    let paths = registry.paths_for_player(World::new().spawn_empty().id());

    assert!(paths.iter().any(|path| path.root == "tp"));
    assert!(paths.iter().any(|path| path.root == "stop"));
    assert!(paths.iter().any(|path| path.root == "echo"));
    assert!(paths.iter().any(|path| path.root == "time"));

    let stop = paths.iter().find(|path| path.root == "stop").unwrap();
    let echo = paths.iter().find(|path| path.root == "echo").unwrap();

    assert!(stop.segments.is_empty());
    assert_eq!(echo.segments.len(), 1);
    assert!(matches!(
        echo.segments[0],
        CommandPathSegment::Argument {
            name: "message",
            ..
        }
    ));

    assert!(paths.iter().any(|path| path.root == "time"
        && matches!(
            path.segments.as_slice(),
            [
                CommandPathSegment::Literal { name: "set", .. },
                CommandPathSegment::Literal { name: "day", .. }
            ]
        )));
    assert!(paths.iter().any(|path| path.root == "time"
        && matches!(
            path.segments.as_slice(),
            [
                CommandPathSegment::Literal { name: "set", .. },
                CommandPathSegment::Literal { name: "d", .. }
            ]
        )));
    assert!(paths.iter().any(|path| path.root == "time"
        && matches!(
            path.segments.as_slice(),
            [
                CommandPathSegment::Literal { name: "set", .. },
                CommandPathSegment::Argument { name: "value", .. }
            ]
        )));
}
