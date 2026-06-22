use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse_macro_input, spanned::Spanned, Data, DataEnum, DataStruct, DeriveInput, Field, Fields,
    Ident, LitStr, Result as SynResult,
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

    match input.data {
        Data::Enum(data_enum) => expand_enum(&ident, command_name, data_enum),
        Data::Struct(data_struct) => expand_struct(&ident, command_name, data_struct),
        Data::Union(_) => Err(syn::Error::new(
            ident.span(),
            "Command can only be derived for enums or structs",
        )),
    }
}

fn expand_enum(
    ident: &Ident,
    command_name: LitStr,
    data_enum: DataEnum,
) -> SynResult<proc_macro2::TokenStream> {
    let mut parse_arms = Vec::new();
    let mut path_entries = Vec::new();
    let mut greedy_assertions = Vec::new();

    for variant in data_enum.variants {
        let variant_ident = variant.ident;
        let fields = CommandFields::from_fields(variant.fields)?;
        let field_parse = FieldParse::new(fields.fields(), ident)?;

        greedy_assertions.extend(field_parse.greedy_assertions);

        let constructor = match &fields {
            CommandFields::Unnamed(_) => {
                let values = &field_parse.tuple_values;
                quote! {
                    Self::#variant_ident(#(#values),*)
                }
            }
            CommandFields::Named(_) => {
                let values = &field_parse.named_values;
                quote! {
                    Self::#variant_ident { #(#values),* }
                }
            }
            CommandFields::Unit => quote! {
                Self::#variant_ident
            },
        };

        let raw_bindings = &field_parse.raw_bindings;
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

        let segments = &field_parse.segments;
        path_entries.push(quote! {
            ::temper_command_infra::CommandPath::new(#command_name, vec![#(#segments),*])
        });
    }

    let parse_body = quote! {
        let mut __best_error: Option<::temper_command_infra::ParseError> = None;

        #(#parse_arms)*

        Err(__best_error.unwrap_or_else(|| {
            ::temper_command_infra::ParseError::expected(__reader.cursor(), "command variant")
        }))
    };

    Ok(expand_command_impl(
        ident,
        command_name,
        parse_body,
        path_entries,
        greedy_assertions,
    ))
}

fn expand_struct(
    ident: &Ident,
    command_name: LitStr,
    data_struct: DataStruct,
) -> SynResult<proc_macro2::TokenStream> {
    let fields = CommandFields::from_fields(data_struct.fields)?;
    let field_parse = FieldParse::new(fields.fields(), ident)?;
    let raw_bindings = &field_parse.raw_bindings;

    let constructor = match &fields {
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
    };

    let parse_body = quote! {
        #(#raw_bindings)*
        __reader.expect_end()?;
        Ok(#constructor)
    };

    let segments = &field_parse.segments;
    let path_entries = vec![quote! {
        ::temper_command_infra::CommandPath::new(#command_name, vec![#(#segments),*])
    }];

    Ok(expand_command_impl(
        ident,
        command_name,
        parse_body,
        path_entries,
        field_parse.greedy_assertions,
    ))
}

fn expand_command_impl(
    ident: &Ident,
    command_name: LitStr,
    parse_body: proc_macro2::TokenStream,
    path_entries: Vec<proc_macro2::TokenStream>,
    greedy_assertions: Vec<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
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
                vec![#(#path_entries),*]
            }
        }

        #registration
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

enum CommandFields {
    Unnamed(Vec<CommandField>),
    Named(Vec<CommandField>),
    Unit,
}

impl CommandFields {
    fn from_fields(fields: Fields) -> SynResult<Self> {
        match fields {
            Fields::Unnamed(fields) => Ok(Self::Unnamed(
                fields.unnamed.into_iter().map(CommandField::from).collect(),
            )),
            Fields::Named(fields) => Ok(Self::Named(
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
            )),
            Fields::Unit => Ok(Self::Unit),
        }
    }

    fn fields(&self) -> &[CommandField] {
        match self {
            CommandFields::Unnamed(fields) | CommandFields::Named(fields) => fields,
            CommandFields::Unit => &[],
        }
    }
}

struct FieldParse {
    raw_bindings: Vec<proc_macro2::TokenStream>,
    tuple_values: Vec<proc_macro2::TokenStream>,
    named_values: Vec<proc_macro2::TokenStream>,
    segments: Vec<proc_macro2::TokenStream>,
    greedy_assertions: Vec<proc_macro2::TokenStream>,
}

impl FieldParse {
    fn new(fields: &[CommandField], _ident: &Ident) -> SynResult<Self> {
        let last_field_idx = fields.len().saturating_sub(1);
        let mut raw_bindings = Vec::new();
        let mut tuple_values = Vec::new();
        let mut named_values = Vec::new();
        let mut segments = Vec::new();
        let mut greedy_assertions = Vec::new();

        for (idx, command_field) in fields.iter().enumerate() {
            let arg_name = arg_name(command_field)?;
            let field = &command_field.field;
            let ty = &field.ty;
            let raw_ident = format_ident!("__raw_{idx}");

            raw_bindings.push(quote! {
                let #raw_ident = <#ty as ::temper_command_infra::CommandArg>::recognize(__reader)?;
            });

            tuple_values.push(quote! {
                <#ty as ::temper_command_infra::CommandArg>::parse(#raw_ident)?
            });

            if let Some(field_ident) = &command_field.ident {
                named_values.push(quote! {
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

        Ok(Self {
            raw_bindings,
            tuple_values,
            named_values,
            segments,
            greedy_assertions,
        })
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
