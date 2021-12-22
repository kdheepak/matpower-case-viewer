#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_must_use)]

mod case;

use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{future_to_promise, spawn_local};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
  // This provides better error messages in debug mode.
  // It's disabled in release mode so it doesn't bloat up the file size.
  #[cfg(debug_assertions)]
  console_error_panic_hook::set_once();

  Ok(())
}

#[wasm_bindgen]
extern "C" {
  fn alert(s: &str);
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen]
extern "C" {
  type Buffer;
}

#[wasm_bindgen(module = "fs")]
extern "C" {
  #[wasm_bindgen(js_name = readFileSync, catch)]
  pub fn read_file(path: &str, encoding: &str) -> Result<String, JsValue>;

  #[wasm_bindgen(js_name = writeFileSync, catch)]
  pub fn write_file(path: &str, content: &str) -> Result<(), JsValue>;
}

#[wasm_bindgen]
pub fn parse_case(s: String) -> Result<JsValue, JsValue> {
  console_log!("Parsing case.");
  let r = case::case(&s);
  if let Ok(c) = r {
    console_log!("Converting case to JsValue.");
    Ok(JsValue::from_serde(&c).unwrap())
  } else {
    Ok(JsValue::NULL)
  }
}
