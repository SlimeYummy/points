use anyhow::Result;
use quote::ToTokens;
use std::collections::HashMap;
use std::ops::AddAssign;
use syn::punctuated::Punctuated;
use syn::{Attribute, Meta, Token};

//
// Types
//

pub enum TypeIn {
    Primitive(TypePrimitive),
    Generic(TypeGeneric),
}

impl TypeIn {
    pub fn new_primitive(name: &str) -> TypeIn {
        TypeIn::Primitive(TypePrimitive { name: name.to_string() })
    }

    pub fn new_generic(name: &str, param_count: usize) -> TypeIn {
        TypeIn::Generic(TypeGeneric {
            name: name.to_string(),
            param_count,
        })
    }
}

pub struct TypePrimitive {
    pub name: String,
}

pub struct TypeGeneric {
    pub name: String,
    pub param_count: usize,
}

pub enum TypeOut {
    Value(TypeValue),
    Reference(TypeReference),
    Placeholder(TypePlaceholder),
}

impl TypeOut {
    pub fn new_value(rs_name: &str, cs_name: &str) -> TypeOut {
        TypeOut::Value(TypeValue {
            rs_name: rs_name.to_string(),
            cs_name: cs_name.to_string(),
            is_primitive: false,
        })
    }

    pub fn new_primitive(rs_name: &str, cs_name: &str) -> TypeOut {
        TypeOut::Value(TypeValue {
            rs_name: rs_name.to_string(),
            cs_name: cs_name.to_string(),
            is_primitive: true,
        })
    }

    pub fn new_reference(rs_name: &str) -> TypeOut {
        TypeOut::Reference(TypeReference {
            rs_name: rs_name.to_string(),
            is_trait: false,
        })
    }

    pub fn new_trait(rs_name: &str) -> TypeOut {
        TypeOut::Reference(TypeReference {
            rs_name: rs_name.to_string(),
            is_trait: true,
        })
    }

    pub fn new_placeholder(rs_name: &str) -> TypeOut {
        TypeOut::Placeholder(TypePlaceholder {
            rs_name: rs_name.to_string(),
        })
    }
}

pub struct TypeValue {
    pub rs_name: String,
    pub cs_name: String,
    pub is_primitive: bool,
}

pub struct TypeReference {
    pub rs_name: String,
    pub is_trait: bool,
}

pub struct TypePlaceholder {
    pub rs_name: String,
}

//
// Base
//

pub struct BaseMeta {
    pub rs_base_name: String,
    pub rs_trait_name: String,
    pub rs_derives: Vec<String>,
    pub code: String,
}

impl BaseMeta {
    pub fn new(rs_base_name: &str, rs_trait_name: &str) -> BaseMeta {
        BaseMeta {
            rs_base_name: rs_base_name.to_string(),
            rs_trait_name: rs_trait_name.to_string(),
            rs_derives: Vec::new(),
            code: String::new(),
        }
    }
}

//
// Task
//

pub trait Task: Send + Sync {
    // fn name(&self) -> &str;
    fn gen(&self, ctx: &GenContext<'_>) -> Result<String>;
    fn gen_base(&self, _ctx: &GenContext<'_>) -> Result<(String, String)> {
        Ok((String::new(), String::new()))
    }
}

pub struct GenContext<'t> {
    pub types_in: &'t HashMap<String, TypeIn>,
    pub types_out: &'t HashMap<String, TypeOut>,
    pub bases: &'t HashMap<String, BaseMeta>,
}

//
// Lines
//

pub struct Lines(Vec<String>);

impl Lines {
    pub fn new(capacity: usize) -> Lines {
        Lines(Vec::with_capacity(capacity))
    }

    pub fn push(&mut self, line: String) {
        self.0.push(line);
    }

    pub fn concat(&mut self, lines: Lines) {
        self.0.extend(lines.0);
    }

    pub fn join(&self) -> String {
        self.0.join("\r\n")
    }
}

impl<T: AsRef<str>> AddAssign<T> for Lines {
    fn add_assign(&mut self, rhs: T) {
        self.push(rhs.as_ref().to_string());
    }
}

macro_rules! f {
    ($($arg:tt)*) => {
        format!($($arg)*)
    };
}
pub(crate) use f;

//
// helpers
//

pub fn extract_attr_raw(attrs: &[Attribute], name: &str) -> Result<String> {
    let mut raw = String::new();
    if let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident(name)) {
        attr.parse_nested_meta(|meta| {
            raw = meta.path.into_token_stream().to_string();
            Ok(())
        })?;
    }
    Ok(raw)
}

pub fn extract_attr_args(attrs: &[Attribute], name: &str) -> Result<Vec<String>> {
    let mut args = Vec::new();
    if let Some(attr) = attrs.iter().find(|attr| attr.path().is_ident(name)) {
        let nested = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)?;
        for meta in nested {
            args.push(meta.path().to_token_stream().to_string());
        }
    }
    Ok(args)
}
