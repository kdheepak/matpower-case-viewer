import init, { parse_case } from '/wasm-matpower/pkg'
init()

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
