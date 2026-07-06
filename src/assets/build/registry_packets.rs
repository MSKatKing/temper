use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const REGISTRY_PACKET_IDS: &[&str] = &[
    "minecraft:worldgen/biome",
    "minecraft:chat_type",
    "minecraft:trim_pattern",
    "minecraft:trim_material",
    "minecraft:wolf_variant",
    "minecraft:wolf_sound_variant",
    "minecraft:pig_variant",
    "minecraft:frog_variant",
    "minecraft:cat_variant",
    "minecraft:cow_variant",
    "minecraft:chicken_variant",
    "minecraft:painting_variant",
    "minecraft:dimension_type",
    "minecraft:damage_type",
    "minecraft:banner_pattern",
];

pub fn write_registry_packets(out_dir: &Path, data_dir: &Path) -> PathBuf {
    let mut registry_packets = BTreeMap::new();

    for registry_id in REGISTRY_PACKET_IDS {
        let registry_dir = data_dir.join(registry_id.replacen(':', "/", 1));
        assert!(
            registry_dir.exists(),
            "Generated registry data missing for {registry_id}"
        );

        let entries = read_registry_entries(&registry_dir, &registry_dir);
        if !entries.is_empty() {
            registry_packets.insert((*registry_id).to_string(), entries);
        }
    }

    let path = out_dir.join("registry_packets.json");
    let content =
        serde_json::to_string(&registry_packets).expect("Failed to serialize registry packets");
    fs::write(&path, content).expect("Failed to write registry packets");
    path
}

fn read_registry_entries(dir: &Path, root: &Path) -> BTreeMap<String, serde_json::Value> {
    let mut entries = fs::read_dir(dir)
        .expect("Failed to read registry directory")
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to read registry directory entry");

    entries.sort_by_key(|entry| entry.path());

    let mut registry_entries = BTreeMap::new();
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            registry_entries.extend(read_registry_entries(&path, root));
            continue;
        }

        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }

        let id = path
            .strip_prefix(root)
            .expect("Registry entry should be inside registry root")
            .with_extension("")
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");

        registry_entries.insert(id, read_json(&path));
    }

    registry_entries
}

fn read_json(path: &Path) -> serde_json::Value {
    let content = fs::read_to_string(path).expect("Failed to read generated registry data");
    serde_json::from_str(&content).expect("Failed to parse generated registry data")
}
