<script context="module">
</script>

<script lang="ts">
  import Graph from './Graph.svelte'

  import { onMount } from 'svelte'

  import createWorker from 'worker-iife:../workers/matpower'
  import { writable } from 'svelte/store'
  import { browser } from '$app/env'

  let worker: Worker
  onMount(() => {
    worker = createWorker()
  })

  let loading = false
  let loaded = false

  // persist case data
  const case_obj = writable({})

  if (browser) {
    $case_obj = localStorage.getItem('matpowerCase') || {}
    case_obj.subscribe((val) => localStorage.setItem('matpowerCase', JSON.stringify(val)))
  }

  function resetCase(e) {
    localStorage.removeItem('matpowerCase')
  }

  $: graph = case_graph($case_obj)

  function case_graph(obj) {
    if (obj.branch === undefined) {
      return {
        nodes: [],
        links: [],
      }
    } else {
      const node_names = obj.bus_name
      const nodes = obj.bus.map((element, i) => {
        return {
          id: element.idx,
          name: node_names[i] ? node_names[i] : element.idx,
        }
      })
      const links = obj.branch.map((element) => {
        return { source: element.f_bus, target: element.t_bus }
      })
      return {
        nodes: nodes,
        links: links,
      }
    }
  }

  function uploadFile(e) {
    loaded = false
    loading = true
    const file = e.target.files[0]
    if (file) {
      var reader = new FileReader()
      reader.readAsText(file, 'UTF-8')
      reader.onload = function (evt) {
        worker.postMessage({
          data: evt.target.result,
        })
      }

      worker.addEventListener('message', (event) => {
        $case_obj = event.data.data
        loaded = true
        loading = false
      })

      reader.onerror = function (_) {
        $case_obj = null
        loading = false
        loaded = false
      }.bind(this)
    }
  }
</script>

<div class="flex flex-col h-full">
  <div class="grid grid-cols-2 justify-items-stretch">
    <input class="justify-self-start" on:change={uploadFile} type="file" />
    <button
      class="justify-self-end bg-blue-500 hover:bg-blue-700 text-white font-bold rounded px-4"
      on:click={resetCase}
    >
      Reset
    </button>
  </div>
  {#if loading}
    <div>Loading...</div>
  {:else if loaded}
    <div class="grow grid grid-areas-layout justify-items-stretch my-auto">
      <div
        class="grid-in-bus grid place-content-center text-center text-green-400 font-mono hover:underline hover:decoration-green-500"
        v-if="loaded"
      >
        Number of buses: {$case_obj.bus.length}
      </div>
      <div
        class="grid-in-gen grid place-content-center text-center text-green-400 font-mono hover:underline hover:decoration-green-500"
        v-if="loaded"
      >
        Number of generators: {$case_obj.gen.length}
      </div>
      <div
        class="grid-in-branch grid place-content-center text-center text-green-400 font-mono hover:underline hover:decoration-green-500"
        v-if="loaded"
      >
        Number of branches: {$case_obj.branch.length}
      </div>
      <Graph class="grid-in-graph justify-content align-content" {graph} />
    </div>
  {/if}
</div>
