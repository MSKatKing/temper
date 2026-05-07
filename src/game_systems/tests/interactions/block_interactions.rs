use interactions::block_interactions::{
    get_interaction_type, is_interactive, try_interact, InteractionResult, InteractionType,
};
use std::collections::BTreeMap;
use temper_core::block_data::BlockData;
use temper_core::block_state_id::BlockStateId;
use temper_macros::block;

#[test]
fn door_detection() {
    let door_data = BlockData {
        name: "minecraft:oak_door".to_string(),
        properties: Some(BTreeMap::from([
            ("facing".to_string(), "north".to_string()),
            ("open".to_string(), "false".to_string()),
            ("half".to_string(), "lower".to_string()),
            ("hinge".to_string(), "left".to_string()),
        ])),
    };

    assert!(matches!(
        get_interaction_type(&door_data),
        Some(InteractionType::Toggleable("open"))
    ));
}

#[test]
fn try_interact_opens_door() {
    let closed_door = block!("oak_door", { facing: "north", half: "lower", hinge: "left", open: false, powered: false });

    let result = try_interact(closed_door);
    let InteractionResult::Toggled(new_id) = result else {
        panic!("Expected Toggled, got {:?}", result);
    };

    let new_data = new_id
        .to_block_data()
        .expect("new state ID should be valid");
    let props = new_data.properties.expect("door should have properties");
    assert_eq!(props["open"], "true");
}

#[test]
fn try_interact_closes_door() {
    let open_door = block!("oak_door", { facing: "north", half: "lower", hinge: "left", open: true, powered: false });

    let result = try_interact(open_door);
    let InteractionResult::Toggled(new_id) = result else {
        panic!("Expected Toggled, got {:?}", result);
    };

    let new_data = new_id
        .to_block_data()
        .expect("new state ID should be valid");
    let props = new_data.properties.expect("door should have properties");
    assert_eq!(props["open"], "false");
}

#[test]
fn try_interact_not_interactive() {
    let stone = block!("stone");
    assert!(matches!(
        try_interact(stone),
        InteractionResult::NotInteractive
    ));
}

#[test]
fn is_interactive_reports_doors_only() {
    let door = block!("oak_door", { facing: "north", half: "lower", hinge: "left", open: false, powered: false });
    let stone = block!("stone");

    assert!(is_interactive(door));
    assert!(!is_interactive(stone));
}
