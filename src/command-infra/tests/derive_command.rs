use temper_command_infra::args::{
    EntityArg, GreedyStringArg, IntegerArg, PositionArg, SingleWordArg,
};
use temper_command_infra::{CommandGraph, CommandNodeKind, CommandSpec};
use temper_macros::Command;

#[derive(Debug, PartialEq, Command)]
#[command("tp")]
enum TpCommand {
    TpToPos {
        location: PositionArg,
    },
    TpToEntity {
        destination: EntityArg,
    },
    TpEntityToPos {
        target: EntityArg,
        location: PositionArg,
    },
    TpEntityToEntity {
        target: EntityArg,
        destination: EntityArg,
    },
}

#[derive(Debug, PartialEq, Command)]
#[command("overlap")]
enum OverlapCommand {
    Word { value: SingleWordArg },
    Entity { target: EntityArg },
}

#[derive(Debug, PartialEq, Command)]
#[command("say")]
enum SayCommand {
    Say { message: GreedyStringArg },
}

#[derive(Debug, PartialEq, Command)]
#[command("number")]
enum NumberCommand {
    Number { value: IntegerArg<0, 10> },
}

#[derive(Debug, PartialEq, Command)]
#[command("rename")]
enum RenameCommand {
    Rename {
        #[arg("display_name")]
        name: SingleWordArg,
    },
}

#[test]
fn tp_to_position_parses() {
    let command = TpCommand::parse("~ ~ ~").unwrap();

    assert!(matches!(command, TpCommand::TpToPos { .. }));
}

#[test]
fn tp_to_entity_parses() {
    let command = TpCommand::parse("Steve").unwrap();

    match command {
        TpCommand::TpToEntity { destination } => assert_eq!(&*destination, "Steve"),
        _ => panic!("expected entity destination"),
    }
}

#[test]
fn tp_entity_to_position_parses() {
    let command = TpCommand::parse("Steve ~ ~ ~").unwrap();

    match command {
        TpCommand::TpEntityToPos { target, location } => {
            assert_eq!(&*target, "Steve");
            assert_eq!(location.x, "~");
            assert_eq!(location.y, "~");
            assert_eq!(location.z, "~");
        }
        _ => panic!("expected entity to position"),
    }
}

#[test]
fn tp_entity_to_entity_parses() {
    let command = TpCommand::parse("Steve Alex").unwrap();

    match command {
        TpCommand::TpEntityToEntity {
            target,
            destination,
        } => {
            assert_eq!(&*target, "Steve");
            assert_eq!(&*destination, "Alex");
        }
        _ => panic!("expected entity to entity"),
    }
}

#[test]
fn failed_variants_rewind_cleanly() {
    let command = TpCommand::parse("Steve 1 2 3").unwrap();

    assert!(matches!(command, TpCommand::TpEntityToPos { .. }));
}

#[test]
fn variant_order_breaks_ties() {
    let command = OverlapCommand::parse("Steve").unwrap();

    assert!(matches!(command, OverlapCommand::Word { .. }));
}

#[test]
fn greedy_tail_variant_parses() {
    let command = SayCommand::parse("hello there").unwrap();

    match command {
        SayCommand::Say { message } => assert_eq!(&*message, "hello there"),
    }
}

#[test]
fn parse_errors_report_farthest_failure() {
    let err = NumberCommand::parse("20").unwrap_err();

    assert_eq!(err.cursor, 0);
    assert!(err.message.contains("too large"));
}

#[test]
fn graph_generation_merges_shared_prefixes() {
    let graph = CommandGraph::from_paths(&TpCommand::paths());

    let root = &graph.nodes[graph.root_idx];
    assert_eq!(root.children.len(), 1);

    let tp_idx = root.children[0];
    let tp = &graph.nodes[tp_idx];
    assert_eq!(tp.kind, CommandNodeKind::Literal);
    assert_eq!(tp.name.as_deref(), Some("tp"));

    let child_names = tp
        .children
        .iter()
        .map(|idx| graph.nodes[*idx].name.as_deref().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(child_names, vec!["location", "destination", "target"]);

    let target_idx = tp
        .children
        .iter()
        .copied()
        .find(|idx| graph.nodes[*idx].name.as_deref() == Some("target"))
        .unwrap();
    let target = &graph.nodes[target_idx];

    assert_eq!(target.children.len(), 2);
    assert!(!target.executable);
    assert!(
        target
            .children
            .iter()
            .all(|idx| graph.nodes[*idx].executable)
    );
}

#[test]
fn graph_uses_arg_attribute_names() {
    let graph = CommandGraph::from_paths(&NumberCommand::paths());
    let number_idx = graph.nodes[graph.root_idx].children[0];
    let value_idx = graph.nodes[number_idx].children[0];

    assert_eq!(graph.nodes[number_idx].name.as_deref(), Some("number"));
    assert_eq!(graph.nodes[value_idx].name.as_deref(), Some("value"));
    assert!(graph.nodes[value_idx].executable);
}

#[test]
fn arg_attribute_overrides_named_field_name() {
    let graph = CommandGraph::from_paths(&RenameCommand::paths());
    let rename_idx = graph.nodes[graph.root_idx].children[0];
    let name_idx = graph.nodes[rename_idx].children[0];

    assert_eq!(graph.nodes[name_idx].name.as_deref(), Some("display_name"));
}
