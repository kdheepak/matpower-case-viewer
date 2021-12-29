<script>
import * as d3 from 'd3'

function create_worker() {
  return new Worker(new URL('../workers/matpower.js', import.meta.url), {
    type: 'module',
  })
}

const worker = create_worker()

export default {
  name: 'Home',
  data() {
    return {
      case_obj: {},
      loading: false,
      loaded: false,
    }
  },
  mounted() {
    this.generatePlot()
  },
  methods: {
    generatePlot() {
      const w = 500
      const h = 500
      const svg = d3
        .select('#plot')
        .append('svg')
        .attr('width', w)
        .attr('height', h)
    },
    uploadFile(e) {
      this.loaded = false
      this.loading = true
      const file = e.target.files[0]
      const that = this
      if (file) {
        var reader = new FileReader()
        reader.readAsText(file, 'UTF-8')
        reader.onload = function (evt) {
          worker.postMessage({
            data: evt.target.result,
          })
        }

        worker.addEventListener('message', (event) => {
          that.case_obj = event.data.data
          that.loaded = true
          that.loading = false
        })

        reader.onerror = function (_) {
          this.case_obj = null
          this.loading = false
          this.loaded = false
        }.bind(this)
      }
    },
  },
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
  <div id="plot" />
</template>
