use std::path::PathBuf;

use proc_macro2::Span;
use syn::{
    punctuated::Punctuated, token::Bracket, Block, Expr, ExprArray, ExprLit, Item, ItemConst,
    ItemFn, LitInt,
};

/// A dynamic library that can be loaded by the generated crate.
#[derive(Debug, Clone)]
pub enum Bundle {
    RawBytes(Vec<u8>),
    File(PathBuf),
}

impl Bundle {
    /// Creates a constant that contains the bytes of the dynamic library that
    /// should be loaded.
    fn create_bundle_constant(&self) -> std::io::Result<(ItemConst, String)> {
        let bytes = match self {
            Self::File(path) => std::fs::read(&path)?,
            Self::RawBytes(bytes) => bytes.clone(),
        };

        let hash = md5::compute(&bytes);
        let bundle_hash = hex::encode(*hash);

        let array_elements = bytes
            .into_iter()
            .fold(Punctuated::default(), |mut punc, byte| {
                let lit = LitInt::new(&byte.to_string(), Span::call_site());
                let expr_lit = ExprLit {
                    attrs: Vec::new(),
                    lit: lit.into(),
                };

                punc.push(Expr::Lit(expr_lit));
                punc
            });
        let array_expression = ExprArray {
            bracket_token: Bracket::default(),
            elems: array_elements,
            attrs: Vec::new(),
        };

        let bundle_constant = syn::parse_quote!(const BUNDLE_BYTES: &[u8] = &#array_expression;);
        Ok((bundle_constant, bundle_hash))
    }
}

#[derive(Debug, Clone)]
pub enum LoadingStrategy {
    /// The user must manually call the generated `load` function with a path to the correct dynamic library
    /// on disk. No functions are automatically loading and will panic if called.
    Manual,
    /// The user must manually call the generated `load_bundle` function, which will write the bundled dynamic
    /// operating system's temp directory and load it.
    ManuallyLoadedBundle(Bundle),
    /// Automatically writes the bundle to the temp directory and then loads it. Doesn't require any function calls.
    ImplicitlyLoadedBundle(Bundle),
}

impl LoadingStrategy {
    pub(crate) fn generate_loader_function(
        &self,
        item_loader_block: Block,
    ) -> anyhow::Result<Item> {
        let loader = match self {
            Self::Manual => manual_load_function(item_loader_block),
            Self::ManuallyLoadedBundle(bundle) => manual_load_from_bundle_function(
                item_loader_block,
                bundle.create_bundle_constant()?,
            ),
            Self::ImplicitlyLoadedBundle(bundle) => {
                implicit_load_from_bundle_ctor(item_loader_block, bundle.create_bundle_constant()?)
            }
        };

        Ok(loader)
    }

    /// Returns `true` if the loading_strategy is [`ImplicitlyLoadedBundle`].
    pub fn is_implicitly_loaded_bundle(&self) -> bool {
        matches!(self, Self::ImplicitlyLoadedBundle(..))
    }
}

fn manual_load_function(item_loader_block: Block) -> Item {
    syn::parse_quote!(
        /// Loads the provided library into the generate functions.
        /// This function will panic if the library cannot be loaded.
        pub unsafe fn load<P: AsRef<std::path::Path>>(library: P) {
            use std::ops::Deref;

            let lib = libloading::Library::new(library.as_ref())
                .expect("unable to load library");
            let lib = Box::leak(Box::new(lib)); // Leak the library so it isn't dropped

            unsafe #item_loader_block
        }
    )
}

fn manual_load_from_bundle_function(item_loader_block: Block, bundle_items: (ItemConst, String)) -> Item {
    let bundle_writer_fn = bundle_writer_function(bundle_items);

    syn::parse_quote!(
        pub unsafe fn load_bundle() {
            use std::ops::Deref;

            #bundle_writer_fn

            let lib_path = write_bundle_to_temp();
            let lib = libloading::Library::new(&lib_path)
                .expect("unable to load library");
            let lib = Box::leak(Box::new(lib));

            unsafe #item_loader_block
        }
    )
}

fn implicit_load_from_bundle_ctor(item_loader_block: Block, bundle_items: (ItemConst, String)) -> Item {
    let bundle_writer_fn = bundle_writer_function(bundle_items);

    syn::parse_quote!(
        #[used]
        #[cfg_attr(
            any(target_os = "linux", target_os = "android"),
            link_section = ".init_array"
        )]
        #[cfg_attr(target_os = "freebsd", link_section = ".init_array")]
        #[cfg_attr(target_os = "netbsd", link_section = ".init_array")]
        #[cfg_attr(target_os = "openbsd", link_section = ".init_array")]
        #[cfg_attr(
            any(target_os = "macos", target_os = "ios"),
            link_section = "__DATA,__mod_init_func"
        )]
        #[cfg_attr(target_os = "windows", link_section = ".CRT$XCU")]
        static IMPLICIT_DYN_BINDGEN_LOADER: unsafe extern "C" fn() = {
            #[cfg_attr(
                any(target_os = "linux", target_os = "android"),
                link_section = ".text.startup"
            )]
            unsafe extern "C" fn loader() {
                use std::ops::Deref;

                #bundle_writer_fn

                let lib_path = write_bundle_to_temp();
                let lib = libloading::Library::new(&lib_path)
                    .expect("unable to load library");
                let lib = Box::leak(Box::new(lib));

                unsafe #item_loader_block
            }

            loader
        };
    )
}

fn bundle_writer_function(bundle_items: (ItemConst, String)) -> ItemFn {
    let (bundle_constant, bundle_hash) = bundle_items;

    syn::parse_quote!(
        fn write_bundle_to_temp() -> std::path::PathBuf {
            #bundle_constant

            let mut path = std::env::temp_dir();
            path.push(format!("bundle.{}.module", #bundle_hash)); // TODO: Actually use the hash of the bundle

            std::fs::write(&path, BUNDLE_BYTES).expect("could not write dynamic library bundle");

            path
        }
    )
}
