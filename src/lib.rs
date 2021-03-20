
mod bundle;
mod generator;
mod glue;
mod parse;

use quote::ToTokens;

pub use crate::bundle::*;

pub struct Config {
    pub bundle_strategy: BundleStrategy,
    pub use_rust_fmt: bool,
}

pub fn generate(builder: bindgen::Builder, config: Config) -> anyhow::Result<String> {
    let code = builder.generate().unwrap();
    let code = code.to_string();

    let parsed = parse::parse(&code)?;
    let generated = generator::generate_bindings_module(config.bundle_strategy, parsed)?;

    let code = generated.to_token_stream().to_string();

    if config.use_rust_fmt {
        // TODO: rustfmt sometimes crashes with an underflow, so for now we won't format the output
        // maybe switch to creating a temp file and running rustfmt as a command on that?
        let input = rustfmt::Input::Text(code);
        let config = rustfmt::config::Config::default();
        let mut buffer = Vec::new();

        let (_, file_map, _) =
            rustfmt::format_input(input, &config, Some(&mut buffer)).map_err(|(e, _)| e)?;
        let (_, formatted) = &file_map[0];

        Ok(formatted.to_string())
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
            bundle_strategy: BundleStrategy::ImplicitlyLoadedBundle(bundle),
            use_rust_fmt: true,
        };
        let code = generate(builder, config).unwrap();

        println!("{}", code);
    }
}
