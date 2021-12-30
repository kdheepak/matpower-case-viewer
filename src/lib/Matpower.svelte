<script lang="ts">
  import { onMount } from 'svelte'
  import MatpowerWorker from '../workers/matpower?worker'
  import wasm from '../../wasm-matpower/pkg/wasm-matpower_bg.wasm'

  let worker: Worker

  onMount(async () => {
    worker = new MatpowerWorker()
  })

  let case_obj = {
    bus: [],
    gen: [],
    branch: [],
  }
  let loading = false
  let loaded = false

  function uploadFile(e) {
    loaded = false
    loading = true
    const file = e.target.files[0]
    if (file) {
      var reader = new FileReader()
      reader.readAsText(file, 'UTF-8')
      reader.onload = function (evt) {
        console.log(evt.target.result)
        worker.postMessage({
          data: evt.target.result,
        })
      }

      worker.addEventListener('message', (event) => {
        case_obj = event.data.data
        loaded = true
        loading = false
      })

      reader.onerror = function (_) {
        case_obj = null
        loading = false
        loaded = false
      }.bind(this)
    }
  }
</script>

<div>
  <input on:change={uploadFile} type="file" />
</div>
{#if loading}
  <div>Loading...</div>
{:else if loaded}
  <div v-if="loaded">Number of buses: {case_obj.bus.length}</div>
  <div v-if="loaded">Number of generators: {case_obj.gen.length}</div>
  <div v-if="loaded">Number of branches: {case_obj.branch.length}</div>
{/if}
