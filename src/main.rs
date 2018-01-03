extern crate hound;
extern crate image;
extern crate num;
extern crate rustfft;
extern crate palette;

mod frequency;

use hound::WavReader;
use std::fs::File;
use image::ImageBuffer;
use rustfft::FFTplanner;
use num::complex::Complex;
use palette::gradient::Gradient;
use std::env;
use frequency::Frequency;

fn single_sample_specter(samples: Vec<f32>) -> Vec<f32> {
    let num_samples = samples.len();
    let mut planner = FFTplanner::new(false);
    let fft = planner.plan_fft(num_samples);

    let mut signal = samples
        .iter()
        .map(|x| Complex::new(*x, 0f32))
        .collect::<Vec<_>>();

    let mut spectrum = signal.clone();

    //let bin = 44100f32 / num_samples as f32;

    fft.process(&mut signal, &mut spectrum);
    spectrum
        .iter()
        .map(|x| x.norm() / num_samples as f32)// * bin as f32)
        .take(num_samples / 2)
        .collect()
}


fn main() {
    let gradient = Gradient::new(vec![
        palette::Rgb::new(0., 0., 0.),
        palette::Rgb::new(0., 0., 1.),
        palette::Rgb::new(0., 1., 1.),
        palette::Rgb::new(1., 1., 0.),
        palette::Rgb::new(1., 0., 0.),
    ]);

    let filename = env::args().skip(1).next().expect("No WAV file specified.");
    let mut reader = WavReader::open(filename).expect("Failed to open WAV file.");
    let signal_len = reader.len() as usize;

    let width = 1000;
    let height = 500;
    let window_size = signal_len as f32 / width as f32;
    let pixel_size = window_size as f32 / (4. * height as f32);

    println!("{}x{}", width, height);
    let mut out = ImageBuffer::new(width as u32, height as u32);
    for (x, signal) in reader
        .samples::<i16>()
        .map(|x| x.unwrap() as f32)
        .frequency(window_size)
        .take(width) // division isn't exact
        .enumerate()
    {
        let specter = single_sample_specter(signal);

        // average chunks
        let averaged: Vec<f32> = specter
            .iter()
            .frequency(pixel_size)
            .into_iter()
            .take(height) // remove conjugate
            .map(|it| it.into_iter().sum())
            .map(f32::ln) // log scale
            .collect();

        // top average value
        let max = averaged.iter().cloned().fold(-1. / 0. /* -inf */, f32::max);

        for (y, &val) in averaged.iter().enumerate() {
            let ratio = val / max;
            let pixel = match gradient.get(ratio) {
                palette::Rgb { red, green, blue } => {
                    image::Rgb(
                        [
                            (255. * red) as u8,
                            (255. * green) as u8,
                            (255. * blue) as u8,
                        ],
                    )
                }
            };
            out.put_pixel(x as u32, (height - 1 - y) as u32, pixel);
            // println!("{}, {}, {}", x, y, log_scaled,);
        }
    }

    // write output
    let file = &mut File::create("out.png").unwrap();
    image::ImageRgb8(out).save(file, image::PNG).unwrap();
}
