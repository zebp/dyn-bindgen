use anyhow::Result;
use syn::{export::*, visit_mut::VisitMut, *};

#[derive(Default)]
pub struct Parsed {
    pub functions: Vec<BoundFunction>,
    pub const_items: Vec<ItemConst>,
    // TODO: Figure out how it should handle static items.
    // pub static_items: Vec<ForeignItemStatic>,
    pub struct_items: Vec<ItemStruct>,
    pub type_items: Vec<ItemType>,
}

pub fn parse(code: &str) -> Result<Parsed> {
    let file: &mut File = &mut syn::parse_str(code)?;
    let mut parsed = Parsed::default();

    parsed.visit_file_mut(file);

    Ok(parsed)
}

impl VisitMut for Parsed {
    fn visit_foreign_item_fn_mut(&mut self, item_fn: &mut ForeignItemFn) {
        let sig = item_fn.sig.clone();
        self.functions.push(BoundFunction(sig));
    }
    
    fn visit_item_const_mut(&mut self, i: &mut ItemConst) {
        self.const_items.push(i.clone());
    }
    
    fn visit_item_struct_mut(&mut self, i: &mut ItemStruct) {
        self.struct_items.push(i.clone());
    }

    fn visit_item_type_mut(&mut self, i: &mut ItemType) {
        self.type_items.push(i.clone());
    }
}

pub struct BoundFunction(pub Signature);

impl BoundFunction {
    pub fn name(&self) -> String {
        self.0.ident.to_string()
    }
}

impl Into<TypeBareFn> for &BoundFunction {
    fn into(self) -> TypeBareFn {
        let Signature {
            constness: _,
            asyncness: _,
            unsafety: _,
            abi: _,
            fn_token,
            ident: _,
            generics: _,
            paren_token,
            inputs,
            variadic,
            output,
        } = &self.0;

        let inputs = inputs
            .iter()
            .map(|arg| match arg {
                FnArg::Receiver(_) => unreachable!(),
                FnArg::Typed(ty) => BareFnArg {
                    name: None,
                    attrs: Vec::with_capacity(0),
                    ty: *ty.ty.clone(),
                },
            })
            .collect();

        TypeBareFn {
            lifetimes: None, // FFI functions cannot have lifetimes
            unsafety: Some(<Token![unsafe]>::default()),
            abi: Some(Abi {
                extern_token: Default::default(),
                name: Some(LitStr::new("C", Span::call_site())),
            }),
            paren_token: paren_token.clone(),
            inputs,
            variadic: variadic.clone(),
            output: output.clone(),
            fn_token: fn_token.clone(),
        }
    }
}
