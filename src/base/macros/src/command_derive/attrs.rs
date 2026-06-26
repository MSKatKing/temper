use syn::{Attribute, LitStr, Result as SynResult};

pub enum CommandKind {
    Root(LitStr),
    Subcommand,
}

pub enum VariantPrefix {
    Literal(LitStr),
    Subcommand(LitStr),
}

pub fn command_kind(attrs: &[Attribute]) -> SynResult<CommandKind> {
    for attr in attrs {
        if !attr.path().is_ident("command") {
            continue;
        }

        if let Ok(name) = attr.parse_args::<LitStr>() {
            return Ok(CommandKind::Root(name));
        }

        let mut name = None;
        let mut subcommand = false;

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("name") {
                name = Some(meta.value()?.parse::<LitStr>()?);
                Ok(())
            } else if meta.path.is_ident("subcommand") {
                subcommand = true;
                Ok(())
            } else {
                Err(meta.error("unsupported command option"))
            }
        })?;

        return match (name, subcommand) {
            (Some(name), false) => Ok(CommandKind::Root(name)),
            (None, true) => Ok(CommandKind::Subcommand),
            (Some(_), true) => Err(syn::Error::new_spanned(
                attr,
                "command cannot be both named and a subcommand",
            )),
            (None, false) => Err(syn::Error::new_spanned(
                attr,
                "expected #[command(\"name\")], #[command(name = \"name\")], or #[command(subcommand)]",
            )),
        };
    }

    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "missing #[command(...)] attribute",
    ))
}

pub fn variant_prefix(attrs: &[Attribute]) -> SynResult<Option<VariantPrefix>> {
    let mut prefix = None;

    for attr in attrs {
        let next = if attr.path().is_ident("literal") {
            Some(VariantPrefix::Literal(attr.parse_args::<LitStr>()?))
        } else if attr.path().is_ident("subcommand") {
            Some(VariantPrefix::Subcommand(attr.parse_args::<LitStr>()?))
        } else {
            None
        };

        let Some(next) = next else {
            continue;
        };

        if prefix.is_some() {
            return Err(syn::Error::new_spanned(
                attr,
                "command variants can only have one #[literal(...)] or #[subcommand(...)] attribute",
            ));
        }

        prefix = Some(next);
    }

    Ok(prefix)
}
