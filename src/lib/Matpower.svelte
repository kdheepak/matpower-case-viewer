<script context="module">
</script>

<script lang="ts">
  import Graph from './Graph.svelte'

  import { onMount } from 'svelte'

  import createWorker from 'worker-iife:../workers/matpower'
  let worker: Worker
  onMount(() => {
    worker = createWorker()
  })

  let case_obj = []
  let loading = false
  let loaded = false

  $: graph = case_graph(case_obj)

  function case_graph(case_obj) {
    if (case_obj.branch === undefined) {
      return {
        nodes: [],
        links: [],
      }
    } else {
      const node_names = case_obj.bus_name
      const nodes = case_obj.bus.map((element, i) => {
        return {
          id: element.idx,
          name: node_names[i] ? node_names[i] : element.idx,
        }
      })
      const links = case_obj.branch.map((element) => {
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
  <Graph {graph} />
{/if}
