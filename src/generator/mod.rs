mod functions;

use anyhow::Result;
use syn::File;

use crate::parse::Parsed;
use crate::bundle::LoadingStrategy;

pub fn generate_bindings_module(loading_strategy: LoadingStrategy, parsed: Parsed) -> Result<File> {
    let mut file = File {
        shebang: None,
        attrs: Vec::new(),
        items: Vec::new(),
    };

    file.items.push(syn::parse_quote!(
        pub use glue::*;
    ));

    let glue_mod = crate::glue::generate_libloading_glue(loading_strategy, &parsed);
    file.items.push(glue_mod);

    let Parsed {
        functions,
        const_items,
        struct_items,
        type_items,
    } = parsed;

    const_items
        .into_iter()
        .for_each(|item| file.items.push(item.into()));
    struct_items
        .into_iter()
        .for_each(|item| file.items.push(item.into()));
    type_items
        .into_iter()
        .for_each(|item| file.items.push(item.into()));

    functions
        .iter()
        .for_each(|func| functions::append_items(&mut file.items, func));

    Ok(file)
}
