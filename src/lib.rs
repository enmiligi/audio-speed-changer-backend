mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Processor {}

#[wasm_bindgen]
impl Processor {
    #[wasm_bindgen]
    pub fn new(volume: f32) -> Self {
        Self {}
    }

    #[wasm_bindgen]
    pub fn process(&self, input_list: &mut [f32], output_list: &mut [f32]) -> bool {
        let mut i = 0;
        while i < input_list.len() {
            output_list[i] = input_list[i];
            i += 1;
        }

        return true;
    }
}
