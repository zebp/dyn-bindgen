//! Generate Rust bindings for dynamic library from C/C++ headers.

mod bundle;
mod generator;
mod glue;
mod parse;

use std::process::Command;

use quote::ToTokens;

pub use crate::bundle::*;

#[derive(Debug, Clone)]
pub struct Config {
    /// How the dyanmic library should be loaded at runtime.
    pub loading_strategy: LoadingStrategy,
    /// If the rust formatter should be used on the generated code, this has no effect on runtime but
    /// might be useful if you need to debug
    pub use_rust_fmt: bool,
}

/// Generates a rust file with bindings to a dynamic library.
pub fn generate(builder: bindgen::Builder, config: Config) -> anyhow::Result<String> {
    let code = builder.generate().unwrap();
    let code = code.to_string();

    let parsed = parse::parse(&code)?;
    let generated = generator::generate_bindings_module(config.loading_strategy, parsed)?;

    let code = generated.to_token_stream().to_string();

    if config.use_rust_fmt {
        // This is a hack, but the `rustfmt` crate is deprecated and it's replacement requires nightly
        let file = std::env::temp_dir().join("unformatted.bindings.rs");
        std::fs::write(&file, code)?;

        let _ = Command::new("rustfmt")
            .arg(file.as_os_str())
            .output()
            .expect("unable to run rustfmt on the bindings");

        let code = std::fs::read_to_string(&file)?;
        std::fs::remove_file(file)?;

        Ok(code)
    } else {
        Ok(code)
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dyn_bindings() {
        let builder = bindgen::builder()
            .header_contents("add.h", "int add(int a, int b);int sub(int a, int b);");
        let bundle = Bundle::RawBytes(vec![0u8; 32]);
        let config = Config {
            loading_strategy: LoadingStrategy::ImplicitlyLoadedBundle(bundle),
            use_rust_fmt: true,
        };
        let code = generate(builder, config).unwrap();

        println!("{}", code);
    }
}
