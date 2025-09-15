use anyhow::{anyhow, Context, Result};
use case::CaseExt;
use quote::ToTokens;
use regex::Regex;
use std::collections::HashMap;
use std::sync::LazyLock;
use syn::*;

use super::base::*;

//
// Struct In
//

pub fn parse_struct_in(input: &ItemStruct, consts: &HashMap<String, u32>) -> Result<(String, Box<dyn Task>, TypeIn)> {
    let rs_name = input.ident.to_string();
    let args = extract_attr_args(&input.attrs, "cs_attr")?;
    let is_class = args.iter().any(|x| x == "Class");
    let is_struct = args.iter().any(|x| x == "Struct");
    let mut task = Box::new(TaskStructIn {
        cs_name: rs_name.clone(),
        fields: Vec::with_capacity(input.fields.len()),
        is_struct: is_struct || !is_class,
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
enum ParsedArray {
    Type(String),
    Array(String, u32),
}

fn parse_type_array(array: &TypeArray, consts: &HashMap<String, u32>) -> Result<ParsedArray> {
    let typ = array.elem.to_token_stream().to_string();
    let len = array.len.to_token_stream().to_string();
    let len: u32 = match consts.get(&len) {
        Some(c) => *c,
        None => len.parse()?,
    };
    if typ == "f32" && len == 2 {
        Ok(ParsedArray::Type("[f32; 2]".into()))
    }
    else {
        Ok(ParsedArray::Array(typ, len))
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
}

impl Task for TaskStructIn {
    fn gen(&self, ctx: &GenContext<'_>) -> Result<String> {
        let mut ls = Lines::new(self.fields.len());
        ls += f!("  [MessagePackObject(keyAsPropertyName: true)]");
        ls += match self.is_struct {
            true => f!("  public struct {} {{", self.cs_name),
            false => f!("  public class {} {{", self.cs_name),
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

//
// Struct Out
//

pub fn parse_struct_out(
    input: &ItemStruct,
    consts: &HashMap<String, u32>,
) -> Result<(String, String, Box<dyn Task>, TypeOut)> {
    let repr = extract_attr_raw(&input.attrs, "repr")?;
    if repr != "C" {
        return Err(anyhow!("CsOut must repr C"));
    }

    let rs_name = input.ident.to_string();
    let args = extract_attr_args(&input.attrs, "cs_attr")?;
    let is_ref = args.iter().any(|x| x == "Ref");
    let is_value = args.iter().any(|x| x == "Value");
    let mut task = Box::new(TaskStructOut {
        rs_name: rs_name.clone(),
        fields: Vec::with_capacity(input.fields.len()),
        is_value: is_value || !is_ref,
    });

    let mut base = String::new();
    for (idx, fd) in input.fields.iter().enumerate() {
        let field = match fd.ident.as_ref() {
            Some(ident) => ident.to_string(),
            None => return Err(anyhow!("Empty field name not supported")),
        };

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

    if task.is_value {
        Ok((rs_name.clone(), base, task, TypeOut::new_value(&rs_name, &rs_name)))
    }
    else {
        Ok((rs_name.clone(), base, task, TypeOut::new_reference(&rs_name)))
    }
}

#[derive(Debug)]
enum ParsedPathOut {
    Type(String),
    Reference(String, ReferenceType),
    VecReference(String, ReferenceType),
    Generic(String, Vec<String>),
}

static RE_BOX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^Box < ((?:dyn )?\w+) >$"#).unwrap());
static RE_ARC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^Arc < ((?:dyn )?\w+) >$"#).unwrap());
static RE_VEC_BOX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^Vec < Box < ((?:dyn )?\w+) > >$"#).unwrap());
static RE_VEC_ARC: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"^Vec < Arc < ((?:dyn )?\w+) > >$"#).unwrap());

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

#[derive(Debug)]
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
    Reference {
        field: String,
        rs_type: String,
        #[allow(dead_code)]
        ref_type: ReferenceType,
    },
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

#[derive(Debug, PartialEq)]
enum ReferenceType {
    Box,
    Arc,
}

#[derive(Debug)]
struct TaskStructOut {
    rs_name: String,
    fields: Vec<FieldOut>,
    is_value: bool,
}

impl Task for TaskStructOut {
    fn gen(&self, ctx: &GenContext<'_>) -> Result<String> {
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

    fn gen_base(&self, ctx: &GenContext<'_>) -> Result<(String, String)> {
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

    fn gen_value_type(&self, ctx: &GenContext<'_>) -> Result<String> {
        let mut ls = Lines::new(self.fields.len() * 2);
        ls += f!("  [StructLayout(LayoutKind.Sequential)]");
        ls += f!("  public unsafe struct {} {{", self.cs_name());

        for (idx, field) in self.fields.iter().enumerate() {
            if idx == 0 && field.field() == "_base" {
                match ctx.bases.get(field.rs_type()) {
                    Some(_) => ls += f!("    public Rs{} _base;", field.rs_type()),
                    None => return Err(anyhow!("Unknown base type {}", field.rs_type())),
                };
                continue;
            }

            let typ = ctx
                .types_out
                .get(field.rs_type())
                .ok_or_else(|| anyhow!("Unknown type {} in {}", field.rs_type(), self.rs_name))?;

            match field {
                FieldOut::Type { field, rs_type } => {
                    match typ {
                        TypeOut::Value(v) => {
                            if v.cs_name == "bool" {
                                ls += f!("    [MarshalAs(UnmanagedType.U1)] public bool {};", field);
                            }
                            else {
                                ls += f!("    public {0} {1};", v.cs_name, field);
                            }
                        }
                        _ => return Err(anyhow!("Value type ({}) not found", rs_type)),
                    };
                }
                FieldOut::Array { field, rs_type, len } => {
                    match typ {
                        TypeOut::Value(v) => {
                            if v.is_primitive {
                                ls += f!("    private fixed {} {}[{}];", v.cs_name, field, len);
                            }
                            else {
                                for idx in 0..*len {
                                    ls += f!("    private {} {}_{};", v.cs_name, field, idx);
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
                    match typ {
                        TypeOut::Value(v) => {
                            ls += f!("    private RsVec<{0}> _{1};", v.cs_name, field);
                            ls += f!(
                                "    public RefVecVal<{0}> {1} => new RefVecVal<{0}>(_{1});",
                                v.cs_name,
                                field
                            );
                        }
                        TypeOut::Reference(r) if !r.is_trait => {
                            ls += f!("    private RsVec<Rs{0}> _{1};", r.rs_name, field);
                            ls += f!("    public RefVecRs{0} {1} => new RefVecRs{0}(_{1});", r.rs_name, field);
                        }
                        _ => return Err(anyhow!("Type ({}) not found", rs_type)),
                    };
                }
                FieldOut::String { field } => {
                    ls += f!("    private RsString _{0};", field);
                    ls += f!("    public RefString {0} => new RefString(_{0});", field);
                }
                FieldOut::VecReference {
                    field,
                    rs_type,
                    ref_type,
                } => match typ {
                    TypeOut::Reference(r) => {
                        if r.is_trait {
                            if *ref_type == ReferenceType::Box {
                                ls += f!("    private RsVec<RsBoxDyn{0}> _{1};", r.rs_name, field);
                                ls += f!(
                                    "    public RefVecBox{0} {1} => new RefVecBox{0}(_{1});",
                                    r.rs_name,
                                    field
                                );
                            }
                            else {
                                ls += f!("    private RsVec<RsArcDyn{0}> _{1};", r.rs_name, field);
                                ls += f!(
                                    "    public RefVecArc{0} {1} => new RefVecArc{0}(_{1});",
                                    r.rs_name,
                                    field
                                );
                            }
                        }
                        else {
                            if *ref_type == ReferenceType::Box {
                                ls += f!("    private RsVec<RsBox{0}> _{1};", r.rs_name, field);
                                ls += f!(
                                    "    public RefVecBox{0} {1} => new RefVecBox{0}(_{1});",
                                    r.rs_name,
                                    field
                                );
                            }
                            else {
                                ls += f!("    private RsVec<RsArc{0}> _{1};", r.rs_name, field);
                                ls += f!(
                                    "    public RefVecArc{0} {1} => new RefVecArc{0}(_{1});",
                                    r.rs_name,
                                    field
                                );
                            }
                        }
                    }
                    _ => return Err(anyhow!("Reference type ({}) not found", rs_type)),
                },
                _ => return Err(anyhow!("Type ({}) not support", field.rs_type())),
            };
        }
        ls += f!("  }}\r\n");
        Ok(ls.join())
    }

    #[rustfmt::skip]
    fn gen_ref_type_base(&self, ctx: &GenContext<'_>, meta: &BaseMeta) -> Result<String> {
        let rs_name = &self.rs_name;
        let trait_name = &meta.rs_trait_name;
        let snake_case = trait_name.to_snake();
        let mut ls = Lines::new(self.fields.len());

        // RsBox
        ls += f!("  internal unsafe struct RsBox{} {{", trait_name);
        ls += f!("    private RsBoxDyn<Rs{}> _dyn;", rs_name);
        ls += f!("    internal Ref{0} MakeRef() => new Ref{0}(_dyn);", trait_name);
        ls += f!("    internal Box{0} MakeBox() => new Box{0}(_dyn);", trait_name);
        ls += f!("  }}\r\n");

        // Ref
        ls += f!("  public unsafe ref struct Ref{} {{", trait_name);
        ls += f!("    private RsBoxDyn<Rs{}> _dyn;", rs_name);
        self.gen_ref_fields(ctx, &mut ls, "_dyn.ptr->")?;
        ls += f!("");
        ls += f!("    internal Ref{}(RsBoxDyn<Rs{}> dyn) => _dyn = dyn;", trait_name, rs_name);
        ls += f!("    public Ref{0} Ref() => new Ref{0}(_dyn);", trait_name);
        ls += f!("");
        for derive in &meta.rs_derives {
            let snake_derive = derive.to_snake();
            ls += f!("    [DllImport(\"critical_point_csbridge.dll\")]");
            ls += f!("    private static extern unsafe Rs{}* {}_box_ref(RsBoxDyn<Rs{}>* pbox);", derive, snake_derive, rs_name);
            ls += f!("    public Ref{0} AsRef{0}() {{", derive);
            ls += f!("      var dyn = _dyn;");
            ls += f!("      var ptr = {}_box_ref(&dyn);", snake_derive);
            ls += f!("      if (ptr == null) throw new NullReferenceException(\"Invalid {}\");", derive);
            ls += f!("      return new Ref{}(ptr);", derive);
            ls += f!("    }}");
        }
        ls += f!("  }}\r\n");

        // Box
        ls += f!("  public unsafe class Box{} : IDisposable {{", trait_name);
        ls += f!("    private RsBoxDyn<Rs{}> _dyn;", rs_name);
        self.gen_ref_fields(ctx, &mut ls, "_dyn.ptr->")?;
        ls += f!("");
        ls += f!("    internal Box{}(RsBoxDyn<Rs{}> dyn) => _dyn = dyn;", trait_name, rs_name);
        ls += f!("    public Ref{0} Ref() => new Ref{0}(_dyn);", trait_name);
        ls += f!("");
        ls += f!("    [DllImport(\"critical_point_csbridge.dll\")]");
        ls += f!("    private static extern unsafe void {}_box_drop(RsBoxDyn<Rs{}> box);", snake_case, rs_name);
        ls += f!("    public void Dispose() {{");
        ls += f!("      if (!_dyn.IsNull) {{");
        ls += f!("        {}_box_drop(_dyn);", snake_case);
        ls += f!("        _dyn.Clear();");
        ls += f!("      }}");
        ls += f!("    }}");
        ls += f!("    ~Box{}() => Dispose();", trait_name);
        ls += f!("");
        for derive in &meta.rs_derives {
            ls += f!("    public Ref{0} AsRef{0}() => new Ref{1}(_dyn).AsRef{0}();", derive, trait_name);
        }
        ls += f!("  }}\r\n");

        // RsArc
        ls += f!("  internal unsafe struct RsArc{} {{", trait_name);
        ls += f!("    private RsArcDyn<Rs{}> _dyn;", rs_name);
        ls += f!("    internal Weak{0} MakeWeak() => new Weak{0}(_dyn);", trait_name);
        ls += f!("    internal Arc{0} MakeArc() => new Arc{0}(_dyn);", trait_name);
        ls += f!("  }}\r\n");

        // Weak
        ls += f!("  public unsafe ref struct Weak{} {{", trait_name);
        ls += f!("    private RsArcDyn<Rs{}> _dyn;", rs_name);
        self.gen_ref_fields(ctx, &mut ls, "_dyn.ptr->data.")?;
        ls += f!("");
        ls += f!("    internal Weak{}(RsArcDyn<Rs{}> dyn) => _dyn = dyn;", trait_name, rs_name);
        ls += f!("    public Weak{0} Weak() => new Weak{0}(_dyn);", trait_name);
        ls += f!("    [DllImport(\"critical_point_csbridge.dll\")]");
        ls += f!("    private static extern unsafe RsArcDyn<Rs{0}> {1}_arc_clone(RsArcDyn<Rs{0}>* parc);", rs_name, snake_case);
        ls += f!("    public Arc{0} Arc() {{", trait_name);
        ls += f!("      var dyn = _dyn;");
        ls += f!("      return new Arc{}({}_arc_clone(&dyn));", trait_name, snake_case);
        ls += f!("    }}");
        ls += f!("");
        for derive in &meta.rs_derives {
            let snake_derive = derive.to_snake();
            ls += f!("    [DllImport(\"critical_point_csbridge.dll\")]");
            ls += f!("    private static extern unsafe RsArcInner<Rs{}>* {}_arc_ref(RsArcDyn<Rs{}>* dyn);", derive, snake_derive, rs_name);
            ls += f!("    public Weak{0} AsWeak{0}() {{", derive);
            ls += f!("      var dyn = _dyn;");
            ls += f!("      var ptr = {}_arc_ref(&dyn);", snake_derive);
            ls += f!("      if (ptr == null) throw new NullReferenceException(\"Invalid {}\");", derive);
            ls += f!("      return new Weak{}(ptr);", derive);
            ls += f!("    }}");
            ls += f!("    [DllImport(\"critical_point_csbridge.dll\")]");
            ls += f!("    private static extern unsafe RsArcInner<Rs{}>* {}_arc_arc(RsArcDyn<Rs{}>* dyn);", derive, snake_derive, rs_name);
            ls += f!("    public Arc{0} AsArc{0}() {{", derive);
            ls += f!("      var dyn = _dyn;");
            ls += f!("      var ptr = {}_arc_arc(&dyn);", snake_derive);
            ls += f!("      if (ptr == null) throw new NullReferenceException(\"Invalid {}\");", derive);
            ls += f!("      return new Arc{}(ptr);", derive);
            ls += f!("    }}");
        }
        ls += f!("  }}\r\n");

        // Arc
        ls += f!("  public unsafe class Arc{} : IDisposable {{", trait_name);
        ls += f!("    private RsArcDyn<Rs{}> _dyn;", rs_name);
        self.gen_ref_fields(ctx, &mut ls, "_dyn.ptr->data.")?;
        ls += f!("");
        ls += f!("    internal Arc{}(RsArcDyn<Rs{}> dyn) => _dyn = dyn;", trait_name, rs_name);
        ls += f!("    public Weak{0} Weak() => new Weak{0}(_dyn);", trait_name);
        ls += f!("    public Arc{0} Arc() => new Weak{0}(_dyn).Arc();", trait_name);
        ls += f!("    public IntPtr StrongCount => _dyn.ptr->strong;");
        ls += f!("    public IntPtr WeakCount => _dyn.ptr->weak;");
        ls += f!("");
        ls += f!("    [DllImport(\"critical_point_csbridge.dll\")]");
        ls += f!("    private static extern unsafe void {}_arc_drop(RsArcDyn<Rs{}> arc);", snake_case, rs_name);
        ls += f!("    public void Dispose() {{");
        ls += f!("      if (!_dyn.IsNull) {{");
        ls += f!("        {}_arc_drop(_dyn);", snake_case);
        ls += f!("        _dyn.Clear();");
        ls += f!("      }}");
        ls += f!("    }}");
        ls += f!("    ~Arc{}() => Dispose();", trait_name);
        ls += f!("");
        for derive in &meta.rs_derives {
            ls += f!("    public Weak{0} AsWeak{0}() => new Weak{1}(_dyn).AsWeak{0}();", derive, trait_name);
            ls += f!("    public Arc{0} AsArc{0}() => new Weak{1}(_dyn).AsArc{0}();", derive, trait_name);
        }
        ls += f!("  }}\r\n");

        Ok(ls.join())
    }

    #[rustfmt::skip]
    fn gen_ref_type(&self, ctx: &GenContext<'_>) -> Result<String> {
        let rs_name = &self.rs_name;
        let snake_case = rs_name.to_snake();
        let mut ls = Lines::new(self.fields.len());

        // RsBox
        ls += f!("  internal unsafe struct RsBox{} {{", rs_name);
        ls += f!("    private Rs{}* _ptr;", rs_name);
        ls += f!("    internal Ref{0} MakeRef() => new Ref{0}(_ptr);", rs_name);
        ls += f!("    internal Box{0} MakeBox() => new Box{0}(_ptr);", rs_name);
        ls += f!("  }}\r\n");

        // Box
        ls += f!("  public unsafe class Box{} : IDisposable {{", rs_name);
        ls += f!("    private Rs{}* _ptr;", rs_name);
        self.gen_ref_fields(ctx, &mut ls, "_ptr->")?;
        ls += f!("");
        ls += f!("    internal Box{0}(Rs{0}* ptr) => _ptr = ptr;", rs_name);
        ls += f!("    public Ref{0} Ref() => new Ref{0}(_ptr);", rs_name);
        ls += f!("");
        ls += f!("    [DllImport(\"critical_point_csbridge.dll\")]");
        ls += f!("    private static extern unsafe void {}_box_drop(Rs{}* box);", snake_case, rs_name);
        ls += f!("    public void Dispose() {{");
        ls += f!("      if (_ptr != null) {{");
        ls += f!("        {}_box_drop(_ptr);", snake_case);
        ls += f!("        _ptr = null;");
        ls += f!("      }}");
        ls += f!("    }}");
        ls += f!("    ~Box{}() => Dispose();", rs_name);
        ls += f!("  }}\r\n");

        // Ref
        ls += f!("  public unsafe ref struct Ref{} {{", rs_name);
        ls += f!("    private Rs{}* _ptr;", rs_name);
        self.gen_ref_fields(ctx, &mut ls, "_ptr->")?;
        ls += f!("");
        ls += f!("    internal Ref{0}(Rs{0}* ptr) {{ _ptr = ptr; }}", rs_name);
        ls += f!("  }}\r\n");

        // RsArc
        ls += f!("  internal unsafe struct RsArc{} {{", rs_name);
        ls += f!("    private RsArcInner<Rs{}>* _ptr;", rs_name);
        ls += f!("    internal Weak{0} MakeWeak() => new Weak{0}(_ptr);", rs_name);
        ls += f!("    internal Arc{0} MakeArc() => new Arc{0}(_ptr);", rs_name);
        ls += f!("  }}\r\n");

        // Arc
        ls += f!("  public unsafe class Arc{} : IDisposable {{", rs_name);
        ls += f!("    private RsArcInner<Rs{}>* _ptr;", rs_name);
        self.gen_ref_fields(ctx, &mut ls, "_ptr->data.")?;
        ls += f!("");
        ls += f!("    [DllImport(\"critical_point_csbridge.dll\")]");
        ls += f!("    private static extern unsafe RsArcInner<Rs{0}>* {1}_arc_clone(RsArcInner<Rs{0}>** pptr);", rs_name, snake_case);
        ls += f!("    internal Arc{}(RsArcInner<Rs{}>* ptr) => _ptr = ptr;", rs_name, rs_name);
        ls += f!("    public Weak{0} Weak() => new Weak{0}(_ptr);", rs_name);
        ls += f!("    public Arc{} Arc() {{", rs_name);
        ls += f!("      var ptr = _ptr;");
        ls += f!("      return new Arc{}({}_arc_clone(&ptr));", rs_name, snake_case);
        ls += f!("    }}");
        ls += f!("    public IntPtr StrongCount => _ptr->strong;");
        ls += f!("    public IntPtr WeakCount => _ptr->weak;");
        ls += f!("");
        ls += f!("    [DllImport(\"critical_point_csbridge.dll\")]");
        ls += f!("    private static extern unsafe void {}_arc_drop(RsArcInner<Rs{}>* ptr);", snake_case, rs_name);
        ls += f!("    public void Dispose() {{");
        ls += f!("      if (_ptr != null) {{");
        ls += f!("        {}_arc_drop(_ptr);", snake_case);
        ls += f!("        _ptr = null;");
        ls += f!("      }}");
        ls += f!("    }}");
        ls += f!("    ~Arc{}() => Dispose();", rs_name);
        ls += f!("  }}\r\n");

        // Weak
        ls += f!("  public unsafe ref struct Weak{} {{", rs_name);
        ls += f!("    private RsArcInner<Rs{}>* _ptr;", rs_name);
        self.gen_ref_fields(ctx, &mut ls, "_ptr->data.")?;
        ls += f!("");
        ls += f!("    internal Weak{}(RsArcInner<Rs{}>* ptr) => _ptr = ptr;", rs_name, rs_name);
        ls += f!("    public Weak{0} Weak() => new Weak{0}(_ptr);", rs_name);
        ls += f!("    public Arc{0} Arc() => new Weak{0}(_ptr).Arc();", rs_name);
        ls += f!("  }}\r\n");

        Ok(ls.join())
    }

    fn gen_ref_fields(&self, ctx: &GenContext<'_>, ls: &mut Lines, visitor: &str) -> Result<()> {
        for (idx, field) in self.fields.iter().enumerate() {
            if idx == 0 && field.field() == "_base" {
                match ctx.bases.get(field.rs_type()) {
                    Some(base) => *ls += base.code.replace("@@@@", visitor),
                    None => return Err(anyhow!("Unknown base type {}", field.rs_type())),
                };
                continue;
            }

            let typ = ctx
                .types_out
                .get(field.rs_type())
                .ok_or_else(|| anyhow!("Unknown type {} in {}", field.rs_type(), self.rs_name))?;

            match field {
                FieldOut::Type { field, rs_type } => {
                    match typ {
                        TypeOut::Value(v) => {
                            *ls += f!("    public {0} {1} => {2}{1};", v.cs_name, field, visitor);
                        }
                        _ => return Err(anyhow!("Value type ({}) not found", rs_type)),
                    };
                }
                FieldOut::Array { field, rs_type, .. } => {
                    match typ {
                        TypeOut::Value(v) => {
                            *ls += f!("    public RefArrayVal<{0}> {1} => {2}{1};", v.cs_name, field, visitor);
                        }
                        _ => return Err(anyhow!("Value type ({}) not found", rs_type)),
                    };
                }
                FieldOut::Vec { field, rs_type } => {
                    match typ {
                        TypeOut::Value(v) => {
                            *ls += f!("    public RefVecVal<{0}> {1} => {2}{1};", v.cs_name, field, visitor);
                        }
                        TypeOut::Reference(r) if !r.is_trait => {
                            *ls += f!("    public RefVecRs{0} {1} => {2}{1};", r.rs_name, field, visitor);
                        }
                        _ => return Err(anyhow!("Value type ({}) not found", rs_type)),
                    };
                }
                FieldOut::String { field } => {
                    *ls += f!("    public RefString {0} => {1}{0};", field, visitor);
                }
                FieldOut::VecReference {
                    field,
                    rs_type,
                    ref_type,
                } => match typ {
                    TypeOut::Reference(r) => {
                        if *ref_type == ReferenceType::Box {
                            *ls += f!("    public RefVecBox{0} {1} => {2}{1};", r.rs_name, field, visitor);
                        }
                        else {
                            *ls += f!("    public RefVecArc{0} {1} => {2}{1};", r.rs_name, field, visitor);
                        }
                    }
                    _ => return Err(anyhow!("Reference type ({}) not found", rs_type)),
                },
                _ => return Err(anyhow!("Type ({}) not support", field.rs_type())),
            };
        }
        Ok(())
    }
}
