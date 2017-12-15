extern crate hound;
extern crate image;
extern crate rustfft;
extern crate num;
extern crate itertools;

use std::f32::consts::PI;
use hound::{SampleFormat, WavSpec, WavWriter, WavReader};
use std::fs::File;
use image::ImageBuffer;
use image::Luma;
use rustfft::FFTplanner;
use num::complex::Complex;
use itertools::Itertools;

fn generate_sine(filename: &str, frequency: f32, duration: u32) {
    let header = WavSpec {
        channels: 1,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(filename, header).expect("Failed to created WAV writer");
    let num_samples = duration * header.sample_rate;
    let signal_amplitude = 1000f32; //16384f32;
    for n in 0..num_samples {
        let t: f32 = n as f32 / header.sample_rate as f32;
        let x = signal_amplitude * (t * frequency * 2.0 * PI).sin();
        writer.write_sample(x as i16).unwrap();
    }
}


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
        .collect()
}

fn main() {
    // let mut out = ImageBuffer::new(1000, 220);

    // generate_sine("test.wav", 1000f32, 5);

    let mut reader = WavReader::open("new.wav").expect("Failed to open WAV file");
    let signal_len = reader.len() as usize;
    for (x, sample) in reader
        .samples::<i16>()
        .chunks(signal_len / 10)
        .into_iter()
        .take(10) // division isn't exact
        .enumerate()
    {
        let signal = sample.map(|x| x.unwrap() as f32).collect();
        let specter = single_sample_specter(signal);
        for (y, val) in specter.iter().enumerate() {
            // out.put_pixel(x as u32, y as u32, Luma::new());
            println!(
                "{}, {}, {}",
                x,
                y,
                val,
            );
        }

    }


    // write output
    // let file = &mut File::create("spectrogram.png").unwrap();
    // image::ImageLuma8(out).save(file, image::PNG).unwrap();
}
