use alloc::vec::Vec;
use core::convert::TryFrom;
use core::iter::Iterator;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn enum_discriminant_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = input.ident;

    let data_enum = match input.data {
        Data::Enum(e) => e,
        _ => {
            return syn::Error::new_spanned(
                enum_name,
                "EnumDiscriminant can only be derived for enums",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut arms = Vec::with_capacity(data_enum.variants.len());

    for (idx, variant) in data_enum.variants.iter().enumerate() {
        let v_ident = &variant.ident;
        let idx_i32 = i32::try_from(idx).unwrap_or(i32::MAX);

        let pat = match &variant.fields {
            Fields::Unit => quote! { Self::#v_ident },
            Fields::Unnamed(_) => quote! { Self::#v_ident ( .. ) },
            Fields::Named(_) => quote! { Self::#v_ident { .. } },
        };

        arms.push(quote! { #pat => #idx_i32, });
    }

    let expanded = quote! {
        impl #enum_name {
            #[inline]
            pub fn discriminant(&self) -> i32 {
                match self {
                    #( #arms )*
                }
            }
        }
    };

    expanded.into()
}

pub fn inverse_enum_discriminant_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let enum_name = input.ident;

    let data_enum = match input.data {
        Data::Enum(e) => e,
        _ => {
            return syn::Error::new_spanned(
                enum_name,
                "InverseDiscriminant can only be derived for enums",
            )
            .to_compile_error()
            .into();
        }
    };

    let mut arms = Vec::with_capacity(data_enum.variants.len());

    for (idx, variant) in data_enum.variants.iter().enumerate() {
        let v_ident = &variant.ident;
        let idx_usize = idx;

        if !matches!(&variant.fields, Fields::Unit) {
            return syn::Error::new_spanned(
                variant,
                "InverseDiscriminant can only be derived for field-less enums",
            )
            .to_compile_error()
            .into();
        }

        arms.push(quote! { #idx_usize => Some(Self::#v_ident), });
    }

    let expanded = quote! {
        impl #enum_name {
            #[inline]
            pub fn from_discriminant(value: impl ::core::convert::Into<usize>) -> Option<Self> {
                match value.into() {
                    #( #arms )*
                    _ => None,
                }
            }
        }
    };

    expanded.into()
}
