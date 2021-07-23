use super::utils;
use wasm_bindgen::prelude::*;

include!("../../hector/maps/data/temp_pattern.in");

type BiomeLabel = usize;

const STRIDE: usize = 3; // For r,g,b
const RADIUS: usize = 3;
const INTENSITY: f64 = 25.;

// Biome colors
const COLORS: [Color; 17] = [
  (21,120,194),  // Water Bodies
  (27,100,6),    // Evergreen Needleleaf Forests
  (55,172,81),   // Evergreen Broadleaf Forests
  (27,114,24),   // Deciduous Needleleaf Forests
  (10,120,70),   // Deciduous Broadleaf Forests
  (23,112,57),   // Mixed Forests
  (127,171,98),  // Closed Shrublands
  (178,130,44),  // Open Shrublands
  (55,180,92),   // Woody Savannas
  (239,191,57),  // Savannas
  (57,166,100),  // Grasslands
  (78,84,32),    // Permanent Wetlands
  (200,247,142), // Croplands
  (171,234,226), // Urban and Built-up Lands
  (219,225,120), // Cropland/Natural Vegetation Mosaics
  (201,225,244), // Permanent Snow and Ice
  (234,171,68),  // Barren
];

// Technically should be u8
// but we need larger numbers,
// which we later divide down to fit u8
type BigColor = (usize, usize, usize);
type Color = (u8, u8, u8);

#[wasm_bindgen]
pub struct EarthSurface {
    width: usize,
    height: usize,
    scale: usize,
    biomes: Vec<BiomeLabel>,
    pixels: Vec<u8>,
    intensities: Vec<(BigColor, usize)>
}

