use anyhow::{anyhow, Context, Result};
use case::CaseExt;
use quote::ToTokens;
use regex::Regex;
use std::cell::Cell;
use std::collections::HashMap;
use std::sync::LazyLock;
use syn::*;

use super::base::*;

pub fn parse_struct_out(
    input: &ItemStruct,
    consts: &HashMap<String, u32>,
) -> Result<(String, String, Box<dyn GenerateTask>, LayoutTask, TypeOut)> {
    let repr = extract_attr_raw(&input.attrs, "repr")?;
    if repr != "C" {
        return Err(anyhow!("CsOut must repr C"));
    }

    let rs_name = input.ident.to_string();
    let args = extract_attr_args(&input.attrs, "cs_attr")?;
    let is_ref = args.iter().any(|x| x == "Ref");
    let is_value = args.iter().any(|x| x == "Value") || !is_ref;
    let is_partial = args.iter().any(|x| x == "Partial");
    let mut task = Box::new(TaskStructOut {
        rs_name: rs_name.clone(),
        fields: Vec::with_capacity(input.fields.len()),
        is_value,
        is_partial,
    });

    let mut base = String::new();
    for (idx, fd) in input.fields.iter().enumerate() {
        let field = match fd.ident.as_ref() {
            Some(ident) => ident.to_string(),
            None => return Err(anyhow!("Empty field name not supported")),
        };

        if is_value {
            match &fd.ty {
                Type::Path(path) => {
                    match parse_type_path_out(path)? {
                        ParsedPathOut::Type(rs_type) if rs_type != "String" => {
                            task.fields.push(FieldOut::Type { field, rs_type });
                        }
                        x @ _ => return Err(anyhow!("Not supported type {:?}", x)),
                    };
                }
                _ => return Err(anyhow!("Not supported type")),
            }
        } else {
            match &fd.ty {
                Type::Path(path) => {
                    match parse_type_path_out(path)? {
                        ParsedPathOut::Type(rs_type) => {
                            if idx == 0 && field == "_base" {
                                base = rs_type.clone();
                            }
                            if rs_type == "String" {
                                task.fields.push(FieldOut::String { field });
                            }
                            else {
                                task.fields.push(FieldOut::Type { field, rs_type });
                            }
                        }
                        ParsedPathOut::Reference(rs_type, ref_type) => {
                            task.fields.push(FieldOut::Reference {
                                field,
                                rs_type,
                                ref_type,
                            });
                        }
                        ParsedPathOut::VecReference(rs_type, ref_type) => {
                            task.fields.push(FieldOut::VecReference {
                                field,
                                rs_type,
                                ref_type,
                            });
                        }
                        ParsedPathOut::Generic(rs_type, args) => {
                            if rs_type == "Vec" && args.len() == 1 {
                                task.fields.push(FieldOut::Vec {
                                    field,
                                    rs_type: args[0].clone(),
                                });
                            }
                            else {
                                return Err(anyhow!("Unknown generic type ({})", rs_type));
                            }
                        }
                    };
                }
                Type::Array(array) => {
                    match parse_type_array(array, &consts)? {
                        ParsedArray::Type(rs_type) => {
                            task.fields.push(FieldOut::Type { field, rs_type });
                        }
                        ParsedArray::Array(rs_type, len) => {
                            task.fields.push(FieldOut::Array { field, rs_type, len });
                        }
                    };
                }
                _ => return Err(anyhow!("Not supported type")),
            }
        }
    }

    let layout_task = LayoutTask::new(&rs_name, task.fields.clone());
    if task.is_value {
        Ok((
            rs_name.clone(),
            base,
            task,
            layout_task,
            TypeOut::new_value(&rs_name, &rs_name, usize::MAX, usize::MAX),
        ))
    }
    else {
        Ok((
            rs_name.clone(),
            base,
            task,
            layout_task,
            TypeOut::new_reference(&rs_name, usize::MAX, usize::MAX),
        ))
    }
}

