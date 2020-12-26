use syn::export::ToTokens;

mod glue;
mod generator;
mod parse;

pub fn generate(builder: bindgen::Builder) -> anyhow::Result<String> {
    let code = builder.generate().unwrap();
    let code = code.to_string();

    let parsed = parse::parse(&code)?;
    let generated = generator::generate_bindings_module(parsed)?;

    let code = generated.to_token_stream().to_string();

    let input = rustfmt::Input::Text(code);
    let config = rustfmt::config::Config::default();
    let mut buffer = Vec::new();

    let (_, file_map, _) =
        rustfmt::format_input(input, &config, Some(&mut buffer)).map_err(|(e, _)| e)?;
    let (_, formatted) = &file_map[0];

    Ok(formatted.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dyn_bindings() {
        let builder = bindgen::builder().header_contents("add.h", "int add(int a, int b);int sub(int a, int b);");
        let code = generate(builder).unwrap();

        println!("{}", code);
    }
}
