use anyhow::Result;
use quote::quote;
use syn::ItemImpl;

pub fn gen_impl(input: &ItemImpl) -> Result<String> {
    let tokens = quote! { #input };
    Ok(format!("{}\n", tokens.to_string()))
}
