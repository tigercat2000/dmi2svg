use std::{path::Path, time::Instant};

use dmi2svg::dmi2svg_symbol;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        println!("Usage: dmi2svg_symbol [PATH]");
        return;
    }

    let path = Path::new(&args[1]);

    if !path.exists() {
        println!("Usage: dmi2svg_symbol [PATH]");
        println!("Error: Cannot find path {:?}", path);
        return;
    }

    let start = Instant::now();

    let svg = dmi2svg_symbol(path).expect("Failed to create SVG");
    std::fs::write(
        Path::new(".").join(format!(
            "{}.svg",
            path.file_stem()
                .expect("Unable to automatically determine svg name, aborting")
                .to_string_lossy()
        )),
        svg,
    )
    .expect("Failed to write SVG to disk");

    let duration = start.elapsed();

    println!("Finished in {}ms", duration.as_millis());
}
