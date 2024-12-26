mod base;
mod gen_enum;
mod gen_struct;

use anyhow::{anyhow, Result};
use case::CaseExt;
use gen_enum::parse_enum;
use gen_struct::{parse_struct_in, parse_struct_out};
use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::sync::{LazyLock, Mutex};
use syn::*;

use crate::base::*;

struct Generator {
    consts: HashMap<String, u32>,
    types_in: HashMap<String, TypeIn>,
    types_out: HashMap<String, TypeOut>,
    tasks: Vec<Box<dyn Task>>,
    bases: HashMap<String, BaseMeta>,
}

static GENERATOR: LazyLock<Mutex<Generator>> = LazyLock::new(|| Mutex::new(Generator::new()));

impl Generator {
    fn new() -> Generator {
        let mut consts = HashMap::new();
        consts.insert("FPS".to_string(), 15);
        consts.insert("MAX_ACTION_ANIMATION".to_string(), 4);
        consts.insert("MAX_ACCESSORY_COUNT".to_string(), 4);
        consts.insert("MAX_ENTRY_PLUS".to_string(), 3);
        consts.insert("MAX_EQUIPMENT_COUNT".to_string(), 3);

        let mut types_in = HashMap::new();
        types_in.insert("bool".to_string(), TypeIn::new_primitive("bool"));
        types_in.insert("i8".to_string(), TypeIn::new_primitive("sbyte"));
        types_in.insert("u8".to_string(), TypeIn::new_primitive("byte"));
        types_in.insert("i16".to_string(), TypeIn::new_primitive("short"));
        types_in.insert("u16".to_string(), TypeIn::new_primitive("ushort"));
        types_in.insert("i32".to_string(), TypeIn::new_primitive("int"));
        types_in.insert("u32".to_string(), TypeIn::new_primitive("uint"));
        types_in.insert("i64".to_string(), TypeIn::new_primitive("long"));
        types_in.insert("u64".to_string(), TypeIn::new_primitive("ulong"));
        types_in.insert("f32".to_string(), TypeIn::new_primitive("float"));
        types_in.insert("f64".to_string(), TypeIn::new_primitive("double"));
        types_in.insert("NumID".to_string(), TypeIn::new_primitive("ulong"));
        types_in.insert("StrID".to_string(), TypeIn::new_primitive("string"));
        types_in.insert("Symbol".to_string(), TypeIn::new_primitive("string"));
        types_in.insert("[f32; 2]".to_string(), TypeIn::new_primitive("Vec2"));
        types_in.insert("[f32; 3]".to_string(), TypeIn::new_primitive("Vec3"));
        types_in.insert("Vec2".to_string(), TypeIn::new_primitive("Vec2"));
        types_in.insert("Vec3".to_string(), TypeIn::new_primitive("Vec3"));
        types_in.insert("Vec4".to_string(), TypeIn::new_primitive("Vec4"));
        types_in.insert("Quat".to_string(), TypeIn::new_primitive("Quat"));
        types_in.insert("String".to_string(), TypeIn::new_primitive("string"));
        types_in.insert("Vec".to_string(), TypeIn::new_generic("List", 1));
        types_in.insert("HashMap".to_string(), TypeIn::new_generic("Dictionary", 2));
        types_in.insert("HashSet".to_string(), TypeIn::new_generic("HashSet", 1));

        let mut types_out = HashMap::new();
        types_out.insert("".to_string(), TypeOut::new_value("#error#", "#error#"));
        types_out.insert("bool".to_string(), TypeOut::new_value("bool", "bool"));
        types_out.insert("i8".to_string(), TypeOut::new_value("i8", "sbyte"));
        types_out.insert("u8".to_string(), TypeOut::new_value("u8", "byte"));
        types_out.insert("i16".to_string(), TypeOut::new_value("i16", "short"));
        types_out.insert("u16".to_string(), TypeOut::new_value("u16", "ushort"));
        types_out.insert("i32".to_string(), TypeOut::new_value("i32", "int"));
        types_out.insert("u32".to_string(), TypeOut::new_value("u32", "uint"));
        types_out.insert("i64".to_string(), TypeOut::new_value("i64", "long"));
        types_out.insert("u64".to_string(), TypeOut::new_value("u64", "ulong"));
        types_out.insert("f32".to_string(), TypeOut::new_value("f32", "float"));
        types_out.insert("f64".to_string(), TypeOut::new_value("f64", "double"));
        types_out.insert("NumID".to_string(), TypeOut::new_value("NumID", "ulong"));
        types_out.insert("StrID".to_string(), TypeOut::new_value("StrID", "Symbol"));
        types_out.insert("Symbol".to_string(), TypeOut::new_value("Symbol", "Symbol"));
        types_out.insert("[f32; 2]".to_string(), TypeOut::new_value("[f32; 2]", "Vec2"));
        types_out.insert("[f32; 3]".to_string(), TypeOut::new_value("[f32; 3]", "Vec3"));
        types_out.insert("Vec2".to_string(), TypeOut::new_value("Vec2", "Vec2"));
        types_out.insert("Vec3".to_string(), TypeOut::new_value("Vec3", "Vec3"));
        types_out.insert("Vec4".to_string(), TypeOut::new_value("Vec4", "Vec4"));
        types_out.insert("Quat".to_string(), TypeOut::new_value("Quat", "Quat"));
        types_out.insert("SoaVec3".to_string(), TypeOut::new_value("SoaVec3", "SoaVec3"));
        types_out.insert("SoaQuat".to_string(), TypeOut::new_value("SoaQuat", "SoaQuat"));
        types_out.insert(
            "SoaTransform".to_string(),
            TypeOut::new_value("SoaTransform", "SoaTransform"),
        );
        types_out.insert("dyn StateAny".to_string(), TypeOut::new_trait("StateAny"));
        types_out.insert("dyn StateAction".to_string(), TypeOut::new_trait("StateAction"));

        let mut bases = HashMap::new();
        bases.insert("StateAnyBase".to_string(), BaseMeta::new("StateAnyBase", "DynStateAny"));
        bases.insert(
            "StateActionBase".to_string(),
            BaseMeta::new("StateActionBase", "DynStateAction"),
        );

        unsafe { libc::atexit(Generator::on_exit) };

        Generator {
            consts,
            types_in,
            types_out,
            tasks: Vec::new(),
            bases,
        }
    }

