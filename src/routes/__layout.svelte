<script lang="ts">
  import '../app.css'
  import NavBar from '$lib/NavBar.svelte'
  import Footer from '$lib/Footer.svelte'
</script>

<div class="h-screen grid grid-rows-[auto_1fr_auto]">
  <NavBar />

  <main class="container max-w-6xl mx-auto">
    <slot />
  </main>

  <Footer />
</div>
