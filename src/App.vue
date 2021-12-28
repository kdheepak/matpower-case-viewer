<script lang="ts">
// This starter template is using Vue 3 <script setup> SFCs
// Check out https://v3.vuejs.org/api/sfc-script-setup.html#sfc-script-setup
import NavBar from './components/NavBar.vue'
import Footer from './components/Footer.vue'
export default {
  components: {
    NavBar,
    Footer,
  },
}
</script>

<template>
  <div class="h-screen grid grid-rows-[auto_1fr_auto]">
    <NavBar />
    <div class="container max-w-6xl mx-auto">
      <router-view />
    </div>
    <Footer />
  </div>
</template>