#[wasm_bindgen]
impl EarthSurface {
    pub fn new(biomes: Vec<BiomeLabel>, width: usize, height: usize, scale: usize) -> EarthSurface {
        utils::set_panic_hook();

        let mut pixels: Vec<u8> = biomes_to_pixels(&biomes);
        pixels = nearest_neighbor_scale(&pixels, width, height, scale);
        let intensities = compute_intensities(&pixels);

        // Assert they have the same number of values
        // (assumes they are the same aspect ratio)
        assert!(biomes.len() == TEMP_PATTERN_W.len());
        assert!(biomes.len() == TEMP_PATTERN_B.len());

        let w = width * scale;
        let h = height * scale;
        EarthSurface {
            biomes,
            pixels,
            scale,
            intensities,
            width: w,
            height: h
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn update_surface(&mut self) {
        oil_paint_effect(&mut self.pixels, &self.intensities, self.width, self.height);
    }

    // JS will access surface pixel data directly
    // from WASM memory to avoid copying
    pub fn surface(&self) -> *const u8 {
        self.pixels.as_ptr()
    }

    pub fn update_biomes(&mut self, tgav: f64) {
        // Above we assert that TEMP_PATTERN_W, TEMP_PATTER_B, and tgav are all the same size,
        // so no scaling necessary.
        for (idx, (temp, biome)) in pscl_apply(&TEMP_PATTERN_W, &TEMP_PATTERN_B, tgav).zip(self.biomes.iter_mut()).enumerate() {
            if let Some(label) = biome_for_temp(biome, temp) {
                *biome = label;
                let color = color_for_biome(label);
                let r = color.0 as usize;
                let g = color.1 as usize;
                let b = color.2 as usize;

                // Update intensities
                // Then you can run `update_surface()` to update the surface pixels
                let intensity = compute_intensity(r,g,b);
                let scaled_idx = scale_idx(idx, self.width, self.scale);
                for i in 0..self.scale {
                    let ii = scaled_idx + (i * self.width * self.scale);
                    self.intensities[ii..ii+self.scale].fill(((r,g,b), intensity));
                }
            };
        }
    }
}

// TODO this is where we implement the biome changing logic
// If the biome hasn't changed, return None
fn biome_for_temp(biome: &mut BiomeLabel, temp: f64) -> Option<usize> {
    let label = 9; // Savannas
    if temp > 0. && *biome < 255 { // Not water
        Some(label)
    } else {
        None
    }
}

fn scale_idx(idx: usize, width: usize, scale: usize) -> usize {
    let scaled_width = width * scale;
    let x = idx % width;
    let y = idx / width;
    let y_scaled = y * scale;
    let x_scaled = x * scale;
    (y_scaled * scaled_width) + x_scaled
}

fn color_for_biome(label: usize) -> Color {
    if label == 255 {
        COLORS[0]
    } else {
        COLORS[label]
    }
}

// Convert biome labels to RGB
fn biomes_to_pixels(biomes: &[usize]) -> Vec<u8> {
    let mut pixels: Vec<u8> = Vec::with_capacity(biomes.len() * STRIDE);
    for label in biomes {
        let (r, g, b) = color_for_biome(*label);
        pixels.push(r);
        pixels.push(g);
        pixels.push(b);
    }
    pixels
}

fn add_colors(a: BigColor, b: BigColor) -> BigColor {
    (a.0 + b.0, a.1 + b.1, a.2 + b.2)
}

fn nearest_neighbor_scale(img: &[u8], width: usize, height: usize, scale: usize) -> Vec<u8> {
    let new_width = width * scale;
    let new_height = height * scale;
    let mut result: Vec<u8> = Vec::with_capacity(new_width * new_height * STRIDE);

    for i in 0..new_height {
        let i_ = i/scale;
        for j in 0..new_width {
            let j_ = j/scale;
            let idx_ = (i_ * width + j_) * STRIDE;
            result.push(img[idx_]);
            result.push(img[idx_+1]);
            result.push(img[idx_+2]);
        }
    }
    result
}

// Compute pixel intensities, for applying the oil paint effect
fn compute_intensities(img: &[u8]) -> Vec<(BigColor, usize)> {
    img.chunks_exact(3).map(|rgb| {
        let r = rgb[0] as usize;
        let g = rgb[1] as usize;
        let b = rgb[2] as usize;
        ((r,g,b), compute_intensity(r,g,b))
    }).collect()
}

fn compute_intensity(r: usize, g: usize, b: usize) -> usize {
    let avg = (r + g + b) as f64 / 3.;
    ((avg * INTENSITY) / 255.).round() as usize
}

// Ported from <https://codepen.io/loktar00/pen/Fhzot>
fn oil_paint_effect(pixels: &mut[u8], intensities: &[(BigColor, usize)], width: usize, height: usize) {
    // For each pixel, get the most common intensity value of the neighbors in radius
    let mut top;                                                            // Max intensity value
    let mut pixel_intensity_count: Vec<Option<(usize, BigColor)>> = vec![None; INTENSITY as usize + 1];
    for idx in 0..intensities.len() {
        top = (0, (0, 0, 0));
        for item in &mut pixel_intensity_count { *item = None; }

        // Find intensities of nearest pixels within radius.
        let x = idx % width;
        let y = idx / width;
        let up_span = y.min(RADIUS);              // rows to traverse up from idx
        let down_span = (height-y-1).min(RADIUS); // rows to traverse down from idx
        let left_span = x.min(RADIUS);            // rows to traverse left from idx
        let right_span = (width-x-1).min(RADIUS); // rows to traverse right from idx
        let y_span = up_span + down_span + 1;     // rows to traverse up and down, including idx
        let start_idx = idx - (up_span * width);

        for i in 0..y_span {
            let midpoint = start_idx + i * width;
            for (rgb, intensity_val) in &intensities[midpoint-left_span..midpoint+right_span] {
                let count = match pixel_intensity_count[*intensity_val] {
                    Some((val, color)) => (val + 1, add_colors(color, *rgb)),
                    None => (1, *rgb)
                };

                if count.0 > top.0 {
                    top = count;
                }
                pixel_intensity_count[*intensity_val] = Some(count);
            }
        }

        let i = idx * STRIDE;
        pixels[i]   = !!(top.1.0 / top.0) as u8; // r
        pixels[i+1] = !!(top.1.1 / top.0) as u8; // g
        pixels[i+2] = !!(top.1.2 / top.0) as u8; // b
    }
}


/*
Applies tgav from Hector over a scaling pattern,
to spatialize temperatures to a grid.
This approach is what `hectorui` uses.

Jason Evanoff, Chris Vernon, Stephanie Pennington, & Robert Link. (2021, May 13). JGCRI/hectorui: v1.2.0 PNNL web feature (Version v1.2.0). Zenodo. http://doi.org/10.5281/zenodo.4758524

Ported from:
- <https://rdrr.io/github/JGCRI/fldgen/man/pscl_apply.html>
- <https://rdrr.io/github/JGCRI/fldgen/src/R/meanfield.R>

The original `pscl_apply` takes a vector for `tgav`, where each
value is the temperature anomaly for one year. We only need to
calculate one year at a time, so for simplicity this takes a single
value for `tgav`.

Important note: If using `temperature.Tgav` from Hector,
add 15 to it (the base temperature) before passing it here.
This is what they do in `hectorui`.
*/
fn pscl_apply<'a>(pscl_w: &'a [f64], pscl_b: &'a [f64], tgav: f64) -> impl Iterator<Item=f64> + 'a {
    pscl_w.iter().zip(pscl_b).map(move |(w_i, b_i)| w_i * tgav + b_i)
}


#[cfg(test)]
mod test {
    use super::*;
    use float_cmp::approx_eq;

    #[test]
    fn test_pscl_apply() {
        let pscl_w: [f64; 6] = [ 0., 1., 0., 0.5, 1.0, 0.];
        let pscl_b: [f64; 6] = [-1., 1., 0., 0., 0.5, 0.5];
        let tgav = 8.;
        let expected = vec![-1., 9., 0., 4., 8.5, 0.5];
        let map: Vec<f64> = pscl_apply(&pscl_w, &pscl_b, tgav).collect();

        assert!(map.len() == expected.len());
        assert!(map.iter().zip(expected)
                .all(|(x1,x2)| approx_eq!(f64, *x1, x2, epsilon=1e-8)))
    }

    #[test]
    fn test_nearest_neighbor_scale() {
        let img: [u8; 18] = [
            0, 0, 0,
            1, 1, 1,
            2, 2, 2,
            3, 3, 3,
            4, 4, 4,
            5, 5, 5,
        ];
        let width = 3;
        let height = 2;
        let scale = 2;
        let expected: [u8; 72] = [
            0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2,
            0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2,
            3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5,
            3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5,
        ];
        let scaled = nearest_neighbor_scale(&img, width, height, scale);
        println!("{:?}", scaled);

        assert!(scaled.len() == expected.len());
        assert!(scaled.iter().zip(expected)
                .all(|(x1,x2)| *x1 == x2));
    }

    #[test]
    fn test_scale_idx() {
        let mut scale = 2;
        let mut width = 3;
        assert!(scale_idx(0, width, scale) == 0);
        assert!(scale_idx(1, width, scale) == 2);
        assert!(scale_idx(5, width, scale) == 16);
        assert!(scale_idx(7, width, scale) == 26);

        scale = 4;
        width = 3;
        assert!(scale_idx(0, width, scale) == 0);
        assert!(scale_idx(1, width, scale) == 4);
        assert!(scale_idx(2, width, scale) == 8);
        assert!(scale_idx(3, width, scale) == 48);
        assert!(scale_idx(4, width, scale) == 52);
        assert!(scale_idx(5, width, scale) == 56);
    }

    #[test]
    fn test_earth_surface_update_biomes() {
        let biomes: Vec<usize> = (0..TEMP_PATTERN_W.len()).map(|_| 0).collect();
        let width = 320;
        let height = 160;
        let scale = 2;
        let mut surface = EarthSurface::new(biomes, width, height, scale);

        surface.update_biomes(1000.);
        // TODO implement an actual test
    }
}
