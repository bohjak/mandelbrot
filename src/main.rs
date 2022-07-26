use colored::*;
use crossbeam;
use image::png::PNGEncoder;
use image::ColorType;
use num::Complex;
use std::fs::File;
use std::str::FromStr;

fn main() {
    let args = parse_args();

    let mut pixels = vec![0; args.bounds.0 * args.bounds.1];

    let threads = 8;
    let rows_per_band = args.bounds.1 / threads + 1;

    {
        let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * args.bounds.0).collect();
        crossbeam::scope(|spawner| {
            for (i, band) in bands.into_iter().enumerate() {
                let top = rows_per_band * i;
                let height = band.len() / args.bounds.0;
                let band_bounds = (args.bounds.0, height);
                let band_upper_left =
                    pixel_to_point(args.bounds, (0, top), args.upper_left, args.lower_right);
                let band_lower_right = pixel_to_point(
                    args.bounds,
                    (args.bounds.0, top + height),
                    args.upper_left,
                    args.lower_right,
                );

                spawner.spawn(move |_| {
                    render(band, band_bounds, band_upper_left, band_lower_right);
                });
            }
        })
        .unwrap();
    }

    write_image(&args.filename, &pixels, args.bounds).expect("error writing image");
}

fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize),
) -> Result<(), std::io::Error> {
    let output = File::create(filename)?;
    let encoder = PNGEncoder::new(output);
    encoder.encode(
        &pixels,
        bounds.0 as u32,
        bounds.1 as u32,
        ColorType::Gray(8),
    )?;

    Ok(())
}

fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    assert!(pixels.len() == bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for col in 0..bounds.0 {
            let point = pixel_to_point(bounds, (col, row), upper_left, lower_right);

            pixels[row * bounds.0 + col] = match escape_time(point, 255) {
                Some(t) => 255 - t as u8,
                None => 0,
            };
        }
    }
}

/// Returns the amount of iteration it takes z to escape the 2.0 circle given a c.
/// If limit is reached, z is assumed to not escape.
fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        } else {
            z = z * z + c;
        }
    }

    None
}

/// Parses a pair argument of type T separated by a separator
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
    }
}

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}

fn print_usage(name: &str) {
    eprintln!("{} - plots the Mandelbrot set", name.blue());
    eprintln!("usage: {} FILE PIXELS UPPERLEFT LOWERRIGHT", name);
    eprintln!("example: {} mandel.png 1024x768 -1.20,0.35 -1,0.20", name);
}

struct Args {
    filename: String,
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        print_usage(&args[0]);
        eprintln!(
            "{} wrong number of arguments: expected 4, got {}",
            "ERROR:".bold().red(),
            args.len() - 1
        );
        std::process::exit(1);
    }

    let filename = args[1].clone();
    let bounds = parse_pair(&args[2], 'x').expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3]).expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4]).expect("error parsing lower right corner point");

    Args {
        filename,
        bounds,
        upper_left,
        lower_right,
    }
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<u64>("123x987", 'x'), Some((123, 987)));
    assert_eq!(parse_pair::<f64>("3.141,2.728", ','), Some((3.141, 2.728)));
    assert_eq!(parse_pair::<i32>("no separator", '|'), None);
}

fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let width = lower_right.re - upper_left.re;
    let height = upper_left.im - lower_right.im;

    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point(
            (100, 200),
            (25, 175),
            Complex { re: -1.0, im: 1.0 },
            Complex { re: 1.0, im: -1.0 }
        ),
        Complex {
            re: -0.5,
            im: -0.75
        }
    );
}

