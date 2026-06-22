use std::fmt;

use temper_codec::net_types::{length_prefixed_vec::LengthPrefixedVec, var_int::VarInt};
use temper_command_infra::{
    ArgumentSpec, CommandGraph as InfraCommandGraph, CommandNode as InfraCommandNode,
    CommandNodeKind as InfraCommandNodeKind, IntegerProperties, ParserKind, ParserProperties,
    StringMode,
};
use temper_commands::{
    arg::primitive::{
        EntityArgumentFlags, PrimitiveArgumentFlags, PrimitiveArgumentType, int::IntArgumentFlags,
        string::StringArgumentType,
    },
    graph::{CommandGraph, node::CommandNode as OldCommandNode},
};
use temper_macros::{NetEncode, packet};

#[derive(Clone, NetEncode)]
pub struct CommandNode {
    pub flags: u8,
    pub children: LengthPrefixedVec<VarInt>,
    pub redirect_node: Option<VarInt>,
    pub name: Option<String>,
    pub parser_id: Option<PrimitiveArgumentType>,
    pub properties: Option<PrimitiveArgumentFlags>,
    pub suggestions_type: Option<String>,
}

impl fmt::Debug for CommandNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CommandNode")
            .field("node_type", &command_node_kind(self.flags))
            .field("executable", &(self.flags & 0x04 != 0))
            .field("has_redirect", &(self.flags & 0x08 != 0))
            .field("has_suggestions_type", &(self.flags & 0x10 != 0))
            .field("flags", &self.flags)
            .field("children", &self.children)
            .field("redirect_node", &self.redirect_node)
            .field("name", &self.name)
            .field("parser_id", &self.parser_id)
            .field("properties", &self.properties)
            .field("suggestions_type", &self.suggestions_type)
            .finish()
    }
}

fn command_node_kind(flags: u8) -> &'static str {
    match flags & 0x03 {
        0 => "Root",
        1 => "Literal",
        2 => "Argument",
        _ => "Invalid",
    }
}

#[derive(NetEncode, Debug)]
#[packet(packet_id = "commands", state = "play")]
pub struct CommandsPacket {
    pub graph: LengthPrefixedVec<CommandNode>,
    pub root_idx: VarInt,
}

impl CommandsPacket {
    /// Creates a CommandsPacket from the provided command graph.
    pub fn new(graph: CommandGraph) -> Self {
        Self {
            graph: LengthPrefixedVec::new(graph.nodes.iter().map(convert_old_node).collect()),
            root_idx: VarInt::new(0),
        }
    }

    pub fn from_command_infra_graph(graph: &InfraCommandGraph) -> Self {
        Self {
            graph: LengthPrefixedVec::new(graph.nodes.iter().map(convert_node).collect()),
            root_idx: VarInt::new(graph.root_idx as i32),
        }
    }
}

fn convert_old_node(node: &OldCommandNode) -> CommandNode {
    CommandNode {
        flags: node.flags,
        children: node.children.clone(),
        redirect_node: node.redirect_node.clone(),
        name: node.name.clone(),
        parser_id: node.parser_id.clone(),
        properties: node.properties.clone(),
        suggestions_type: node.suggestions_type.clone(),
    }
}

fn convert_node(node: &InfraCommandNode) -> CommandNode {
    let mut flags = match node.kind {
        InfraCommandNodeKind::Root => 0x00,
        InfraCommandNodeKind::Literal => 0x01,
        InfraCommandNodeKind::Argument => 0x02,
    };

    if node.executable {
        flags |= 0x04;
    }

    if node.argument.and_then(|arg| arg.suggestions).is_some() {
        flags |= 0x10;
    }

    CommandNode {
        flags,
        children: LengthPrefixedVec::new(
            node.children
                .iter()
                .map(|child| VarInt::new(*child as i32))
                .collect(),
        ),
        redirect_node: None,
        name: node.name.clone(),
        parser_id: node.argument.map(parser_id),
        properties: node.argument.and_then(parser_properties),
        suggestions_type: node
            .argument
            .and_then(|argument| argument.suggestions)
            .map(str::to_string),
    }
}

fn parser_id(argument: ArgumentSpec) -> PrimitiveArgumentType {
    match argument.parser {
        ParserKind::Word | ParserKind::String => PrimitiveArgumentType::String,
        ParserKind::Integer => PrimitiveArgumentType::Int,
        ParserKind::Position => PrimitiveArgumentType::Vec3,
        ParserKind::Entity => PrimitiveArgumentType::Entity,
    }
}

fn parser_properties(argument: ArgumentSpec) -> Option<PrimitiveArgumentFlags> {
    match argument.properties {
        Some(ParserProperties::String(mode)) => {
            Some(PrimitiveArgumentFlags::String(string_mode(mode)))
        }
        Some(ParserProperties::Integer(IntegerProperties { min, max })) => {
            Some(PrimitiveArgumentFlags::Int(IntArgumentFlags { min, max }))
        }
        None if argument.parser == ParserKind::Word => {
            Some(PrimitiveArgumentFlags::String(StringArgumentType::Word))
        }
        None if argument.parser == ParserKind::Entity => Some(PrimitiveArgumentFlags::Entity(
            EntityArgumentFlags::default(),
        )),
        None => None,
    }
}

fn string_mode(mode: StringMode) -> StringArgumentType {
    match mode {
        StringMode::Word => StringArgumentType::Word,
        StringMode::Quotable => StringArgumentType::Quotable,
        StringMode::Greedy => StringArgumentType::Greedy,
    }
}

#[cfg(test)]
mod tests {
    use temper_command_infra::{
        ArgumentSpec, CommandGraph, CommandPath, CommandPathSegment, ParserKind,
    };
    use temper_commands::arg::primitive::PrimitiveArgumentFlags;

    use super::CommandsPacket;

    #[test]
    fn converts_command_infra_graph_to_protocol_nodes() {
        let graph = CommandGraph::from_paths(&[CommandPath::new(
            "tp",
            vec![CommandPathSegment::argument(
                "target",
                ArgumentSpec::new(ParserKind::Entity).with_suggestions("ask_server"),
            )],
        )]);

        let packet = CommandsPacket::from_command_infra_graph(&graph);

        assert_eq!(packet.root_idx.0, 0);
        assert_eq!(packet.graph.data.len(), 3);
        assert_eq!(packet.graph.data[1].name.as_deref(), Some("tp"));
        assert_eq!(packet.graph.data[2].name.as_deref(), Some("target"));
        assert!(packet.graph.data[2].flags & 0x04 != 0);
        assert!(matches!(
            packet.graph.data[2].properties,
            Some(PrimitiveArgumentFlags::Entity(_))
        ));
        assert_eq!(
            packet.graph.data[2].suggestions_type.as_deref(),
            Some("ask_server")
        );
    }
}
