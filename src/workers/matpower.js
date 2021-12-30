import init, { parse_case } from '../../wasm_matpower/pkg/'

async function init_wasm_matpower() {
  await init()

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