    fn parse_enum(&mut self, input: &ItemEnum) -> Result<()> {
        let (rs_name, task, type_in, type_out) = parse_enum(input)?;
        self.tasks.push(task);
        self.types_in.insert(rs_name.clone(), type_in);
        self.types_out.insert(rs_name.clone(), type_out);
        Ok(())
    }

    fn parse_struct_in(&mut self, input: &ItemStruct) -> Result<()> {
        let (rs_name, task, type_in) = parse_struct_in(input, &self.consts)?;
        self.tasks.push(task);
        self.types_in.insert(rs_name.clone(), type_in);
        Ok(())
    }

    fn parse_struct_out(&mut self, input: &ItemStruct) -> Result<()> {
        let (rs_name, base, task, type_out) = parse_struct_out(input, &self.consts)?;
        self.tasks.push(task);
        self.types_out.insert(rs_name.clone(), type_out);
        if !base.is_empty() {
            if let Some(meta) = self.bases.get_mut(&base) {
                meta.rs_derives.push(rs_name);
            } else {
                return Err(anyhow!("Base ({}) not found", base));
            }
        }
        Ok(())
    }

    fn generate_file(&mut self) -> Result<()> {
        // let crate_name = env::var("CARGO_PKG_NAME")?;
        // let mut file = if crate_name.ends_with("-core") {
        //     File::create("../critical-point-cs/bridge/AutoGenCore.cs")?
        // } else if crate_name.ends_with("-csbridge") {
        //     File::create("../critical-point-cs/bridge/AutoGenCsBridge.cs")?
        // } else {
        //     unreachable!()
        // };
        let mut file = File::create("../critical-point-cs/bridge/AutoGen.cs")?;
        file.write_all(
            [
                "using MessagePack;",
                "using System;",
                "using System.Collections.Generic;",
                "using System.Numerics;",
                "using System.Runtime.InteropServices;",
                "",
                "namespace CriticalPoint {\n",
            ]
            .join("\r\n")
            .as_bytes(),
        )?;

        for task in &self.tasks {
            let (rs_name, code) = task.gen_base(&GenContext {
                types_in: &self.types_in,
                types_out: &self.types_out,
                bases: &self.bases,
            })?;
            if !code.is_empty() {
                match self.bases.get_mut(&rs_name) {
                    Some(meta) => meta.code = code,
                    None => return Err(anyhow!("Base ({}) not found", rs_name)),
                }
            }
        }

        for task in &self.tasks {
            let code = task.gen(&GenContext {
                types_in: &self.types_in,
                types_out: &self.types_out,
                bases: &self.bases,
            })?;
            file.write_all(code.as_bytes())?;
            file.write_all("\r\n".as_bytes())?;
        }

        file.write_all("}\r\n".as_bytes())?;
        Ok(())
    }

    extern "C" fn on_exit() {
        if let Ok(mut gen) = GENERATOR.lock() {
            let res = gen.generate_file();
            println!("\r\n════════════════════════════════════════════════════════════");
            println!("------------------------------------------------------------\r\n");
            match res {
                Ok(_) => {
                    println!("Critical Point generate C# OK.");
                }
                Err(e) => {
                    println!("Critical Point generate C# error:");
                    println!("{:?}", e);
                }
            }
            println!("\r\n------------------------------------------------------------");
            println!("════════════════════════════════════════════════════════════\r\n");
        }
    }
}

#[proc_macro_derive(CsEnum)]
pub fn csharp_enum_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemEnum);
    GENERATOR.lock().unwrap().parse_enum(&input).unwrap();
    TokenStream::from(quote! {})
}

#[proc_macro_derive(CsIn, attributes(cs_attr))]
pub fn csharp_struct_in_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let res = GENERATOR.lock().unwrap().parse_struct_in(&input);
    res.unwrap();
    TokenStream::from(quote! {})
}

#[proc_macro_derive(CsOut, attributes(cs_attr))]
pub fn csharp_struct_out_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemStruct);
    let res = GENERATOR.lock().unwrap().parse_struct_out(&input);
    res.unwrap();
    TokenStream::from(quote! {})
}
