import adapter from '@sveltejs/adapter-static'
import preprocess from 'svelte-preprocess'

import worker, { pluginHelper } from 'vite-plugin-worker'
import wasmPack from 'vite-plugin-wasm-pack'

const server = {}
const prod = process.env.NODE_ENV === 'production'

let paths = {}
if (prod) {
  paths = { base: '/matpower-case-viewer' }
}

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: preprocess(),

  kit: {
    paths,
    adapter: adapter({
      // default options are shown
      pages: 'dist',
      assets: 'dist',
      fallback: null,
    }),

    // hydrate the <div id="svelte"> element in src/app.html
    target: '#svelte',

    vite: {
      plugins: [pluginHelper(), worker.default({}), wasmPack(['./wasm_matpower'], [])],
      ssr: {
        noExternal: [/^@smui(?:-extra)?\//],
      },
      server: {
        fs: {
          allow: ['wasm_matpower', '.rsw'],
        },
        ...server,
      },
    },
  },
}

export default config
