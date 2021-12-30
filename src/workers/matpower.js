importScripts('../../wasm_matpower/pkg/wasm_matpower.js')

console.log('Initializing worker')

const { parse_case } = wasm_bindgen

async function init_wasm_matpower() {
  await wasm_bindgen('../../wasm_matpower/pkg/wasm_matpower_bg.wasm')

  // Handle incoming messages
  self.addEventListener(
    'message',
    function (event) {
      const { data } = event.data
      self.postMessage({
        data: parse_case(data),
      })
    },
    false,
  )
}

init_wasm_matpower()
