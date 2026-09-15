use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(untagged)]
enum ValueOrRange {
    Range([f64; 2]),
    Value(f64),
}

#[derive(Deserialize)]
struct BiomeParams {
    continentalness: ValueOrRange,
    erosion: ValueOrRange,
    humidity: ValueOrRange,
    temperature: ValueOrRange,
    weirdness: ValueOrRange,
    depth: ValueOrRange,
    offset: ValueOrRange,
}

#[derive(Deserialize)]
struct BiomeEntry {
    biome: String,
    parameters: BiomeParams,
}

impl ToTokens for ValueOrRange {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let start = match self {
            ValueOrRange::Value(v) => *v,
            ValueOrRange::Range(range) => range[0],
        };

        let end = match self {
            ValueOrRange::Value(v) => *v,
            ValueOrRange::Range(range) => range[1],
        };

        tokens.extend(quote! {
            #start..#end
        })
    }
}

fn main() {
    #[derive(Deserialize)]
    struct ParamFormat {
        biomes: Vec<BiomeEntry>,
    }

    let biomes: ParamFormat = serde_json::from_str(
        temper_assets::generated::reports::biome_parameters::minecraft::OVERWORLD,
    )
    .unwrap();
    let biomes = biomes.biomes;

    let mut constants = Vec::new();

    for entry in biomes {
        let name = format_ident!(
            "{}",
            entry
                .biome
                .strip_prefix("minecraft:")
                .unwrap_or(entry.biome.as_str())
                .to_shouty_snake_case()
        );

        let continentalness = entry.parameters.continentalness;
        let erosion = entry.parameters.erosion;
        let humidity = entry.parameters.humidity;
        let temperature = entry.parameters.temperature;
        let weirdness = entry.parameters.weirdness;
        let depth = entry.parameters.depth;
        let offset = entry.parameters.offset;

        constants.push(quote! {
            Self {
                biome: &Biome::#name,
                continentalness: #continentalness,
                erosion: #erosion,
                humidity: #humidity,
                temperature: #temperature,
                weirdness: #weirdness,
                depth: #depth,
                offset: #offset,
            }
        });
    }

    let len = constants.len();

    let block = quote! {
        impl BiomeParameters {
            pub const OVERWORLD: [Self; #len] = [#(#constants),*];
        }
    };

    std::fs::write(
        format!("{}/biome_params.rs", std::env::var("OUT_DIR").unwrap()),
        block.to_string(),
    )
    .unwrap()
}
