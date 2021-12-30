import adapter from '@sveltejs/adapter-static'
import preprocess from 'svelte-preprocess'
import { ViteRsw } from 'vite-plugin-rsw'

const server = {}
const path = '/matpower-case-viewer'

const prod = process.env.NODE_ENV === 'production'

/** @type {import('@sveltejs/kit').Config} */
const config = {
  // Consult https://github.com/sveltejs/svelte-preprocess
  // for more information about preprocessors
  preprocess: preprocess(),

  kit: {
    // // fix gh-pages assets not loaded
    // ...(prod && {
    //   paths: {
    //     assets: `https://kdheepak.com{path}`,
    //     base: path,
    //   },
    // }),

    adapter: adapter({
      // default options are shown
      pages: 'dist',
      assets: 'dist',
      fallback: null,
    }),

    // hydrate the <div id="svelte"> element in src/app.html
    target: '#svelte',

    vite: {
      plugins: [
        ViteRsw({
          profile: 'release',
          crates: ['wasm_matpower'],
          unwatch: ['src/lib/*', 'src/routes/*'],
        }),
      ],
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
