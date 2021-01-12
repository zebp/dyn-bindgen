use proc_macro2::Span;
use syn::{Block, Ident, Item, ItemMod, token::Brace};

use crate::parse::{BoundFunction, Parsed};

pub fn generate_libloading_glue(parsed: &Parsed) -> Item {
    let calls = parsed.functions.iter()
        .map(BoundFunction::name)
        .map(|name| {
            let name_ptr = format!("{}_ptr", name);
            let name_ptr = Ident::new(&name_ptr, Span::call_site());

            let name_fn = format!("{}_fn", name);
            let name_fn = Ident::new(&name_fn, Span::call_site());

            syn::parse_quote!({
                match lib.get::<super::#name_fn>(#name.as_bytes()) {
                    Ok(sym) => {
                        let sym = Box::new(sym);
                        let sym = Box::leak(sym);
                        super::#name_ptr = (*sym).deref() as *const super::#name_fn;
                    },
                    Err(e) => {
                        // TODO: Figure out an elegant way to display this,
                        // maybe only print it in debug mode?
                        // eprintln!("Error loading {}: {:#?}", #name, e);
                    }
                }
            })
        })
        .collect::<Vec<syn::Stmt>>();

    let block = Block {
        brace_token: Brace::default(),
        stmts: calls
    };

    let module_item: ItemMod = syn::parse_quote!(mod glue {
        use std::path::Path;
        use std::ops::Deref;

        pub fn load<P: AsRef<Path>>(library: P) {
            
            let lib = libloading::Library::new(library.as_ref()).unwrap();
            let lib = Box::leak(Box::new(lib)); // Leak the library so it isn't dropped

            unsafe #block
        }
    });

    Item::Mod(module_item)
}