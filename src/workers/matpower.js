import init, { parse_case } from '../../wasm-matpower/pkg/wasm-matpower'
// Workaround because Vite is using window.location in Web Workers
import wasm from '../../wasm-matpower/pkg/wasm-matpower_bg.wasm?url'

init(wasm)

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
