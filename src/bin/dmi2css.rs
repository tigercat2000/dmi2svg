use base64::{engine::general_purpose, Engine as _};
use dmi2svg::dmi2svg;
use std::fmt::Write;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: dmi2svg [PATH]");
        return;
    }

    let path = Path::new(&args[1]);

    if !path.exists() {
        println!("Usage: dmi2svg [PATH]");
        println!("Error: Cannot find path {:?}", path);
        return;
    }

    let filename = path
        .file_stem()
        .expect("Unable to figure out filename")
        .to_string_lossy();

    let svgs = dmi2svg(path).expect("Failed to create SVGs");

    let mut css = String::new();

    for state in svgs {
        let mut name = state.name;
        if name.is_empty() {
            name = "DEFAULT".to_owned();
        }
        let svg_b64 = general_purpose::STANDARD_NO_PAD.encode(state.svg);
        writeln!(
            css,
            ".{}.{}{{background-image: url(\"data:image/svg+xml;base64,{}\")}}",
            filename, name, svg_b64
        )
        .unwrap();
    }

    std::fs::write(format!("{}.css", filename), css).unwrap();
}