#[derive(Debug)]
enum ParsedPathOut {
    Type(String),
    Reference(String, ReferenceType),
    VecReference(String, ReferenceType),
    Generic(String, Vec<String>),
    // ArrayVec(String, String),
}

static RE_COMMON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^\w+$"#).unwrap());
static RE_GENERIC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^(\w+)\s*<(\s*(\w+)\s*,)*\s*(\w+)\s*>$"#).unwrap());
static RE_BOX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^Box < ((?:dyn )?\w+) >$"#).unwrap());
static RE_ARC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^Arc < ((?:dyn )?\w+) >$"#).unwrap());
static RE_VEC_BOX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^Vec < Box < ((?:dyn )?\w+) > >$"#).unwrap());
static RE_VEC_ARC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^Vec < Arc < ((?:dyn )?\w+) > >$"#).unwrap());
// static RE_ARRAY_VEC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^ArrayVec < \[(\w+); (\w+)\] >$"#).unwrap());

fn parse_type_path_out(path: &TypePath) -> Result<ParsedPathOut> {
    let code = path.to_token_stream().to_string();
    if RE_COMMON.is_match(&code) {
        Ok(ParsedPathOut::Type(code))
    }
    else if let Some(caps) = RE_BOX.captures(&code) {
        let name = caps.get(1).unwrap().as_str().into();
        Ok(ParsedPathOut::Reference(name, ReferenceType::Box))
    }
    else if let Some(caps) = RE_ARC.captures(&code) {
        let name = caps.get(1).unwrap().as_str().into();
        Ok(ParsedPathOut::Reference(name, ReferenceType::Arc))
    }
    else if let Some(caps) = RE_VEC_BOX.captures(&code) {
        let name = caps.get(1).unwrap().as_str().into();
        Ok(ParsedPathOut::VecReference(name, ReferenceType::Box))
    }
    else if let Some(caps) = RE_VEC_ARC.captures(&code) {
        let name = caps.get(1).unwrap().as_str().into();
        Ok(ParsedPathOut::VecReference(name, ReferenceType::Arc))
    }
    else if let Some(caps) = RE_GENERIC.captures(&code) {
        let name = caps.get(1).unwrap().as_str().into();
        let args = caps
            .iter()
            .skip(2)
            .filter_map(|m| m.map(|m| m.as_str().into()))
            .collect();
        Ok(ParsedPathOut::Generic(name, args))
    }
    else {
        Err(anyhow::anyhow!("Unsupported type: {}", code))
    }
}

#[derive(Debug, Clone)]
enum FieldOut {
    Type {
        field: String,
        rs_type: String,
    },
    Array {
        field: String,
        rs_type: String,
        len: u32,
    },
    Vec {
        field: String,
        rs_type: String,
    },
    String {
        field: String,
    },
    // Box<dyn _> | Arc<dyn _>
    Reference {
        field: String,
        rs_type: String,
        #[allow(dead_code)]
        ref_type: ReferenceType,
    },
    // Vec<Box<dyn _>> | Vec<Arc<dyn _>>
    VecReference {
        field: String,
        rs_type: String,
        ref_type: ReferenceType,
    },
}

impl FieldOut {
    fn field(&self) -> &str {
        match self {
            FieldOut::Type { field, .. } => field,
            FieldOut::Array { field, .. } => field,
            FieldOut::Vec { field, .. } => field,
            FieldOut::String { field } => field,
            FieldOut::Reference { field, .. } => field,
            FieldOut::VecReference { field, .. } => field,
        }
    }

