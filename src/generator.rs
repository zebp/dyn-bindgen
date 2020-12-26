use anyhow::Result;
use syn::{
    export::Span, punctuated::Punctuated, token::Comma, Abi, File, FnArg, Ident, Item, ItemFn,
    LitStr, Pat, Signature, Token, Type, TypeBareFn, VisPublic, Visibility,
};

use crate::parse::{BoundFunction, Parsed};

pub fn generate_bindings_module(parsed: Parsed) -> Result<File> {
    let mut file = File {
        shebang: None,
        attrs: Vec::new(),
        items: Vec::new(),
    };

    file.items.push(syn::parse_quote!(pub use glue::*;));

    let glue_mod = crate::glue::generate_libloading_glue(&parsed);
    file.items.push(glue_mod);

    parsed
        .functions
        .iter()
        .for_each(|func| append_items(&mut file.items, func));

    Ok(file)
}

pub fn append_items(module_items: &mut Vec<Item>, func: &BoundFunction) {
    append_type(module_items, func);
    append_function_ptr(module_items, func);
    append_function(module_items, func);
}

fn append_type(module_items: &mut Vec<Item>, func: &BoundFunction) {
    let type_sig: TypeBareFn = func.into();
    let ty = Type::BareFn(type_sig);

    let name = format!("{}_fn", func.0.ident);
    let name = Ident::new(&name, Span::call_site());

    module_items.push(syn::parse_quote! {
        #[allow(non_camel_case_types)]
        type #name = #ty;
    });
}

fn append_function_ptr(module_items: &mut Vec<Item>, func: &BoundFunction) {
    let name = Ident::new(&format!("{}_ptr", func.0.ident), Span::call_site());
    let type_name = Ident::new(&format!("{}_fn", func.0.ident), Span::call_site());

    module_items.push(syn::parse_quote! {
        #[allow(non_upper_case_globals)]
        static mut #name: *const #type_name = std::ptr::null();
    });
}

fn append_function(module_items: &mut Vec<Item>, func: &BoundFunction) {
    let name = Ident::new(&format!("{}_ptr", func.0.ident), Span::call_site());

    let sig = Signature {
        unsafety: Some(Default::default()),
        abi: Some(Abi {
            extern_token: Default::default(),
            name: Some(LitStr::new("C", Span::call_site())),
        }),
        ..func.0.clone()
    };

    let args = func
        .0
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat) => match *(pat.pat.clone()) {
                Pat::Ident(ident) => Some(ident.ident.clone()),
                _ => None,
            },
            _ => None,
        })
        .fold(Punctuated::new(), |mut acc, ident| {
            acc.push_value(ident);
            acc.push_punct(<Token![,]>::default());
            acc
        });

    module_items.push(construct_debug_function(
        args.clone(),
        name.clone(),
        sig.clone(),
    ));
    module_items.push(construct_release_function(args.clone(), name.clone(), sig));
}

fn construct_debug_function(args: Punctuated<Ident, Comma>, name: Ident, sig: Signature) -> Item {
    let cfg_debug_attr = syn::parse_quote! { #[cfg(debug_assertions)] };
    let panic_str = format!(
        "attempt to call '{}' but it has not been loaded from it's library.",
        name
    );

    Item::Fn(ItemFn {
        vis: Visibility::Public(VisPublic {
            pub_token: Default::default(),
        }),
        attrs: vec![cfg_debug_attr],
        block: syn::parse_quote! {
            {
                if #name.is_null() {
                    panic!(#panic_str);
                } else {
                    (*#name)(#args)
                }
            }
        },
        sig,
    })
}

fn construct_release_function(args: Punctuated<Ident, Comma>, name: Ident, sig: Signature) -> Item {
    let cfg_not_debug_attr = syn::parse_quote! { #[cfg(not(debug_assertions))] };
    Item::Fn(ItemFn {
        vis: Visibility::Public(VisPublic {
            pub_token: Default::default(),
        }),
        attrs: vec![cfg_not_debug_attr],
        block: syn::parse_quote! {
            { #name.read()(#args) }
        },
        sig,
    })
}
