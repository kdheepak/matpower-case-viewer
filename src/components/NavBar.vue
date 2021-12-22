<template>
  <nav
    class="container flex z-20 w-full h-24 pt-2 place-items-center justify-between space-x-4 max-w-6xl mx-auto"
  >
    <div class="relative flex place-items-center h-5">
      <router-link to="/" class="text-2xl font-black">
        MATPOWER Case Viewer
      </router-link>
    </div>
    <div class="justify-center place-items-center">
      <router-link
        to="/"
        class="{$page.path === '/' ? 'border-b-2 border-yellow-600' : ''} font-bold px-3 py-2 text-gray-700 rounded-lg hover:bg-gray-100"
      >
        Home
      </router-link>
      <router-link
        to="/about"
        class="{$page.path === '/about' ? 'border-b-2 border-yellow-600' : ''} font-bold px-3 py-2 text-gray-700 rounded-lg hover:bg-gray-100"
      >
        About
      </router-link>
    </div>
  </nav>
</template>
