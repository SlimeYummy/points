use anyhow::{anyhow, Result};
use quote::ToTokens;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use syn::*;

use super::base::*;

pub fn parse_struct_in(
    input: &ItemStruct,
    consts: &HashMap<String, u32>,
) -> Result<(String, Box<dyn GenerateTask>, TypeIn)> {
    let rs_name = input.ident.to_string();
    let args = extract_attr_args(&input.attrs, "cs_attr")?;
    let is_class = args.iter().any(|x| x == "Class");
    let is_struct = args.iter().any(|x| x == "Struct");
    let is_partial = args.iter().any(|x| x == "Partial");
    let mut task = Box::new(TaskStructIn {
        cs_name: rs_name.clone(),
        fields: Vec::with_capacity(input.fields.len()),
        is_struct: is_struct || !is_class,
        is_partial: is_partial,
    });

    for fd in input.fields.iter() {
        let field = match fd.ident.as_ref() {
            Some(ident) => ident.to_string(),
            None => return Err(anyhow!("Empty field name not supported")),
        };

        match &fd.ty {
            Type::Path(path) => {
                match parse_type_path_in(path)? {
                    ParsedPathIn::Type(rs_type) => {
                        task.fields.push(FieldIn::Type { field, rs_type });
                    }
                    ParsedPathIn::Generic(rs_type, rs_sub_types) => {
                        task.fields.push(FieldIn::Generic {
                            field,
                            rs_type,
                            rs_sub_types,
                        });
                    }
                };
            }
            Type::Array(array) => {
                match parse_type_array(array, &consts)? {
                    ParsedArray::Type(rs_type) => {
                        task.fields.push(FieldIn::Type { field, rs_type });
                    }
                    ParsedArray::Array(rs_type, _) => {
                        task.fields.push(FieldIn::Array { field, rs_type });
                    }
                };
            }
            _ => return Err(anyhow!("Not supported type")),
        }
    }

    let type_in = TypeIn::new_primitive(&rs_name);
    Ok((rs_name, task, type_in))
}

static RE_COMMON: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^\w+$"#).unwrap());
static RE_GENERIC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^(\w+)\s*<(\s*(\w+)\s*,)*\s*(\w+)\s*>$"#).unwrap());

#[derive(Debug)]
enum ParsedPathIn {
    Type(String),
    Generic(String, Vec<String>),
}

fn parse_type_path_in(path: &TypePath) -> Result<ParsedPathIn> {
    let code = path.to_token_stream().to_string();
    if RE_COMMON.is_match(&code) {
        Ok(ParsedPathIn::Type(code))
    }
    else if let Some(caps) = RE_GENERIC.captures(&code) {
        let name = caps.get(1).unwrap().as_str().into();
        let args = caps
            .iter()
            .skip(2)
            .filter_map(|m| m.map(|m| m.as_str().into()))
            .collect();
        Ok(ParsedPathIn::Generic(name, args))
    }
    else {
        Err(anyhow::anyhow!("Unsupported type: {}", code))
    }
}

#[derive(Debug)]
enum FieldIn {
    Type {
        field: String,
        rs_type: String,
    },
    Generic {
        field: String,
        rs_type: String,
        rs_sub_types: Vec<String>,
    },
    Array {
        field: String,
        rs_type: String,
    },
}

impl FieldIn {
    // fn field(&self) -> &str {
    //     match self {
    //         FieldIn::Type { field, .. } => field,
    //         FieldIn::Generic { field, .. } => field,
    //         FieldIn::Array { field, .. } => field,
    //     }
    // }

    fn rs_type(&self) -> &str {
        match self {
            FieldIn::Type { rs_type, .. } => rs_type,
            FieldIn::Generic { rs_type, .. } => rs_type,
            FieldIn::Array { rs_type, .. } => rs_type,
        }
    }
}

#[derive(Debug)]
struct TaskStructIn {
    cs_name: String,
    fields: Vec<FieldIn>,
    is_struct: bool,
    is_partial: bool,
}

impl GenerateTask for TaskStructIn {
    fn generate(&self, ctx: &GenerateContext<'_>) -> Result<String> {
        let mut ls = Lines::new(self.fields.len());
        ls += f!("  [MessagePackObject(keyAsPropertyName: true)]");
        ls += match (self.is_struct, self.is_partial) {
            (true, false) => f!("  public struct {} {{", self.cs_name),
            (true, true) => f!("  public partial struct {} {{", self.cs_name),
            (false, false) => f!("  public class {} {{", self.cs_name),
            (false, true) => f!("  public partial class {} {{", self.cs_name),
        };

        for field in &self.fields {
            let typ = ctx
                .types_in
                .get(field.rs_type())
                .ok_or_else(|| anyhow!("Unknown type {} in {}", field.rs_type(), self.cs_name))?;

            match field {
                FieldIn::Type { field, rs_type } => {
                    match typ {
                        TypeIn::Primitive(p) => {
                            ls += f!("    public {} {};", p.name, field);
                        }
                        _ => return Err(anyhow!("Primitive ({}) not found", rs_type)),
                    };
                }
                FieldIn::Array { field, rs_type } => {
                    match typ {
                        TypeIn::Primitive(p) => {
                            ls += f!("    [Key(\"{}\")]", field);
                            ls += f!("    public List<{}> {};", p.name, field);
                        }
                        _ => return Err(anyhow!("Primitive ({}) not found", rs_type)),
                    };
                }
                FieldIn::Generic {
                    field,
                    rs_type,
                    rs_sub_types,
                } => {
                    match typ {
                        TypeIn::Generic(g) => {
                            if g.param_count != rs_sub_types.len() {
                                return Err(anyhow!("Generic ({}) param count mismatch", rs_type));
                            }
                            let mut cs_sub_types = Vec::with_capacity(rs_sub_types.len());
                            for sub in rs_sub_types {
                                match ctx.types_in.get(sub) {
                                    Some(TypeIn::Primitive(p)) => cs_sub_types.push(p.name.clone()),
                                    _ => return Err(anyhow!("Primitive ({}) not found", sub)),
                                };
                            }
                            ls += f!("    [Key(\"{}\")]", field);
                            ls += f!("    public {}<{}> {};", g.name, cs_sub_types.join(", "), field);
                        }
                        _ => return Err(anyhow!("Generic ({}) not found", rs_type)),
                    };
                }
            };
        }
        ls += f!("  }}\r\n");
        Ok(ls.join())
    }
}
