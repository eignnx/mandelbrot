extern crate num_complex;
extern crate image;

use std::str::FromStr;
use num_complex::Complex64;

const COLOR_BITS: u8 = 8;
const COLOR_DEPTH: usize = 255;

fn parse_pair<T: FromStr>(txt: &str, sep: char) -> Option<(T, T)> {
    match txt.find(sep) {
        None => None,
        Some(index) => match (T::from_str(&txt[..index]), T::from_str(&txt[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None
        }
    }
}

#[test]
fn text_parse_pair() {
    assert_eq!(parse_pair::<i32>("123x456", 'x'), Some((123, 456)));
    assert_eq!(parse_pair::<f64>("1.23,5.67", ','), Some((1.23, 5.67)));
    assert_eq!(parse_pair::<u64>("132x", 'x'), None);
    assert_eq!(parse_pair::<u64>("x", 'x'), None);
    assert_eq!(parse_pair::<u64>("13,2", 'x'), None);
}

fn parse_complex(txt: &str) -> Option<Complex64> {
    match parse_pair(txt, ',') {
        Some((re, im)) => Some(Complex64 { re, im }),
        None => None
    }
}

#[test]
fn text_parse_complex() {
    assert_eq!(parse_complex("12.3,45.6"), Some(Complex64 {re: 12.3, im: 45.6}));
    assert_eq!(parse_complex("3,-4"), Some(Complex64 {re: 3.0, im: -4.0}));
    assert_eq!(parse_complex("12.345"), None);
    assert_eq!(parse_complex("12.345.3.1"), None);
    assert_eq!(parse_complex("12.345,"), None);
}

fn parse_px_window(txt: &str) -> Option<PxWindow> {
    match parse_pair::<usize>(txt, 'x') {
        Some(bounds) => Some(PxWindow { width: bounds.0, height: bounds.1 }),
        None => None
    }
}

#[derive(Debug, Copy, Clone)]
struct CWindow {
    upper_left: Complex64,
    lower_right: Complex64,
}

impl CWindow {
    fn new(upper_left: Complex64, lower_right: Complex64) -> CWindow {
        CWindow { upper_left, lower_right }
    }

    fn from_center_scale(center: Complex64, scale: f64, aspect_ratio: f64) -> CWindow {
        // aspect_ratio == width / height
        let width = scale;
        let height = scale / aspect_ratio;
        CWindow::new(
            center + Complex64 { re: -width / 2., im: height / 2. },
            center + Complex64 { re: width / 2., im: -height / 2. }
        )
    }

    fn from_corner_w_h(upper_left: Complex64, width: f64, height: f64) -> CWindow {
        CWindow::new(
            upper_left,
            upper_left + Complex64 { re: width, im: -height }
        )
    }

    fn width(&self) -> f64 {
        self.lower_right.re - self.upper_left.re
    }

    fn height(&self) -> f64 {
        self.upper_left.im - self.lower_right.im
    }

    fn center(&self) -> Complex64 {
        (self.upper_left + self.lower_right) / 2.
    }
}

#[test]
fn test_cwindow() {
    let win1 = CWindow::new(
        Complex64 { re: -1., im: 3.},
        Complex64 { re: 1., im: -3. }
    );
    assert_eq!(win1.width(), 2.);
    assert_eq!(win1.height(), 6.);

    let win2 = CWindow::from_center_scale(
        win1.upper_left,
        100.,
        2.
    );
    assert_eq!(win2.width(), 100.);
    assert_eq!(win2.height(), 50.);
    assert_eq!(win2.center(), win1.upper_left);

    let win3 = CWindow::from_corner_w_h(
        win1.upper_left,
        4.,
        3.
    );
    assert_eq!(win3.width(), 4., "Wrong window width!");
    assert_eq!(win3.height(), 3., "Wrong window height!");
    assert_eq!(win3.lower_right, Complex64 { re: 3., im: 0.}, "Wrong lower right bound!");
}

#[derive(Debug, Copy, Clone)]
struct PixelPt {
    col: usize,
    row: usize,
}

impl PixelPt {
    fn new(col: usize, row: usize) -> PixelPt {
        PixelPt { col, row }
    }
}

#[derive(Debug, Copy, Clone)]
struct PxWindow {
    width: usize,
    height: usize,
}

fn pixel_to_point(pixel: PixelPt, px_win: PxWindow, c_win: CWindow) -> Complex64 {
    Complex64 {
        re: c_win.upper_left.re + c_win.width() * pixel.col as f64 / px_win.width as f64,
        im: c_win.upper_left.im - c_win.height() * pixel.row as f64 / px_win.height as f64
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point(
            PixelPt::new(25, 75),
            PxWindow {width: 100, height: 100 },
            CWindow::new(
                Complex64 { re: -1.0, im: 1.0 },
                Complex64 { re: 1.0, im: -1.0 }
            )
        ),
        Complex64::new(-0.5, -0.5)
    );

    assert_eq!(
        pixel_to_point(
            PixelPt::new(0, 50),
            PxWindow {width: 100, height: 100 },
            CWindow::new(
                Complex64 { re: -1.0, im: 1.0 },
                Complex64 { re: 1.0, im: -1.0 }
            )
        ),
        Complex64::new(-1., 0.)
    );
}

fn px_to_index(pixel: PixelPt, window: PxWindow) -> usize {
    pixel.col + pixel.row * window.width
}

fn escape_time(c: Complex64, iters: usize) -> Option<usize> {
    let mut z = Complex64::new(0.0, 0.0);

    for i in 0..iters {
        if z.norm_sqr() > 4.0 {
            return Some(i)
        }
        z = z * z + c;
    }
    None
}

use std::f32;

const LOG_COEFF: f32 = 46.018385143;

fn log_color(x: f32) -> f32 {
    LOG_COEFF * (x + 1.0).ln()
}

fn render(pixels: &mut [u8],
          px_win: PxWindow,
          c_win: CWindow,
          color_fn: &Fn(f32) -> f32)
{
    assert_eq!(pixels.len(), px_win.width * px_win.height);

    for col in 0..px_win.width {
        for row in 0..px_win.height {
            let pixel = PixelPt::new(col, row);
            let c = pixel_to_point(pixel, px_win, c_win);

            pixels[px_to_index(pixel, px_win)] =
                match escape_time(c, COLOR_DEPTH) {
                    Some(iters) => color_fn(iters as f32) as u8,
                    None => COLOR_DEPTH as u8
                };
        }
    }
}

extern crate crossbeam;

fn parallel_render<F>(pixels: &mut [u8],
                      px_win: PxWindow,
                      c_win: CWindow,
                      color_fn: &F,
                      threads: usize)
where F : Fn(f32) -> f32 + Send + Sync
{
    let max_rows_per_band = px_win.height / threads + 1; // Rounds up.
    let max_px_per_band = max_rows_per_band * px_win.width;

    let bands =
        pixels.chunks_mut(max_px_per_band);

    crossbeam::scope(|spawner| {
        for (i, band) in bands.enumerate() {

            let band_top = i * max_rows_per_band; // Row index of top row in ith band.
            let actual_rows = band.len() / px_win.width;

            let band_px_win = PxWindow { width: px_win.width, height: actual_rows };

            let band_upper_left_px = PixelPt::new(0, band_top);
            let band_lower_right_px = PixelPt::new(px_win.width, band_top + actual_rows);
            let band_c_win = CWindow::new(
                pixel_to_point(band_upper_left_px, px_win, c_win),
                pixel_to_point(band_lower_right_px, px_win, c_win)
            );

            spawner.spawn(move || {
                render(band, band_px_win, band_c_win, color_fn);
            });
        }
    });
}

use std::io::Result;
use std::fs::File;
use image::png::PNGEncoder;
use image::ColorType;

fn save_image(filename: &str, pixels: &[u8], px_win: PxWindow) -> Result<()> {
    let file = File::create(filename)?;
    let encoder = PNGEncoder::new(file);

    encoder.encode(&pixels,
                   px_win.width as u32,
                   px_win.height as u32,
                   ColorType::Gray(COLOR_BITS))?;

    Ok(())
}


fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 6 {
        eprintln!("Incorrect number of arguments!");
        eprintln!("Usage: {} FILENAME IMG_SIZE UPPER_LEFT LOWER_RIGHT #THREADS", &args[0]);
        eprintln!("Example: {} mandel.png 800x600 -4,3 4,-3 4", &args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let px_window = parse_px_window(&args[2])
        .expect("Incorrect image size formatting!");
    let upper_left = parse_complex(&args[3])
        .expect("Incorrect upper left bounds formatting!");
    let lower_right = parse_complex(&args[4])
        .expect("Incorrect lower right bounds formatting!");
    let threads = usize::from_str(&args[5])
        .expect("Incorrect thread number formatting!");

    let c_window = CWindow::new(upper_left, lower_right);

    let mut pixels = vec![0u8; px_window.width * px_window.height];

    parallel_render(&mut pixels, px_window, c_window, &log_color, threads);

    save_image(filename, &pixels, px_window)
        .expect("Error exporting image!");
}
