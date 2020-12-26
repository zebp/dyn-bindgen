use anyhow::Result;
use syn::{
    export::Span, visit_mut::VisitMut, Abi, BareFnArg, File, FnArg, ForeignItemFn, LitStr,
    Signature, Token, TypeBareFn,
};

#[derive(Default)]
pub struct Parsed {
    pub functions: Vec<BoundFunction>,
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
}

pub struct BoundFunction(pub Signature);

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
