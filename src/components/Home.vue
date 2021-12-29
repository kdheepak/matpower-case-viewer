<script setup>
import * as d3 from 'd3'
import { ref, reactive } from 'vue'

function create_worker() {
  console.log('worker')
  return new Worker(new URL('../workers/matpower.js', import.meta.url), {
    type: 'module',
  })
}

const worker = create_worker()

const data = {
  case_obj: {},
  loading: false,
  loaded: false,
}
const state = reactive(data)

function uploadFile(e) {
  state.loaded = false
  state.loading = true
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
      state.case_obj = event.data.data
      state.loaded = true
      state.loading = false
    })

    reader.onerror = function (_) {
      state.case_obj = null
      state.loading = false
      state.loaded = false
    }.bind(this)
  }
}
</script>

<template>
  <div>
    <input @change="uploadFile" type="file" ref="file" />
  </div>
  <div v-if="state.loading">Loading...</div>
  <div v-if="state.loaded">
    Number of buses: {{ state.case_obj.bus.length }}
  </div>
  <div v-if="state.loaded">
    Number of generators: {{ state.case_obj.gen.length }}
  </div>
  <div v-if="state.loaded">
    Number of branches: {{ state.case_obj.branch.length }}
  </div>
  <div id="plot" />
</template>
