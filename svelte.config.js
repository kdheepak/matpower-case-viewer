import adapter from '@sveltejs/adapter-auto'
import preprocess from 'svelte-preprocess'
import { ViteRsw } from 'vite-plugin-rsw'

const server = {}

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: preprocess(),

  kit: {
    adapter: adapter(),

    // hydrate the <div id="svelte"> element in src/app.html
    target: '#svelte',
    vite: {
      plugins: [
        ViteRsw({
          profile: 'release',
          crates: ['wasm-matpower'],
          unwatch: ['src/lib/*', 'src/routes/*'],
        }),
      ],
      ssr: {
        noExternal: [/^@smui(?:-extra)?\//],
      },
      server: {
        fs: {
          allow: ['wasm-matpower', '.rsw'],
        },
        ...server,
      },
    },
  },
}

export default config
