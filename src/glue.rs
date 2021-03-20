use proc_macro2::Span;
use syn::{token::Brace, Block, Ident, Item};

use crate::{
    bundle::LoadingStrategy,
    parse::{BoundFunction, Parsed},
};

pub fn generate_libloading_glue(loading_strategy: LoadingStrategy, parsed: &Parsed) -> Item {
    let item_loader_block = generate_function_loaders_block(parsed);

    let loader = loading_strategy
        .generate_loader_function(item_loader_block)
        .unwrap();

    if loading_strategy.is_implicitly_loaded_bundle() {
        loader
    } else {
        syn::parse_quote!(mod glue { #loader })
    }
}

fn generate_function_loaders_block(parsed: &Parsed) -> Block {
    let calls = parsed
        .functions
        .iter()
        .map(BoundFunction::name)
        .map(|name| {
            let name_ptr = format!("{}_ptr", name);
            let name_ptr = Ident::new(&name_ptr, Span::call_site());

            let name_fn = format!("{}_fn", name);
            let name_fn = Ident::new(&name_fn, Span::call_site());

            syn::parse_quote!({
                match lib.get::<crate::#name_fn>(#name.as_bytes()) {
                    Ok(sym) => {
                        let sym = Box::new(sym);
                        let sym = Box::leak(sym);
                        crate::#name_ptr = (*sym).deref() as *const crate::#name_fn;
                    },
                    Err(_e) => {
                        // TODO: Figure out an elegant way to display this,
                        // maybe only print it in debug mode?
                        // eprintln!("Error loading {}: {:#?}", #name, e);
                    }
                }
            })
        })
        .collect::<Vec<syn::Stmt>>();

    Block {
        brace_token: Brace::default(),
        stmts: calls,
    }
}
