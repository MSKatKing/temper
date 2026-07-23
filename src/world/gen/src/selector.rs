use std::sync::Arc;

use gen_core::ChunkGenerator;

use crate::UnconfiguredChunkGenerator;

pub fn generator_from_name(name: &str, seed: u64) -> Arc<dyn ChunkGenerator> {
    match name.trim().to_ascii_lowercase().as_str() {
        "superflat" => Arc::new(superflat::SuperflatGenerator::new(seed)),
        "unconfigured" | "none" | "" => Arc::new(UnconfiguredChunkGenerator::new(seed)),
        _ => Arc::new(UnconfiguredChunkGenerator::new(seed)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_unconfigured_generator() {
        let generator = generator_from_name("unconfigured", 0);

        assert_eq!(generator.id().as_str(), "unconfigured");
    }

    #[test]
    fn selects_superflat_generator() {
        let generator = generator_from_name("superflat", 0);

        assert_eq!(generator.id().as_str(), "superflat");
    }

    #[test]
    fn falls_back_to_unconfigured_for_unknown_generators() {
        let generator = generator_from_name("normal", 0);

        assert_eq!(generator.id().as_str(), "unconfigured");
    }
}
