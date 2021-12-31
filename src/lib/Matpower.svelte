<script context="module">
</script>

<script lang="ts">
  import Graph from './Graph.svelte'

  import { onMount } from 'svelte'

  import createWorker from 'worker-iife:../workers/matpower'
  import { writable } from 'svelte/store'
  import { browser } from '$app/env'

  import { Case } from '../../wasm_matpower/pkg'

  let worker: Worker
  onMount(() => {
    worker = createWorker()
  })

  let loading = false
  let loaded = false

  // persist case data
  function createDefaultCase(): Case {
    return {
      bus: [],
      branch: [],
      gen: [],
      gencost: [],
      name: '',
      base_mva: 0,
      dcline: [],
      bus_name: [],
      version: '',
    }
  }
  const case_obj = writable(createDefaultCase())

  function resetCase(_: Event) {
    $case_obj = createDefaultCase()
    loaded = false
    loading = false
  }

  $: graph = case_graph($case_obj)

  function case_graph(obj: Case) {
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

  function uploadFile(e: Event) {
    console.log(e)
    loading = true
    loaded = false
    const file = (e.target as HTMLInputElement).files[0]
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
        loading = false
        loaded = true
      })

      reader.onerror = () => {
        $case_obj = createDefaultCase()
        loading = false
        loaded = false
      }
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
      >
        Number of buses: {$case_obj.bus.length}
      </div>
      <div
        class="grid-in-gen grid place-content-center text-center text-green-400 font-mono hover:underline hover:decoration-green-500"
      >
        Number of generators: {$case_obj.gen.length}
      </div>
      <div
        class="grid-in-branch grid place-content-center text-center text-green-400 font-mono hover:underline hover:decoration-green-500"
      >
        Number of branches: {$case_obj.branch.length}
      </div>
      <Graph class="grid-in-graph justify-content align-content" {graph} />
    </div>
  {/if}
</div>
