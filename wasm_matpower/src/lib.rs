#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_must_use)]

mod case;

use std::{cell::RefCell, rc::Rc};

use js_sys::Promise;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::{future_to_promise, spawn_local};
use web_sys::{console, HtmlElement, HtmlInputElement, MessageEvent, Worker};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
  fn alert(s: &str);
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
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
pub fn startup() {
  // Here, we create our worker. In a larger app, multiple callbacks should be
  // able to interact with the code in the worker. Therefore, we wrap it in
  // `Rc<RefCell>` following the interior mutability pattern. Here, it would
  // not be needed but we include the wrapping anyway as example.
  let worker_handle = Rc::new(RefCell::new(Worker::new("./matpower.js").unwrap()));
  console::log_1(&"Created a new worker from within WASM".into());

  // Pass the worker to the function which sets up the `oninput` callback.
  setup_input_oninput_callback(worker_handle.clone());
}

fn setup_input_oninput_callback(worker: Rc<RefCell<web_sys::Worker>>) {
  let document = web_sys::window().unwrap().document().unwrap();

  // If our `onmessage` callback should stay valid after exiting from the
  // `oninput` closure scope, we need to either forget it (so it is not
  // destroyed) or store it somewhere. To avoid leaking memory every time we
  // want to receive a response from the worker, we move a handle into the
  // `oninput` closure to which we will always attach the last `onmessage`
  // callback. The initial value will not be used and we silence the warning.
  #[allow(unused_assignments)]
  let mut persistent_callback_handle = get_on_msg_callback();

  let callback = Closure::wrap(Box::new(move || {
    console::log_1(&"oninput callback triggered".into());
    let document = web_sys::window().unwrap().document().unwrap();

    let input_field = document.get_element_by_id("inputNumber").expect("#inputNumber should exist");
    let input_field = input_field.dyn_ref::<HtmlInputElement>().expect("#inputNumber should be a HtmlInputElement");

    // If the value in the field can be parsed to a `i32`, send it to the
    // worker. Otherwise clear the result field.
    match input_field.value().parse::<i32>() {
      Ok(number) => {
        // Access worker behind shared handle, following the interior
        // mutability pattern.
        let worker_handle = &*worker.borrow();
        let _ = worker_handle.post_message(&number.into());
        persistent_callback_handle = get_on_msg_callback();

        // Since the worker returns the message asynchronously, we
        // attach a callback to be triggered when the worker returns.
        worker_handle.set_onmessage(Some(persistent_callback_handle.as_ref().unchecked_ref()));
      },
      Err(_) => {
        document
          .get_element_by_id("resultField")
          .expect("#resultField should exist")
          .dyn_ref::<HtmlElement>()
          .expect("#resultField should be a HtmlInputElement")
          .set_inner_text("");
      },
    }
  }) as Box<dyn FnMut()>);

  // Attach the closure as `oninput` callback to the input field.
  document
    .get_element_by_id("inputNumber")
    .expect("#inputNumber should exist")
    .dyn_ref::<HtmlInputElement>()
    .expect("#inputNumber should be a HtmlInputElement")
    .set_oninput(Some(callback.as_ref().unchecked_ref()));

  // Leaks memory.
  callback.forget();
}

/// Create a closure to act on the message returned by the worker
fn get_on_msg_callback() -> Closure<dyn FnMut(MessageEvent)> {
  let callback = Closure::wrap(Box::new(move |event: MessageEvent| {
    console::log_2(&"Received response: ".into(), &event.data().into());

    let result = match event.data().as_bool().unwrap() {
      true => "even",
      false => "odd",
    };

    let document = web_sys::window().unwrap().document().unwrap();
    document
      .get_element_by_id("resultField")
      .expect("#resultField should exist")
      .dyn_ref::<HtmlElement>()
      .expect("#resultField should be a HtmlInputElement")
      .set_inner_text(result);
  }) as Box<dyn FnMut(_)>);

  callback
}

#[wasm_bindgen]
pub fn parse_case(s: String) -> Result<JsValue, JsValue> {
  let r = case::case(&s);
  if let Ok(c) = r {
    Ok(JsValue::from_serde(&c).unwrap())
  } else {
    Ok(JsValue::NULL)
  }
}
