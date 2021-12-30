import adapter from '@sveltejs/adapter-static'
import preprocess from 'svelte-preprocess'

import worker, { pluginHelper } from 'vite-plugin-worker'

const server = {}
const path = '/matpower-case-viewer'

const prod = process.env.NODE_ENV === 'production'

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: preprocess(),

  kit: {
    adapter: adapter({
      // default options are shown
      pages: 'dist',
      assets: 'dist',
      fallback: null,
    }),

    // hydrate the <div id="svelte"> element in src/app.html
    target: '#svelte',

    vite: {
      plugins: [pluginHelper(), worker.default({})],
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
