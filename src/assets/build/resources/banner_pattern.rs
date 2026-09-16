use crate::resources::{BakeError, gather_files};
use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::ReadDir;

#[derive(Deserialize)]
struct BannerPattern {
    asset_id: String,
    translation_key: String,
}

pub fn bake_banner_patterns(generated_data_dir: ReadDir) -> Result<TokenStream, BakeError> {
    let mut patterns = BTreeMap::new();
    gather_files::<BannerPattern>("".to_string(), generated_data_dir, &mut patterns)?;

    let mut constants = Vec::with_capacity(patterns.len());

    for (name, BannerPattern { asset_id, translation_key }) in patterns {
        constants.push(quote! {
            (#name, BannerPattern {
                asset_id: Cow::Borrowed(#asset_id),
                translation_key: Cow::Borrowed(#translation_key),
            })
        })
    }

    let len = constants.len();

    Ok(quote! {
        impl BannerPattern {
            const PATTERNS: [(&'static str, BannerPattern); #len] = [#(#constants),*];
        }
    })
}