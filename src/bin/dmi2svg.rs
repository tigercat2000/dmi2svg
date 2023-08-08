use std::{path::Path, time::Instant};

use dmi2svg::dmi2svg;

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

    let start = Instant::now();

    let svgs = dmi2svg(path).expect("Failed to create SVGs");

    for state in svgs {
        let name = format!("{}.svg", state.name);
        let path = Path::new(".").join(name);
        std::fs::write(&path, state.svg).unwrap_or_else(|_| panic!("Failed to write {:?}", path));
        println!("Wrote state {:?} to {:?}", state.name, path);
    }

    let duration = start.elapsed();

    println!("Finished in {}ms", duration.as_millis());
}
