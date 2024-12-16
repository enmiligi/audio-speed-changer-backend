mod utils;

use std::f64::consts::PI;

use rustfft::{
    num_complex::{Complex, ComplexFloat},
    FftPlanner,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

}

const FFT_SIZE: usize = 4096;

#[wasm_bindgen]
pub struct Processor {
    current: [f32; FFT_SIZE],
    last: [f32; FFT_SIZE],
    buffer1: [f32; FFT_SIZE],
    buffer2: [f32; FFT_SIZE],
    fft: [Complex<f64>; FFT_SIZE],
    fft_scratch: [Complex<f64>; FFT_SIZE],
    shifted_fft: [Complex<f64>; FFT_SIZE],
    buffer_at: usize,
    fft_planner: FftPlanner<f64>,
    window: [f64; FFT_SIZE],
    shift: f32,
}

#[wasm_bindgen]
impl Processor {
    #[wasm_bindgen]
    pub fn new(shift: f32) -> Self {
        let mut window: [f64; FFT_SIZE] = [0.0; FFT_SIZE];
        for i in 0..FFT_SIZE {
            window[i] = ((i as f64) * PI / FFT_SIZE as f64).sin();
        }
        Self {
            current: [0.0; FFT_SIZE],
            last: [0.0; FFT_SIZE],
            buffer1: [0.0; FFT_SIZE],
            buffer2: [0.0; FFT_SIZE],
            fft: [Complex { re: 0.0, im: 0.0 }; FFT_SIZE],
            fft_scratch: [Complex { re: 0.0, im: 0.0 }; FFT_SIZE],
            shifted_fft: [Complex { re: 0.0, im: 0.0 }; FFT_SIZE],
            buffer_at: 0,
            fft_planner: FftPlanner::new(),
            window,
            shift,
        }
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.buffer_at = 0;
        self.current = [0.0; FFT_SIZE];
        self.last = [0.0; FFT_SIZE];
        self.buffer1 = [0.0; FFT_SIZE];
        self.buffer2 = [0.0; FFT_SIZE];
    }

    pub fn change_shift(&mut self, shift: f32) {
        self.shift = shift;
    }

    pub fn move_to_last(&mut self) {
        std::mem::swap(&mut self.current, &mut self.last);
    }

    pub fn prepare_fft(&mut self) {
        for i in 0..FFT_SIZE {
            self.fft[i] = Complex {
                re: self.buffer1[i] as f64 * self.window[i],
                im: 0.0,
            };
        }
    }

    pub fn compute_fft(&mut self) {
        let fft = self.fft_planner.plan_fft_forward(FFT_SIZE);
        fft.process_with_scratch(&mut self.fft, &mut self.fft_scratch);
    }

    pub fn shift_fft(&mut self, by: f32) {
        for val in &mut self.shifted_fft {
            *val = Complex { re: 0.0, im: 0.0 };
        }
        self.fft[FFT_SIZE / 2] /= 2.0;
        let mut freq: usize = 0;
        while (freq as f32) * by < (FFT_SIZE / 2 + 1) as f32 {
            let new_freq = (freq as f32) * by;
            self.shifted_fft[new_freq as usize] += self.fft[freq];
            freq += 1;
        }
        self.shifted_fft[FFT_SIZE / 2] *= 2.0;
        for i in 1..=(FFT_SIZE / 2 - 1) {
            self.shifted_fft[FFT_SIZE - i] = self.shifted_fft[i].conj();
        }
    }

    pub fn compute_ifft(&mut self) {
        let ifft = self.fft_planner.plan_fft_inverse(FFT_SIZE);
        ifft.process_with_scratch(&mut self.shifted_fft, &mut self.fft_scratch);
    }

    pub fn use_ifft(&mut self) {
        for i in 0..FFT_SIZE {
            self.current[i] = (self.shifted_fft[i].re * self.window[i] / FFT_SIZE as f64) as f32;
        }
    }

    #[wasm_bindgen]
    pub fn process(&mut self, input_list: &mut [f32], output_list: &mut [f32]) -> bool {
        let mut i = 0;
        while i < input_list.len() {
            self.buffer1[self.buffer_at] = input_list[i];
            self.buffer2[(self.buffer_at + FFT_SIZE / 2) % FFT_SIZE] = input_list[i];
            self.buffer_at += 1;

            if self.buffer_at == FFT_SIZE {
                self.move_to_last();
                self.prepare_fft();
                self.compute_fft();
                self.shift_fft(self.shift);
                self.compute_ifft();
                self.use_ifft();

                self.buffer_at = FFT_SIZE / 2;
                std::mem::swap(&mut self.buffer1, &mut self.buffer2);
            }

            output_list[i] = self.current[(self.buffer_at + FFT_SIZE / 2) % FFT_SIZE]
                + self.last[self.buffer_at];
            i += 1;
        }

        return true;
    }
}
