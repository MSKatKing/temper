use temper_codec::net_types::{length_prefixed_vec::LengthPrefixedVec, var_int::VarInt};
use temper_command_infra::{
    ArgumentSpec, CommandGraph as InfraCommandGraph, CommandNode as InfraCommandNode,
    CommandNodeKind as InfraCommandNodeKind, IntegerProperties, ParserKind, ParserProperties,
    StringMode,
};
use temper_commands::{
    arg::primitive::{
        PrimitiveArgumentFlags, PrimitiveArgumentType, int::IntArgumentFlags,
        string::StringArgumentType,
    },
    graph::{CommandGraph, node::CommandNode},
};
use temper_macros::{NetEncode, packet};

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
            graph: LengthPrefixedVec::new(graph.nodes),
            root_idx: VarInt::new(0),
        }
    }

    pub fn from_command_infra_graph(graph: &InfraCommandGraph) -> Self {
        Self {
            graph: LengthPrefixedVec::new(graph.nodes.iter().map(convert_node).collect()),
            root_idx: VarInt::new(graph.root_idx as i32),
        }
    }

    /// Creates a CommandsPacket using the globally registered command graph.
    ///
    /// This is the typical way to create this packet, as it includes all
    /// registered server commands for tab-completion and validation.
    pub fn from_global_graph() -> Self {
        Self::new(temper_commands::infrastructure::get_graph())
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

impl Default for CommandsPacket {
    fn default() -> Self {
        Self::from_global_graph()
    }
}
