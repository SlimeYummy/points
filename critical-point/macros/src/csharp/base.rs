use anyhow::Result;
use std::collections::HashMap;
use std::ops::AddAssign;

//
// Types
//

pub enum TypeIn {
    Primitive(TypePrimitive),
    Generic(TypeGeneric),
}

impl TypeIn {
    pub fn new_primitive(name: &str) -> TypeIn {
        TypeIn::Primitive(TypePrimitive { name: name.into() })
    }

    pub fn new_generic(name: &str, param_count: usize) -> TypeIn {
        TypeIn::Generic(TypeGeneric {
            name: name.into(),
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

#[derive(Debug)]
pub enum TypeOut {
    Value(TypeValue),
    Reference(TypeReference),
}

impl TypeOut {
    pub fn new_value(rs_name: &str, cs_name: &str, size: usize, align: usize) -> TypeOut {
        TypeOut::Value(TypeValue {
            rs_name: rs_name.into(),
            cs_name: cs_name.into(),
            is_primitive: false,
            size,
            align,
        })
    }

    #[allow(dead_code)]
    pub fn new_primitive(rs_name: &str, cs_name: &str, size: usize, align: usize) -> TypeOut {
        TypeOut::Value(TypeValue {
            rs_name: rs_name.into(),
            cs_name: cs_name.into(),
            is_primitive: true,
            size,
            align,
        })
    }

    pub fn new_reference(rs_name: &str, size: usize, align: usize) -> TypeOut {
        TypeOut::Reference(TypeReference {
            rs_name: rs_name.into(),
            is_trait: false,
            size,
            align,
        })
    }

    pub fn new_trait(rs_name: &str) -> TypeOut {
        TypeOut::Reference(TypeReference {
            rs_name: rs_name.into(),
            is_trait: true,
            size: 16,
            align: 8,
        })
    }

    pub fn size(&self) -> usize {
        match self {
            TypeOut::Value(t) => t.size,
            TypeOut::Reference(t) => t.size,
        }
    }

    pub fn set_size(&mut self, size: usize) {
        match self {
            TypeOut::Value(t) => t.size = size,
            TypeOut::Reference(t) => t.size = size,
        }
    }

    pub fn align(&self) -> usize {
        match self {
            TypeOut::Value(t) => t.align,
            TypeOut::Reference(t) => t.align,
        }
    }

    pub fn set_align(&mut self, align: usize) {
        match self {
            TypeOut::Value(t) => t.align = align,
            TypeOut::Reference(t) => t.align = align,
        }
    }
}

#[derive(Debug)]
pub struct TypeValue {
    #[allow(dead_code)]
    pub rs_name: String,
    pub cs_name: String,
    pub is_primitive: bool,
    pub size: usize,
    pub align: usize,
}

#[derive(Debug)]
pub struct TypeReference {
    pub rs_name: String,
    pub is_trait: bool,
    pub size: usize,
    pub align: usize,
}

//
// Base
//

pub struct BaseMeta {
    #[allow(dead_code)]
    pub rs_base_name: String,
    pub rs_trait_name: String,
    pub rs_derives: Vec<String>,
    pub code: String,
}

impl BaseMeta {
    pub fn new(rs_base_name: &str, rs_trait_name: &str) -> BaseMeta {
        BaseMeta {
            rs_base_name: rs_base_name.into(),
            rs_trait_name: rs_trait_name.into(),
            rs_derives: Vec::new(),
            code: String::new(),
        }
    }
}

//
// Task
//

pub trait GenerateTask: Send + Sync {
    // fn name(&self) -> &str;
    fn generate(&self, ctx: &GenerateContext<'_>) -> Result<String>;
    fn generate_base(&self, _ctx: &GenerateContext<'_>) -> Result<(String, String)> {
        Ok((String::new(), String::new()))
    }
}

pub struct GenerateContext<'t> {
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

    pub fn join(&self) -> String {
        self.0.join("\r\n")
    }
}

impl<T: AsRef<str>> AddAssign<T> for Lines {
    fn add_assign(&mut self, rhs: T) {
        self.push(rhs.as_ref().into());
    }
}

macro_rules! f {
    ($($arg:tt)*) => {
        format!($($arg)*)
    };
}
pub(crate) use f;
