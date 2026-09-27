use lazy_static::lazy_static;
use temper_codec::net_types::length_prefixed_vec::LengthPrefixedVec;
use temper_codec::net_types::prefixed_optional::PrefixedOptional;
use temper_macros::{NetEncode, build_registry_packets, packet};

#[derive(NetEncode)]
#[packet(packet_id = "registry_data", state = "configuration")]
pub struct RegistryDataPacket {
    pub registry_id: String,
    pub entries: LengthPrefixedVec<RegistryEntry>,
}

impl RegistryDataPacket {
    pub fn new(registry_id: String, entries: Vec<RegistryEntry>) -> Self {
        Self {
            registry_id,
            entries: LengthPrefixedVec::new(entries),
        }
    }
}

lazy_static! {
    // This is a lazy static to ensure that the registry packets are only built once
    // and can be reused across multiple calls.
    pub static ref REGISTRY_PACKETS: Vec<RegistryDataPacket> = process_reg_packets();
}

fn process_reg_packets() -> Vec<RegistryDataPacket> {
    build_registry_packets!()
        .iter()
        .map(|(key, packets)| {
            let decoded: Vec<(String, Vec<u8>)> = bitcode::decode(packets).unwrap();
            RegistryDataPacket {
                registry_id: key.clone(),
                entries: LengthPrefixedVec::new(
                    decoded
                        .into_iter()
                        .map(|(id, data)| RegistryEntry {
                            id,
                            data: if data.is_empty() {
                                PrefixedOptional::None
                            } else {
                                PrefixedOptional::Some(data)
                            },
                        })
                        .collect(),
                ),
            }
        })
        .collect()
}

#[derive(NetEncode)]
pub struct RegistryEntry {
    pub id: String,
    pub data: PrefixedOptional<Vec<u8>>,
}

#[cfg(test)]
mod tests {
    use crate::outgoing::registry_data::REGISTRY_PACKETS;

    #[test]
    fn includes_tag_dependent_synced_registries() {
        let registry_ids = REGISTRY_PACKETS
            .iter()
            .map(|packet| packet.registry_id.as_str())
            .collect::<Vec<_>>();

        assert!(registry_ids.contains(&"minecraft:enchantment"));
        assert!(registry_ids.contains(&"minecraft:instrument"));
        assert!(registry_ids.contains(&"minecraft:dialog"));
        assert!(registry_ids.contains(&"minecraft:timeline"));
        assert!(registry_ids.contains(&"minecraft:world_clock"));
        assert!(registry_ids.contains(&"minecraft:worldgen/biome"));

        assert!(!registry_ids.contains(&"minecraft:trade_set"));
        assert!(!registry_ids.contains(&"minecraft:villager_trade"));
        assert!(!registry_ids.contains(&"minecraft:enchantment_provider"));
        assert!(!registry_ids.contains(&"minecraft:trial_spawner"));
        assert!(!registry_ids.contains(&"minecraft:worldgen/configured_feature"));
        assert!(!registry_ids.contains(&"minecraft:worldgen/flat_level_generator_preset"));
        assert!(!registry_ids.contains(&"minecraft:worldgen/structure"));
        assert!(!registry_ids.contains(&"minecraft:worldgen/world_preset"));
    }
}
