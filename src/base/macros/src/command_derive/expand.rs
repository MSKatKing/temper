use quote::{format_ident, quote};
use syn::{Data, DataEnum, DataStruct, DeriveInput, Ident, LitStr, Result as SynResult};

use super::attrs::{command_kind, variant_prefix, CommandKind, VariantPrefix};
use super::fields::{CommandFields, FieldParse};

pub fn expand(input: DeriveInput) -> SynResult<proc_macro2::TokenStream> {
    let ident = input.ident;
    let command_kind = command_kind(&input.attrs)?;

    match input.data {
        Data::Enum(data_enum) => expand_enum(&ident, command_kind, data_enum),
        Data::Struct(data_struct) => expand_struct(&ident, command_kind, data_struct),
        Data::Union(_) => Err(syn::Error::new(
            ident.span(),
            "Command can only be derived for enums or structs",
        )),
    }
}

fn expand_enum(
    ident: &Ident,
    command_kind: CommandKind,
    data_enum: DataEnum,
) -> SynResult<proc_macro2::TokenStream> {
    let mut parse_arms = Vec::new();
    let mut segment_entries = Vec::new();
    let mut greedy_assertions = Vec::new();

    for variant in data_enum.variants {
        let variant_ident = variant.ident;
        let prefix = variant_prefix(&variant.attrs)?;
        let fields = CommandFields::from_fields(variant.fields)?;

        match prefix {
            Some(VariantPrefix::Subcommand(literal)) => {
                let ty = fields.single_unnamed_type()?;
                let literal_parse = literal_parse(&literal);
                parse_arms.push(quote! {
                    {
                        let __checkpoint = __reader.checkpoint();
                        let __result = (|| -> Result<Self, ::temper_command_infra::ParseError> {
                            #literal_parse
                            let __subcommand = <#ty as ::temper_command_infra::SubcommandSpec>::parse_reader(__reader)?;
                            Ok(Self::#variant_ident(__subcommand))
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

                segment_entries.push(quote! {
                    <#ty as ::temper_command_infra::SubcommandSpec>::segments()
                        .into_iter()
                        .map(|mut __segments| {
                            let mut __path = vec![
                                ::temper_command_infra::CommandPathSegment::literal(#literal),
                            ];
                            __path.append(&mut __segments);
                            __path
                        })
                        .collect::<Vec<_>>()
                });
            }
            prefix => {
                let field_parse = FieldParse::new(fields.fields())?;
                greedy_assertions.extend(field_parse.greedy_assertions.clone());
                let constructor = constructor(ident, &variant_ident, &fields, &field_parse);
                let raw_bindings = &field_parse.raw_bindings;
                let prefix_parse = prefix_parse(prefix.as_ref());
                let prefix_segments = prefix_segments(prefix.as_ref());

                parse_arms.push(quote! {
                    {
                        let __checkpoint = __reader.checkpoint();
                        let __result = (|| -> Result<Self, ::temper_command_infra::ParseError> {
                            #prefix_parse
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

                let segments = &field_parse.segments;
                segment_entries.push(quote! {
                    vec![{
                        let mut __segments = Vec::new();
                        #prefix_segments
                        __segments.extend(vec![#(#segments),*]);
                        __segments
                    }]
                });
            }
        }
    }

    let parse_body = quote! {
        let mut __best_error: Option<::temper_command_infra::ParseError> = None;

        #(#parse_arms)*

        Err(__best_error.unwrap_or_else(|| {
            ::temper_command_infra::ParseError::expected(__reader.cursor(), "command variant")
        }))
    };

    Ok(expand_impl(
        ident,
        command_kind,
        parse_body,
        segment_entries,
        greedy_assertions,
    ))
}

fn expand_struct(
    ident: &Ident,
    command_kind: CommandKind,
    data_struct: DataStruct,
) -> SynResult<proc_macro2::TokenStream> {
    let fields = CommandFields::from_fields(data_struct.fields)?;
    let field_parse = FieldParse::new(fields.fields())?;
    let raw_bindings = &field_parse.raw_bindings;
    let constructor = struct_constructor(&fields, &field_parse);

    let parse_body = quote! {
        #(#raw_bindings)*
        __reader.expect_end()?;
        Ok(#constructor)
    };

    let segments = &field_parse.segments;
    let segment_entries = vec![quote! {
        vec![vec![#(#segments),*]]
    }];

    Ok(expand_impl(
        ident,
        command_kind,
        parse_body,
        segment_entries,
        field_parse.greedy_assertions,
    ))
}

fn expand_impl(
    ident: &Ident,
    command_kind: CommandKind,
    parse_body: proc_macro2::TokenStream,
    segment_entries: Vec<proc_macro2::TokenStream>,
    greedy_assertions: Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let segment_builder = quote! {
        let mut __segments = Vec::new();
        #(
            __segments.extend(#segment_entries);
        )*
        __segments
    };

    match command_kind {
        CommandKind::Root(command_name) => {
            let registration = expand_registration(ident);

            quote! {
                #(#greedy_assertions)*

                impl ::temper_command_infra::CommandSpec for #ident {
                    const NAME: &'static str = #command_name;

                    fn parse_reader(
                        __reader: &mut ::temper_command_infra::CommandReader<'_>,
                    ) -> Result<Self, ::temper_command_infra::ParseError> {
                        #parse_body
                    }

                    fn paths() -> Vec<::temper_command_infra::CommandPath> {
                        #segment_builder
                            .into_iter()
                            .map(|__segments| {
                                ::temper_command_infra::CommandPath::new(#command_name, __segments)
                            })
                            .collect()
                    }
                }

                #registration
            }
        }
        CommandKind::Subcommand => quote! {
            #(#greedy_assertions)*

            impl ::temper_command_infra::SubcommandSpec for #ident {
                fn parse_reader(
                    __reader: &mut ::temper_command_infra::CommandReader<'_>,
                ) -> Result<Self, ::temper_command_infra::ParseError> {
                    #parse_body
                }

                fn segments() -> Vec<Vec<::temper_command_infra::CommandPathSegment>> {
                    #segment_builder
                }
            }
        },
    }
}

fn expand_registration(ident: &Ident) -> proc_macro2::TokenStream {
    let register_fn = format_ident!("__{}_register_command", ident);
    let register_system_fn = format_ident!("__{}_register_command_system", ident);

    quote! {
        #[::temper_command_infra::ctor::ctor(unsafe)]
        #[allow(non_snake_case)]
        #[doc(hidden)]
        fn #register_fn() {
            ::temper_command_infra::register_static_command(
                ::temper_command_infra::RegisteredCommand::of::<#ident>(),
            );
        }

        #[::temper_command_infra::ctor::ctor(unsafe)]
        #[allow(non_snake_case)]
        #[doc(hidden)]
        fn #register_system_fn() {
            ::temper_command_infra::add_system(
                ::temper_command_infra::dispatch_command::<#ident>,
            );
        }
    }
}

fn constructor(
    ident: &Ident,
    variant_ident: &Ident,
    fields: &CommandFields,
    field_parse: &FieldParse,
) -> proc_macro2::TokenStream {
    match fields {
        CommandFields::Unnamed(_) => {
            let values = &field_parse.tuple_values;
            quote! {
                #ident::#variant_ident(#(#values),*)
            }
        }
        CommandFields::Named(_) => {
            let values = &field_parse.named_values;
            quote! {
                #ident::#variant_ident { #(#values),* }
            }
        }
        CommandFields::Unit => quote! {
            #ident::#variant_ident
        },
    }
}

fn struct_constructor(
    fields: &CommandFields,
    field_parse: &FieldParse,
) -> proc_macro2::TokenStream {
    match fields {
        CommandFields::Unnamed(_) => {
            let values = &field_parse.tuple_values;
            quote! {
                Self(#(#values),*)
            }
        }
        CommandFields::Named(_) => {
            let values = &field_parse.named_values;
            quote! {
                Self { #(#values),* }
            }
        }
        CommandFields::Unit => quote! {
            Self
        },
    }
}

fn prefix_parse(prefix: Option<&VariantPrefix>) -> proc_macro2::TokenStream {
    match prefix {
        Some(VariantPrefix::Literal(literal)) => literal_parse(literal),
        Some(VariantPrefix::Subcommand(_)) | None => quote! {},
    }
}

fn prefix_segments(prefix: Option<&VariantPrefix>) -> proc_macro2::TokenStream {
    match prefix {
        Some(VariantPrefix::Literal(literal)) => quote! {
            __segments.push(::temper_command_infra::CommandPathSegment::literal(#literal));
        },
        Some(VariantPrefix::Subcommand(_)) | None => quote! {},
    }
}

fn literal_parse(literal: &LitStr) -> proc_macro2::TokenStream {
    quote! {
        let __literal_cursor = __reader.cursor();
        let __actual_literal = __reader.read_word_span()?;
        if __actual_literal != #literal {
            return Err(::temper_command_infra::ParseError::new(
                __literal_cursor,
                #literal,
                format!("expected literal {}", #literal),
            ));
        }
    }
}
