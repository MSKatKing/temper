use std::sync::Arc;

use gen_core::ChunkGenerator;

use crate::UnconfiguredChunkGenerator;

pub fn generator_from_name(name: &str, seed: u64) -> Option<Arc<dyn ChunkGenerator>> {
    match name.trim().to_ascii_lowercase().as_str() {
        "superflat" => Some(Arc::new(superflat::SuperflatGenerator::new(seed))),
        _ => None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_superflat_generator() {
        let generator = generator_from_name("superflat", 0);

        assert!(generator.is_some());
        assert_eq!(generator.unwrap().id().as_str(), "superflat");
    }
}
