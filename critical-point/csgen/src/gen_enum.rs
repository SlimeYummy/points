use anyhow::{anyhow, Ok, Result};
use proc_macro::TokenStream;
use quote::quote;
use syn::ItemEnum;

use crate::base::*;

pub fn parse_enum(input: &ItemEnum) -> Result<(String, Box<dyn GenerateTask>, TypeIn, TypeOut)> {
    let raw = extract_attr_raw(&input.attrs, "repr")?;
    let (cs_type, size, align) = match raw.as_str() {
        "i8" => ("sbyte", 1, 1),
        "u8" => ("byte", 1, 1),
        "i16" => ("short", 2, 2),
        "u16" => ("ushort", 2, 2),
        "i32" => ("int", 4, 4),
        "u32" => ("uint", 4, 4),
        "i64" => ("long", 8, 8),
        "u64" => ("ulong", 8, 8),
        _ => return Err(anyhow!("Need a #[repr(i8|u8|i16|u16|i32|u32|i64|u64)]")),
    };

    let rs_name = input.ident.to_string();
    let cs_prefix = rs_name.starts_with("Cs") && rs_name.chars().nth(2).map(|c| c.is_uppercase()).unwrap_or(false);
    let cs_name = match cs_prefix {
        true => rs_name[2..].to_string(),
        false => rs_name.clone(),
    };
    let mut task = Box::new(TaskEnum {
        cs_name: cs_name,
        cs_type: cs_type.into(),
        items: Vec::with_capacity(input.variants.len()),
    });

    for variant in &input.variants {
        if !variant.fields.is_empty() {
            return Err(anyhow!("Unsupported enum type"));
        }
        if let Some((_, expr)) = &variant.discriminant {
            task.items.push(EnumItem {
                item: variant.ident.to_string(),
                expr: Some(TokenStream::from(quote! { #expr }).to_string()),
            });
        }
        else {
            task.items.push(EnumItem {
                item: variant.ident.to_string(),
                expr: None,
            });
        }
    }

    let type_in = TypeIn::new_primitive(&rs_name);
    let type_out = TypeOut::new_value(&rs_name, &rs_name, size, align);
    Ok((rs_name, task, type_in, type_out))
}

pub struct TaskEnum {
    cs_name: String,
    cs_type: String,
    items: Vec<EnumItem>,
}

pub struct EnumItem {
    item: String,
    expr: Option<String>,
}

impl GenerateTask for TaskEnum {
    fn generate(&self, _ctx: &GenerateContext<'_>) -> Result<String> {
        let mut ls = Lines::new(self.items.len());
        ls += f!("  public enum {}: {} {{", self.cs_name, self.cs_type);
        for item in &self.items {
            ls += match &item.expr {
                Some(expr) => f!("    {} = {},", item.item, expr),
                None => f!("    {},", item.item),
            };
        }
        ls += f!("  }}\r\n");
        Ok(ls.join())
    }
}
