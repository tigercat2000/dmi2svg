mod opacity_map;
use image::{DynamicImage, GenericImageView};
use opacity_map::OPACITY;
use rayon::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;

use contour_tracing::array::bits_to_paths;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SVGState {
    pub name: String,
    pub svg: String,
}

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to write to string {0:?}")]
    FormatError(#[from] std::fmt::Error),
    #[error("I/O error {0:?}")]
    IoError(#[from] std::io::Error),
    #[error("Internal DMI error {0:?}")]
    DmiError(#[from] dmi::error::DmiError),
    #[error("Icon states should have at least one image, wtf")]
    NoFirstStateImage,
}

/// Most of this is straight from [`raster2svg`](https://github.com/STPR/raster2svg)
///
/// See https://github.com/STPR/raster2svg/blob/main/src/main.rs
fn generate_paths(image: &DynamicImage) -> Vec<String> {
    let (width, height) = (image.width(), image.height());

    // First phase: Color sort
    let mut colors: BTreeMap<[u8; 4], usize> = BTreeMap::new();
    for (_x, _y, pixel) in image.pixels() {
        // Check alpha
        if pixel.0[3] != 0 {
            *colors.entry(pixel.0).or_insert(0) += 1;
        }
    }

    let mut sorted_colors = Vec::from_iter(colors);
    sorted_colors.sort_by(|&(_, a), &(_, b)| a.cmp(&b).reverse());

    sorted_colors
        .into_par_iter()
        .map(|color| {
            let [red, green, blue, alpha] = color.0;
            let alpha = alpha as usize;

            let mut path_string = String::new();

            // Third phase: Create paths
            if alpha == 255 {
                write!(path_string, "<path ").unwrap();
                write!(path_string, r#"fill="rgb({},{},{})" d=""#, red, green, blue).unwrap();
            } else {
                write!(path_string, "<path ").unwrap();
                write!(
                    path_string,
                    r#"fill="rgb({},{},{})" opacity="{}" d=""#,
                    red, green, blue, OPACITY[alpha]
                )
                .unwrap();
            }

            // Fourth phase: fill an array of bits for one color
            // rows/columns
            let mut bits = vec![vec![0i8; width as usize]; height as usize];

            for (x, y, pixel) in image.pixels() {
                if pixel.0 == color.0 {
                    bits[y as usize][x as usize] = 1;
                }
            }

            // Fifth phase: Use contour_tracing
            write!(path_string, "{}", bits_to_paths(bits, true)).unwrap();
            writeln!(path_string, r#""/>"#).unwrap();

            path_string
        })
        .collect()
}

pub fn dmi2svg_symbol(file: &std::path::Path) -> Result<Vec<String>, Error> {
    let dmi = dmi::icon::Icon::load(std::fs::File::open(file)?)?;

    let mut state_vec: Vec<(String, &DynamicImage)> = Vec::with_capacity(dmi.states.len());

    for state in &dmi.states {
        let first_image = state.images.get(0).ok_or(Error::NoFirstStateImage)?;
        state_vec.push((state.name.clone(), first_image));
    }

    let svg_symbols: Vec<String> = state_vec
        .par_iter()
        .map(|(name, image)| {
            let (width, height) = (image.width(), image.height());
            let (elem_width, elem_height) = ("auto", "auto");

            let mut symbol = String::new();

            let mut header: String = r#"<symbol "#.to_string();
            header += &format!(
                r#"id="{}" width="{}" height="{}" viewBox="0 0 {} {}""#,
                name, elem_width, elem_height, width, height
            );
            header += ">\n";

            write!(symbol, "{}", header).unwrap();

            let paths = generate_paths(image);
            symbol.push_str(&paths.concat());
            writeln!(symbol, "</symbol>").unwrap();

            symbol
        })
        .collect();

    Ok(svg_symbols)
}

pub fn dmi2svg_symbol_map(
    file: &std::path::Path,
    map: &HashMap<String, String>,
) -> Result<Vec<String>, Error> {
    let dmi = dmi::icon::Icon::load(std::fs::File::open(file)?)?;

    let mut state_vec: Vec<(String, &DynamicImage)> = Vec::with_capacity(dmi.states.len());

    for state in &dmi.states {
        let first_image = state.images.get(0).ok_or(Error::NoFirstStateImage)?;
        state_vec.push((state.name.clone(), first_image));
    }

    let svg_symbols: Vec<String> = state_vec
        .par_iter()
        .map(|(name, image)| {
            let (width, height) = (image.width(), image.height());
            let (elem_width, elem_height) = ("auto", "auto");

            let mut symbol = String::new();

            let id = if let Some(rename) = map.get(name) {
                rename
            } else {
                name
            };

            let mut header: String = r#"<symbol "#.to_string();
            header += &format!(
                r#"id="{}" width="{}" height="{}" viewBox="0 0 {} {}""#,
                id, elem_width, elem_height, width, height
            );
            header += ">\n";

            write!(symbol, "{}", header).unwrap();

            let paths = generate_paths(image);
            symbol.push_str(&paths.concat());
            writeln!(symbol, "</symbol>").unwrap();

            symbol
        })
        .collect();

    Ok(svg_symbols)
}

pub fn dmi2svg(file: &std::path::Path) -> Result<Vec<SVGState>, Error> {
    let dmi = dmi::icon::Icon::load(std::fs::File::open(file)?)?;

    let mut state_vec: Vec<(String, &DynamicImage)> = Vec::with_capacity(dmi.states.len());

    for state in &dmi.states {
        let first_image = state.images.get(0).ok_or(Error::NoFirstStateImage)?;
        state_vec.push((state.name.clone(), first_image));
    }

    let svg_states: Vec<SVGState> = state_vec
        .par_iter()
        .map(|(name, image)| {
            let (width, height) = (image.width(), image.height());
            let (elem_width, elem_height) = ("auto", "auto");

            // Second phase: svg headers
            let mut svg = String::new();

            let mut header: String = r#"<svg xmlns="http://www.w3.org/2000/svg""#.to_string();
            header += &format!(
                r#" width="{}" height="{}" viewBox="0 0 {} {}""#,
                elem_width, elem_height, width, height
            );
            header += r#" shape-rendering="crispEdges""#;
            header += ">\n";

            write!(svg, "{}", header).unwrap();

            let paths = generate_paths(image);

            svg.push_str(&paths.concat());

            writeln!(svg, "</svg>").unwrap();

            SVGState {
                name: name.clone(),
                svg,
            }
        })
        .collect();

    Ok(svg_states)
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_dmi() {
        let path = Path::new("./test_dmis/test.dmi");

        if !path.exists() {
            panic!("test_dmis/test.dmi not found");
        }

        let svg_states = dmi2svg(path).unwrap();

        assert_eq!(
            svg_states.len(),
            3,
            "Expected 3 SVG states, ended up with {}",
            svg_states.len()
        );

        assert_eq!(svg_states[0].svg.len(), 248);
        assert_eq!(svg_states[1].svg.len(), 324);
        assert_eq!(svg_states[2].svg.len(), 290);
    }

    #[test]
    fn test_weird_size() {
        let path = Path::new("./test_dmis/weird_size.dmi");

        if !path.exists() {
            panic!("test_dmis/weird_size.dmi not found");
        }

        let svg_states = dmi2svg(path).unwrap();

        assert_eq!(
            svg_states.len(),
            2,
            "Expected 2 SVG states, ended up with {}",
            svg_states.len()
        );

        assert_eq!(svg_states[0].svg.len(), 317);
        assert_eq!(svg_states[1].svg.len(), 320);
    }
}
