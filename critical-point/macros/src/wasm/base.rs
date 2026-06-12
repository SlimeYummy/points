use syn::punctuated::Punctuated;
use syn::{Attribute, Ident, Meta, Token};

pub const BUILTIN_DERIVES: &[&str] = &[
    "Debug",
    "Clone",
    "Copy",
    "PartialEq",
    "Eq",
    "PartialOrd",
    "Ord",
    "Hash",
    "Default",
];

pub fn collect_builtin_derives(attrs: &[Attribute]) -> Vec<Ident> {
    let mut result = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("derive") {
            continue;
        }

        let Ok(parsed) = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            continue;
        };

        for meta in parsed {
            if let Some(ident) = get_meta_ident(&meta) {
                if BUILTIN_DERIVES.contains(&ident.to_string().as_str()) {
                    result.push(ident.clone());
                }
            }
        }
    }
    result
}

pub fn get_meta_ident(meta: &Meta) -> Option<&Ident> {
    match meta {
        Meta::Path(path) => path.get_ident(),
        _ => None,
    }
}
