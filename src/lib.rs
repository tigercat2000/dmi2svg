mod opacity_map;
use opacity_map::OPACITY;

use std::collections::BTreeMap;
use std::fmt::Write;

use contour_tracing::array::bits_to_paths;
use dmm_tools::dmi::Image;
use dreammaker::dmi::Dir;

const NO_TINT: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct SVGState {
    pub name: String,
    pub svg: String,
}

#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    FmtError(std::fmt::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}

impl From<std::fmt::Error> for Error {
    fn from(value: std::fmt::Error) -> Self {
        Self::FmtError(value)
    }
}

pub fn dmi2svg(file: &std::path::Path) -> Result<Vec<SVGState>, Error> {
    let dmi = dmm_tools::dmi::IconFile::from_file(file)?;

    let mut state_vec: Vec<(String, Image)> = Vec::with_capacity(dmi.metadata.states.len());

    for state in &dmi.metadata.states {
        let idx = state.index_of_frame(Dir::South, 0);
        let rect = dmi.rect_of_index(idx);
        // println!("rect {:#?}", rect);

        let mut canvas = Image::new_rgba(dmi.metadata.width, dmi.metadata.height);
        canvas.composite(&dmi.image, (0, 0), rect, NO_TINT);

        // let slice = canvas.data.as_slice().unwrap();
        // for y in 0..canvas.height as usize {
        //     for x in 0..canvas.width as usize {
        //         let color = slice[(y * canvas.width as usize) + x];
        //         if color.a != 0 {
        //             print!("1");
        //         } else {
        //             print!("0");
        //         }
        //     }
        //     println!();
        // }

        state_vec.push((state.name.clone(), canvas));
    }

    let mut svg_states: Vec<SVGState> = Vec::with_capacity(dmi.metadata.states.len());

    // Most of this is straight from [`raster2svg`](https://github.com/STPR/raster2svg)
    //
    // See https://github.com/STPR/raster2svg/blob/main/src/main.rs
    for (name, image) in state_vec {
        let (width, height) = (image.width, image.height);
        let (elem_width, elem_height) = ("auto", "auto");

        // First phase: Color sort
        let mut colors: BTreeMap<[u8; 4], usize> = BTreeMap::new();
        for p in &image.data {
            if p.a != 0 {
                *colors.entry(*p.as_bytes()).or_insert(0) += 1;
            }
        }

        let mut sorted_colors = Vec::from_iter(colors);
        sorted_colors.sort_by(|&(_, a), &(_, b)| a.cmp(&b).reverse());

        // Second phase: svg headers
        let mut svg = String::new();

        let mut header: String = r#"<svg xmlns="http://www.w3.org/2000/svg""#.to_string();
        header += &format!(
            r#" width="{}" height="{}" viewBox="0 0 {} {}""#,
            elem_width, elem_height, width, height
        );
        header += r#" shape-rendering="crispEdges""#;
        header += ">\n";

        write!(svg, "{}", header)?;

        for color in sorted_colors.into_iter() {
            let [red, green, blue, alpha] = color.0;
            let alpha = alpha as usize;

            // Third phase: Create paths
            if alpha == 255 {
                write!(svg, "<path ")?;
                write!(svg, r#"fill="rgb({},{},{})" d=""#, red, green, blue)?;
            } else {
                write!(svg, "<path ")?;
                write!(
                    svg,
                    r#"fill="rgb({},{},{})" opacity="{}" d=""#,
                    red, green, blue, OPACITY[alpha]
                )?;
            }

            // Fourth phase: fill an array of bits for one color
            // rows/columns
            let mut bits = vec![vec![0i8; width as usize]; height as usize];

            let slice = image.data.as_slice().unwrap();
            for y in 0..height as usize {
                for x in 0..width as usize {
                    let pixel = slice[(y * width as usize) + x];
                    if pixel.r == red
                        && pixel.g == green
                        && pixel.b == blue
                        && pixel.a == alpha as u8
                    {
                        bits[y][x] = 1;
                    }
                }
            }

            // Fifth phase: Use contour_tracing
            write!(svg, "{}", bits_to_paths(bits, true))?;
            writeln!(svg, r#""/>"#)?;
        }
        writeln!(svg, "</svg>")?;

        svg_states.push(SVGState { name, svg })
    }

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
