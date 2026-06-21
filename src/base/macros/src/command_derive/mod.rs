use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Field, Fields, Ident, LitStr,
    Result as SynResult,
};

pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand(input: DeriveInput) -> SynResult<proc_macro2::TokenStream> {
    let ident = input.ident;
    let command_name = command_name(&input.attrs)?;

    let Data::Enum(data_enum) = input.data else {
        return Err(syn::Error::new(
            ident.span(),
            "Command can only be derived for enums",
        ));
    };

    let mut parse_arms = Vec::new();
    let mut path_entries = Vec::new();
    let mut greedy_assertions = Vec::new();

    for variant in data_enum.variants {
        let variant_ident = variant.ident;
        let fields = match variant.fields {
            Fields::Unnamed(fields) => {
                VariantFields::Unnamed(fields.unnamed.into_iter().map(CommandField::from).collect())
            }
            Fields::Named(fields) => VariantFields::Named(
                fields
                    .named
                    .into_iter()
                    .map(|field| {
                        let ident = field.ident.clone().ok_or_else(|| {
                            syn::Error::new(field.span(), "named command fields must have names")
                        })?;
                        Ok(CommandField {
                            ident: Some(ident),
                            field,
                        })
                    })
                    .collect::<SynResult<Vec<_>>>()?,
            ),
            Fields::Unit => VariantFields::Unit,
        };

        let last_field_idx = fields.fields().len().saturating_sub(1);
        let mut raw_bindings = Vec::new();
        let mut tuple_value_exprs = Vec::new();
        let mut named_value_exprs = Vec::new();
        let mut segments = Vec::new();

        for (idx, command_field) in fields.fields().iter().enumerate() {
            let arg_name = arg_name(command_field)?;
            let field = &command_field.field;
            let ty = &field.ty;
            let raw_ident = format_ident!("__raw_{idx}");

            raw_bindings.push(quote! {
                let #raw_ident = <#ty as ::temper_command_infra::CommandArg>::recognize(__reader)?;
            });

            tuple_value_exprs.push(quote! {
                <#ty as ::temper_command_infra::CommandArg>::parse(#raw_ident)?
            });

            if let Some(field_ident) = &command_field.ident {
                named_value_exprs.push(quote! {
                    #field_ident: <#ty as ::temper_command_infra::CommandArg>::parse(#raw_ident)?
                });
            }

            segments.push(quote! {
                ::temper_command_infra::CommandPathSegment::argument(
                    #arg_name,
                    <#ty as ::temper_command_infra::CommandArg>::argument_spec(),
                )
            });

            if idx != last_field_idx {
                greedy_assertions.push(quote_spanned! { ty.span() =>
                    const _: () = assert!(
                        !matches!(
                            <#ty as ::temper_command_infra::CommandArg>::KIND,
                            ::temper_command_infra::ArgKind::GreedyTail
                        ),
                        "greedy-tail command args must be the final field in a command variant"
                    );
                });
            }
        }

        let constructor = match &fields {
            VariantFields::Unnamed(_) => quote! {
                Self::#variant_ident(#(#tuple_value_exprs),*)
            },
            VariantFields::Named(_) => quote! {
                Self::#variant_ident { #(#named_value_exprs),* }
            },
            VariantFields::Unit => quote! {
                Self::#variant_ident
            },
        };

        parse_arms.push(quote! {
            {
                let __checkpoint = __reader.checkpoint();
                let __result = (|| -> Result<Self, ::temper_command_infra::ParseError> {
                    #(#raw_bindings)*
                    __reader.expect_end()?;
                    Ok(#constructor)
                })();

                match __result {
                    Ok(__command) => return Ok(__command),
                    Err(__err) => {
                        __best_error = Some(match __best_error.take() {
                            Some(__best) => __best.farthest(__err),
                            None => __err,
                        });
                        __reader.rewind(__checkpoint);
                    }
                }
            }
        });

        path_entries.push(quote! {
            ::temper_command_infra::CommandPath::new(#command_name, vec![#(#segments),*])
        });
    }

    Ok(quote! {
        #(#greedy_assertions)*

        impl ::temper_command_infra::CommandSpec for #ident {
            const NAME: &'static str = #command_name;

            fn parse_reader(
                __reader: &mut ::temper_command_infra::CommandReader<'_>,
            ) -> Result<Self, ::temper_command_infra::ParseError> {
                let mut __best_error: Option<::temper_command_infra::ParseError> = None;

                #(#parse_arms)*

                Err(__best_error.unwrap_or_else(|| {
                    ::temper_command_infra::ParseError::expected(__reader.cursor(), "command variant")
                }))
            }

            fn paths() -> Vec<::temper_command_infra::CommandPath> {
                vec![#(#path_entries),*]
            }
        }
    })
}

enum VariantFields {
    Unnamed(Vec<CommandField>),
    Named(Vec<CommandField>),
    Unit,
}

impl VariantFields {
    fn fields(&self) -> &[CommandField] {
        match self {
            VariantFields::Unnamed(fields) | VariantFields::Named(fields) => fields,
            VariantFields::Unit => &[],
        }
    }
}

struct CommandField {
    ident: Option<Ident>,
    field: Field,
}

impl From<Field> for CommandField {
    fn from(field: Field) -> Self {
        Self { ident: None, field }
    }
}

fn command_name(attrs: &[syn::Attribute]) -> SynResult<LitStr> {
    for attr in attrs {
        if attr.path().is_ident("command") {
            return attr.parse_args::<LitStr>();
        }
    }

    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "missing #[command(\"name\")] attribute",
    ))
}

fn arg_name(command_field: &CommandField) -> SynResult<LitStr> {
    for attr in &command_field.field.attrs {
        if attr.path().is_ident("arg") {
            return attr.parse_args::<LitStr>();
        }
    }

    if let Some(ident) = &command_field.ident {
        return Ok(LitStr::new(&ident.to_string(), ident.span()));
    }

    Err(syn::Error::new(
        command_field.field.span(),
        "command tuple fields must have #[arg(\"name\")]",
    ))
}
