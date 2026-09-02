use heck::ToShoutySnakeCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct NoiseParameter {
    first_octave: i32,
    amplitudes: Vec<f64>,
}

pub fn build() -> TokenStream {
    println!("cargo:rerun-if-changed=../../assets/extracted/noise_parameters.json");

    let params: BTreeMap<String, NoiseParameter> = serde_json::from_str(include_str!(
        "../../../assets/extracted/noise_parameters.json"
    ))
    .expect("failed to parse noise_parameters.json");

    let mut constants = Vec::new();
    let mut match_arms = Vec::new();

    for (name, param) in params.into_iter() {
        let ident = format_ident!(
            "{}",
            name.strip_prefix("minecraft:")
                .unwrap_or(&name)
                .to_shouty_snake_case()
        );

        let first_octave = param.first_octave;
        let amplitudes = param.amplitudes;

        constants.push(quote! {
            pub const #ident: NoiseParameter = NoiseParameter {
                first_octave: #first_octave,
                amplitudes: &[#(#amplitudes,)*],
            };
        });

        match_arms.push(quote! {
            #name => Some(&Self::#ident),
        })
    }

    quote::quote! {
        #[derive(Debug, Clone, PartialEq)]
        pub struct NoiseParameter {
            pub first_octave: i32,
            pub amplitudes: &'static [f64],
        }

        impl NoiseParameter {
            #(
                #constants
            )*

            pub fn get_by_name<'a>(name: &str) -> Option<&'a Self> {
                match name {
                    #(#match_arms)*
                    _ => None
                }
            }
        }
    }
}
