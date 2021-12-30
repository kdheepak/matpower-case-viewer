<script lang="ts">
  import NavBar from './NavBar.svelte'
</script>

<main>
  <div class="h-screen grid grid-rows-[auto_1fr_auto]">
    <NavBar />
    <div class="container max-w-6xl mx-auto">
      <slot />
    </div>
  </div>
</main>
