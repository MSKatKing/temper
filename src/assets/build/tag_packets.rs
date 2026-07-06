use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

type RegistryIds = BTreeMap<String, i32>;
type TagPackets = BTreeMap<String, BTreeMap<String, Vec<i32>>>;

pub fn write_tag_packets(out_dir: &Path, assets_dir: &Path, data_dir: &Path) -> PathBuf {
    let extracted_dir = assets_dir
        .parent()
        .expect("Assets directory should have a parent")
        .join("extracted");
    let tags_path = extracted_dir.join("tags.json");

    println!("cargo:rerun-if-changed={}", tags_path.display());

    let tags: BTreeMap<String, BTreeMap<String, Vec<String>>> =
        serde_json::from_str(&fs::read_to_string(&tags_path).expect("Failed to read tags.json"))
            .expect("Failed to parse tags.json");

    let report_registry_ids = read_report_registry_ids(assets_dir);
    let extracted_registry_ids = read_extracted_registry_ids(&extracted_dir);

    let mut tag_packets = TagPackets::new();
    for (tag_registry, tag_set) in tags {
        let registry_id = format!("minecraft:{tag_registry}");
        let registry_ids = report_registry_ids
            .get(&registry_id)
            .cloned()
            .or_else(|| extracted_registry_ids.get(&registry_id).cloned())
            .unwrap_or_else(|| read_data_registry_ids(data_dir, &registry_id));

        let mut packet_tags = BTreeMap::new();
        for (tag_name, values) in tag_set {
            let values = values
                .iter()
                .map(|value| {
                    let value = namespaced(value);
                    *registry_ids.get(&value).unwrap_or_else(|| {
                        panic!("No protocol id for tag value {value} in {registry_id}")
                    })
                })
                .collect();

            packet_tags.insert(tag_name, values);
        }

        tag_packets.insert(registry_id, packet_tags);
    }

    let path = out_dir.join("tag_packets.json");
    let content = serde_json::to_string(&tag_packets).expect("Failed to serialize tag packets");
    fs::write(&path, content).expect("Failed to write tag packets");
    path
}

fn read_report_registry_ids(assets_dir: &Path) -> BTreeMap<String, RegistryIds> {
    let path = assets_dir
        .join("generated")
        .join("reports")
        .join("registries.json");
    let registries: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("Failed to read registries report"))
            .expect("Failed to parse registries report");

    let mut ids = BTreeMap::new();
    let Some(registries) = registries.as_object() else {
        return ids;
    };

    for (registry_id, registry) in registries {
        let Some(entries) = registry.get("entries").and_then(Value::as_object) else {
            continue;
        };

        let mut registry_ids = RegistryIds::new();
        for (entry_id, entry) in entries {
            let protocol_id = entry
                .get("protocol_id")
                .and_then(Value::as_i64)
                .expect("Registry report entry missing protocol_id");
            registry_ids.insert(namespaced(entry_id), protocol_id as i32);
        }

        ids.insert(registry_id.clone(), registry_ids);
    }

    ids
}

fn read_extracted_registry_ids(extracted_dir: &Path) -> BTreeMap<String, RegistryIds> {
    let mut ids = BTreeMap::new();

    read_extracted_ids(
        &mut ids,
        "minecraft:enchantment",
        &extracted_dir.join("enchantments.json"),
    );
    read_extracted_ids(
        &mut ids,
        "minecraft:entity_type",
        &extracted_dir.join("entities.json"),
    );

    ids
}

fn read_extracted_ids(ids: &mut BTreeMap<String, RegistryIds>, registry_id: &str, path: &Path) {
    if !path.exists() {
        return;
    }

    let entries: Value =
        serde_json::from_str(&fs::read_to_string(path).expect("Failed to read extracted registry"))
            .expect("Failed to parse extracted registry");
    let Some(entries) = entries.as_object() else {
        return;
    };

    let registry_ids = ids.entry(registry_id.to_string()).or_default();
    for (entry_id, entry) in entries {
        let Some(id) = entry.get("id").and_then(Value::as_i64) else {
            continue;
        };
        registry_ids
            .entry(namespaced(entry_id))
            .or_insert(id as i32);
    }
}

fn read_data_registry_ids(data_dir: &Path, registry_id: &str) -> RegistryIds {
    let registry_dir = data_dir.join(registry_id.replacen(':', "/", 1));
    assert!(
        registry_dir.exists(),
        "Generated registry data missing for {registry_id}"
    );

    read_registry_entries(&registry_dir, &registry_dir)
        .into_keys()
        .enumerate()
        .map(|(id, entry)| (entry, id as i32))
        .collect()
}

fn read_registry_entries(dir: &Path, root: &Path) -> BTreeMap<String, ()> {
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

        registry_entries.insert(namespaced(&id), ());
    }

    registry_entries
}

fn namespaced(value: &str) -> String {
    if value.contains(':') {
        value.to_string()
    } else {
        format!("minecraft:{value}")
    }
}
