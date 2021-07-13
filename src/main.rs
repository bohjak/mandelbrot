use num_complex::Complex;

fn main() {
    println!("Hello, world!");
}

fn escape_time(c: Complex::<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex{re: 0.0, im: 0.0};

    for i in 0..limit {
        if z.norm_sqr() > 4. {
            return Some(i);
        }

        z = z * z + c;
    }

    None
}

#[test]
fn test_escape_time() {
    assert_eq!(escape_time(Complex{re: 0.0, im: 0.0}, 100), None);
    assert_eq!(escape_time(Complex{re: 2.0, im: 2.0}, 10), Some(1));
    assert_eq!(escape_time(Complex{re: 0.25, im: 0.5}, 100), None);
}
