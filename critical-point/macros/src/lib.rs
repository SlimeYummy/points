mod csharp;
mod utils;
mod wasm;

use proc_macro::TokenStream;
use quote::quote;
use std::sync::Once;
use syn::{ItemEnum, ItemImpl, ItemStruct, parse_macro_input};

static INIT: Once = Once::new();

#[cfg(feature = "csharp")]
static GEN_CSHARP: bool = true;
#[cfg(not(feature = "csharp"))]
static GEN_CSHARP: bool = false;

#[cfg(feature = "wasm")]
static GEN_WASM: bool = true;
#[cfg(not(feature = "wasm"))]
static GEN_WASM: bool = false;

fn init() {
    INIT.call_once(|| unsafe {
        libc::atexit(on_exit);
    });
}

extern "C" fn on_exit() {
    if !GEN_CSHARP && !GEN_WASM {
        return;
    }

    println!("\r\n════════════════════════════════════════════════════════════");
    println!("------------------------------------------------------------\r\n");

    // Generate C# code
    if GEN_CSHARP {
        let mut generator = csharp::lock_csharp_generator();
        let res = generator.generate_file();
        match res {
            Ok(_) => {
                println!("Critical Point generate C# OK.");
            }
            Err(e) => {
                println!("Critical Point generate C# error:");
                println!("{:?}", e);
            }
        }
    }

    // Generate Wasm code
    if GEN_WASM {
        let mut generator = wasm::lock_wasm_generator();
        let res = generator.generate_file();
        match res {
            Ok(_) => {
                println!("Critical Point generate WASM(Rust) OK.");
            }
            Err(e) => {
                println!("Critical Point generate WASM(Rust) error:");
                println!("{:?}", e);
            }
        }
    }

    println!("\r\n------------------------------------------------------------");
    println!("════════════════════════════════════════════════════════════\r\n");
}

#[proc_macro_attribute]
pub fn csharp_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    init();
    let input = parse_macro_input!(input as ItemEnum);
    if GEN_CSHARP {
        csharp::lock_csharp_generator().parse_enum(&input).unwrap();
    }
    TokenStream::from(quote! { #input })
}

#[proc_macro_attribute]
pub fn csharp_in(attr: TokenStream, input: TokenStream) -> TokenStream {
    init();
    let mut input = parse_macro_input!(input as ItemStruct);
    if GEN_CSHARP {
        csharp::lock_csharp_generator().parse_struct_in(attr, &input).unwrap();
    }
    utils::remove_field_attrs(&mut input, "csharp_hide");
    TokenStream::from(quote! { #input })
}

#[proc_macro_attribute]
pub fn csharp_out(attr: TokenStream, input: TokenStream) -> TokenStream {
    init();
    let mut input = parse_macro_input!(input as ItemStruct);
    let mut tokens = Vec::new();
    if GEN_CSHARP {
        tokens = csharp::lock_csharp_generator().parse_struct_out(attr, &input).unwrap();
    }
    utils::remove_field_attrs(&mut input, "csharp_hide");
    TokenStream::from(quote! {
        #input
        #(#tokens)*
    })
}

/// Usage:
/// ```
/// #[repr(u8)]
/// #[wasm_enum]
/// enum Enum {
///   // ...
/// }
/// ```
#[proc_macro_attribute]
pub fn wasm_enum(_attr: TokenStream, input: TokenStream) -> TokenStream {
    init();
    let input = parse_macro_input!(input as ItemEnum);
    if GEN_WASM {
        wasm::lock_wasm_generator().gen_enum(&input).unwrap();
    }
    TokenStream::from(quote! { #input })
}

/// Usage:
/// ```
/// #[repr(C)]
/// #[wasm_struct(8, 4)]
/// struct Struct {
///     field1: u32,
///     #[wasm_hide(4, 4)]
///     field2: u32,
/// }
/// ```
#[proc_macro_attribute]
pub fn wasm_struct(attr: TokenStream, input: TokenStream) -> TokenStream {
    init();
    let mut input = parse_macro_input!(input as ItemStruct);
    if GEN_WASM {
        let res = wasm::lock_wasm_generator().gen_struct(attr, &input);
        let asserts = res.unwrap();
        utils::remove_field_attrs(&mut input, "wasm_hide");
        TokenStream::from(quote! {
            #input
            #(#asserts)*
        })
    }
    else {
        utils::remove_field_attrs(&mut input, "wasm_hide");
        TokenStream::from(quote! { #input })
    }
}

/// Usage:
/// ```
/// #[wasm_impl]
/// impl Xxx {
///     fn func(&self) { }
/// }
/// ```
#[proc_macro_attribute]
pub fn wasm_impl(_attr: TokenStream, input: TokenStream) -> TokenStream {
    init();
    let input = parse_macro_input!(input as ItemImpl);
    if GEN_WASM {
        wasm::lock_wasm_generator().gen_impl(&input).unwrap();
    }
    TokenStream::from(quote! { #input })
}
