<script setup>
import { onMounted, ref, watchEffect, computed } from 'vue'
import * as d3 from 'd3'
import Graph from './Graph.vue'

function create_worker() {
  return new Worker(new URL('../workers/matpower.js', import.meta.url), {
    type: 'module',
  })
}

const worker = create_worker()

const case_obj = ref({})
const loading = ref(false)
const loaded = ref(false)
const case_graph = computed(() => {
  if (case_obj.value.branch === undefined) {
    return {
      nodes: [],
      links: [],
    }
  } else {
    const node_names = case_obj.value.bus_name
    const nodes = case_obj.value.bus.map((element, i) => {
      return {
        id: element.idx,
        name: node_names[i] ? node_names[i] : element.idx,
      }
    })
    const links = case_obj.value.branch.map((element) => {
      return { source: element.f_bus, target: element.t_bus }
    })
    return {
      nodes: nodes,
      links: links,
    }
  }
})

function uploadFile(e) {
  loaded.value = false
  loading.value = true
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
      case_obj.value = event.data.data
      console.log(case_obj.value)
      loaded.value = true
      loading.value = false
    })

    reader.onerror = function (_) {
      case_obj.value = null
      loading.value = false
      loaded.value = false
    }.bind(this)
  }
}
</script>

<template>
  <div>
    <input @change="uploadFile" type="file" ref="file" />
  </div>
  <div v-if="loading">Loading...</div>
  <div v-if="loaded">Number of buses: {{ case_obj.bus.length }}</div>
  <div v-if="loaded">Number of generators: {{ case_obj.gen.length }}</div>
  <div v-if="loaded">Number of branches: {{ case_obj.branch.length }}</div>
  <Graph :data="case_graph" />
</template>