    fn rs_type(&self) -> &str {
        match self {
            FieldOut::Type { rs_type, .. } => rs_type,
            FieldOut::Array { rs_type, .. } => rs_type,
            FieldOut::Vec { rs_type, .. } => rs_type,
            FieldOut::String { .. } => "",
            FieldOut::Reference { rs_type, .. } => rs_type,
            FieldOut::VecReference { rs_type, .. } => rs_type,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ReferenceType {
    Box,
    Arc,
}

#[derive(Debug)]
struct TaskStructOut {
    rs_name: String,
    fields: Vec<FieldOut>,
    is_value: bool,
    is_partial: bool,
}

impl GenerateTask for TaskStructOut {
    fn generate(&self, ctx: &GenerateContext<'_>) -> Result<String> {
        let mut code = self
            .gen_value_type(ctx)
            .with_context(|| "TaskStructOut::gen_value_type()")?;
        if !self.is_value {
            code += "\r\n";
            if let Some(meta) = ctx.bases.get(&self.rs_name) {
                code += &self
                    .gen_ref_type_base(ctx, meta)
                    .with_context(|| "TaskStructOut::gen_ref_type_base()")?;
            }
            else {
                code += &self
                    .gen_ref_type(ctx)
                    .with_context(|| "TaskStructOut::gen_ref_type()")?;
            }
        }
        Ok(code)
    }

    fn generate_base(&self, ctx: &GenerateContext<'_>) -> Result<(String, String)> {
        if ctx.bases.contains_key(&self.rs_name) {
            let mut ls = Lines::new(self.fields.len());
            self.gen_ref_fields(ctx, &mut ls, "@@@@_base.")?;
            return Ok((self.rs_name.clone(), ls.join()));
        }
        Ok((String::new(), String::new()))
    }
}

impl TaskStructOut {
    fn cs_name(&self) -> String {
        if self.is_value {
            self.rs_name.clone()
        }
        else {
            format!("Rs{}", self.rs_name)
        }
    }

    fn gen_value_type(&self, ctx: &GenerateContext<'_>) -> Result<String> {
        let zelf = ctx
            .types_out
            .get(&self.rs_name)
            .ok_or_else(|| anyhow!("Self type not found {}", self.rs_name))?;

        let mut ls = Lines::new(self.fields.len() * 2);
        ls += f!("  [StructLayout(LayoutKind.Explicit, Size = {})]", zelf.size());
        ls += match self.is_partial {
            false => f!("  public unsafe struct {} {{", self.cs_name()),
            true => f!("  public unsafe partial struct {} {{", self.cs_name()),
        };
        ls += f!("    public const int SIZE = {};", zelf.size());
        ls += f!("    public const int ALIGN = {};", zelf.align());

        let mut calculator = LayoutCalculator::default();
        for (idx, field) in self.fields.iter().enumerate() {
            let typ = ctx
                .types_out
                .get(field.rs_type())
                .ok_or_else(|| anyhow!("Unknown type {} in {}", field.rs_type(), self.rs_name))?;

            if idx == 0 && field.field() == "_base" {
                let offset = calculator.add_field(typ.size(), typ.align(), 1);
                match ctx.bases.get(field.rs_type()) {
                    Some(_) => ls += f!("    [FieldOffset({0})] public Rs{1} _base;", offset, field.rs_type()),
                    None => return Err(anyhow!("Unknown base type {}", field.rs_type())),
                };
                continue;
            }

            match field {
                FieldOut::Type { field, rs_type } => {
                    let offset = calculator.add_field(typ.size(), typ.align(), 1);
                    match typ {
                        TypeOut::Value(v) => {
                            if v.cs_name == "bool" {
                                ls += f!(
                                    "    [FieldOffset({0}), MarshalAs(UnmanagedType.U1)] public bool {1};",
                                    offset,
                                    field
                                );
                            }
                            else {
                                ls += f!("    [FieldOffset({0})] public {1} {2};", offset, v.cs_name, field);
                            }
                        }
                        _ => return Err(anyhow!("Value type ({}) not found", rs_type)),
                    };
                }
                FieldOut::Array { field, rs_type, len } => {
                    match typ {
                        TypeOut::Value(v) => {
                            if v.is_primitive {
                                let offset = calculator.add_field(typ.size(), typ.align(), *len as usize);
                                ls += f!(
                                    "    [FieldOffset({0})] private fixed {1} {2}[{3}];",
                                    offset,
                                    v.cs_name,
                                    field,
                                    len
                                );
                            }
                            else {
                                for idx in 0..*len {
                                    let offset = calculator.add_field(typ.size(), typ.align(), 1);
                                    ls += f!(
                                        "    [FieldOffset({0})] private {1} {2}_{3};",
                                        offset,
                                        v.cs_name,
                                        field,
                                        idx
                                    );
                                }
                                ls += f!(
                                    "    public RefArrayVal<{0}> {1} => new RefArrayVal<{0}>(ref {1}_0, {2});",
                                    v.cs_name,
                                    field,
                                    len
                                );
                            }
                        }
                        _ => return Err(anyhow!("Value type ({}) not found", rs_type)),
                    };
                }
                FieldOut::Vec { field, rs_type } => {
                    let offset = calculator.add_field(24, 8, 1);
                    match typ {
                        TypeOut::Value(v) => {
                            ls += f!(
                                "    [FieldOffset({0})] private RsVec<{1}> _{2};",
                                offset,
                                v.cs_name,
                                field
                            );
                            ls += f!(
                                "    public RefVecVal<{0}> {1} => new RefVecVal<{0}>(_{1});",
                                v.cs_name,
                                field
                            );
                        }
                        TypeOut::Reference(r) if !r.is_trait => {
                            ls += f!(
                                "    [FieldOffset({0})] private RsVec<Rs{1}> _{2};",
                                offset,
                                r.rs_name,
                                field
                            );
                            ls += f!("    public RefVecRs{0} {1} => new RefVecRs{0}(_{1});", r.rs_name, field);
                        }
                        _ => return Err(anyhow!("Type ({}) not found", rs_type)),
                    };
                }
                FieldOut::String { field } => {
                    let offset = calculator.add_field(24, 8, 1);
                    ls += f!("    [FieldOffset({0})] private RsString _{1};", offset, field);
                    ls += f!("    public RefRsString {0} => new RefRsString(_{0});", field);
                }
                FieldOut::VecReference {
                    field,
                    rs_type,
                    ref_type,
                } => {
                    let offset = calculator.add_field(24, 8, 1);
                    match typ {
                        TypeOut::Reference(r) => {
                            if r.is_trait {
                                if *ref_type == ReferenceType::Box {
                                    ls += f!(
                                        "    [FieldOffset({0})] private RsVec<RsBoxDyn{1}> _{2};",
                                        offset,
                                        r.rs_name,
                                        field
                                    );
                                    ls += f!(
                                        "    public RefVecBox{0} {1} => new RefVecBox{0}(_{1});",
                                        r.rs_name,
                                        field
                                    );
                                }
                                else {
                                    ls += f!(
                                        "    [FieldOffset({0})] private RsVec<RsArcDyn{1}> _{2};",
                                        offset,
                                        r.rs_name,
                                        field
                                    );
                                    ls += f!(
                                        "    public RefVecArc{0} {1} => new RefVecArc{0}(_{1});",
                                        r.rs_name,
                                        field
                                    );
                                }
                            }
                            else {
                                if *ref_type == ReferenceType::Box {
                                    ls += f!(
                                        "    [FieldOffset({0})] private RsVec<RsBox{1}> _{2};",
                                        offset,
                                        r.rs_name,
                                        field
                                    );
                                    ls += f!(
                                        "    public RefVecBox{0} {1} => new RefVecBox{0}(_{1});",
                                        r.rs_name,
                                        field
                                    );
                                }
                                else {
                                    ls += f!(
                                        "    [FieldOffset({0})] private RsVec<RsArc{1}> _{2};",
                                        offset,
                                        r.rs_name,
                                        field
                                    );
                                    ls += f!(
                                        "    public RefVecArc{0} {1} => new RefVecArc{0}(_{1});",
                                        r.rs_name,
                                        field
                                    );
                                }
                            }
                        }
                        _ => return Err(anyhow!("Reference type ({}) not found", rs_type)),
                    }
                }
                _ => return Err(anyhow!("Type ({}) not support", field.rs_type())),
            };
        }
        ls += f!("  }}\r\n");
        Ok(ls.join())
    }
}