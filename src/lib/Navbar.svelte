<script lang="ts">
  import { base } from '$app/paths'
  import { getStores, navigating, page, session } from '$app/stores'
</script>

<nav
  class="container flex z-20 w-full h-24 pt-2 place-items-center justify-between space-x-4 max-w-6xl mx-auto"
>
  <div class="relative flex place-items-center h-5">
    <a sveltekit:prefetch href="{base}/" class="text-2xl font-black"> MATPOWER Case Viewer </a>
  </div>
  <div class="">
    <a
      sveltekit:prefetch
      href="{base}/"
      class="{$page === '/'
        ? 'border-b-2 border-yellow-600'
        : ''} font-bold px-3 py-2 text-gray-700 rounded-lg hover:bg-gray-100"
    >
      Home
    </a>
    <a
      sveltekit:prefetch
      href="{base}/about"
      class="{$page === '/about'
        ? 'border-b-2 border-yellow-600'
        : ''} font-bold py-2 text-gray-700 rounded-lg hover:bg-gray-100"
    >
      About
    </a>
  </div>
</nav>
