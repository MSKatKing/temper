use quote::{format_ident, quote, quote_spanned};
use syn::{spanned::Spanned, Field, Fields, Ident, LitStr, Result as SynResult, Type};

pub enum CommandFields {
    Unnamed(Vec<CommandField>),
    Named(Vec<CommandField>),
    Unit,
}

impl CommandFields {
    pub fn from_fields(fields: Fields) -> SynResult<Self> {
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

    pub fn fields(&self) -> &[CommandField] {
        match self {
            CommandFields::Unnamed(fields) | CommandFields::Named(fields) => fields,
            CommandFields::Unit => &[],
        }
    }

    pub fn single_unnamed_type(&self) -> SynResult<&Type> {
        let CommandFields::Unnamed(fields) = self else {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "subcommand variants must contain exactly one unnamed field",
            ));
        };

        if fields.len() != 1 {
            return Err(syn::Error::new(
                fields
                    .first()
                    .map(|field| field.field.span())
                    .unwrap_or_else(proc_macro2::Span::call_site),
                "subcommand variants must contain exactly one unnamed field",
            ));
        }

        Ok(&fields[0].field.ty)
    }
}

pub struct FieldParse {
    pub raw_bindings: Vec<proc_macro2::TokenStream>,
    pub tuple_values: Vec<proc_macro2::TokenStream>,
    pub named_values: Vec<proc_macro2::TokenStream>,
    pub segments: Vec<proc_macro2::TokenStream>,
    pub greedy_assertions: Vec<proc_macro2::TokenStream>,
}

impl FieldParse {
    pub fn new(fields: &[CommandField]) -> SynResult<Self> {
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

pub struct CommandField {
    ident: Option<Ident>,
    field: Field,
}

impl From<Field> for CommandField {
    fn from(field: Field) -> Self {
        Self { ident: None, field }
    }
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
