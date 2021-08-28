use std::result::Result;
use wasm_bindgen::prelude::*;

mod edit_distance;
mod html_generator;
mod json_diff;

extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub fn find_diff(arg1: &str, arg2: &str) -> String {
    if let Result::Ok(json) = json_diff::diff(arg1, arg2) {
        log!("{:?}", json);
        let html_str = html_generator::generate(json);

        html_str
    } else {
        "error!".to_string()
    }
}
