<script setup>
import { onMounted, ref, reactive, watchEffect, onBeforeUnmount } from 'vue'
import * as d3 from 'd3'

const props = defineProps({
  data: Object,
})

const useResizeObserver = () => {
  const resizeRef = ref()
  const resizeState = reactive({
    dimensions: {},
  })

  const observer = new ResizeObserver((entries) => {
    entries.forEach((entry) => {
      resizeState.dimensions = entry.contentRect
    })
  })

  onMounted(() => {
    // set initial dimensions right before observing: Element.getBoundingClientRect()
    resizeState.dimensions = resizeRef.value.getBoundingClientRect()
    observer.observe(resizeRef.value)
  })

  onBeforeUnmount(() => {
    observer.unobserve(resizeRef.value)
  })

  return { resizeState, resizeRef }
}

const svg = ref(null)

onMounted(() => {
  // set the dimensions and margins of the graph
  const margin = { top: 10, right: 30, bottom: 30, left: 40 }

  // append the svg object to the body of the page
  const svg = d3
    .select('#plot')
    .append('svg')
    .attr('class', 'container h-full max-w-6xl mx-auto')
    .append('g')

  watchEffect(() => {
    const width = 500
    const height = 500

    console.log(props.data)
    // Initialize the links
    const link = svg
      .selectAll('line')
      .data(props.data.links)
      .join('line')
      .style('stroke', '#aaa')

    // Initialize the nodes
    const node = svg
      .selectAll('circle')
      .data(props.data.nodes)
      .join('circle')
      .attr('r', 20)
      .style('fill', '#69b3a2')

    const label = node
      .append('text')
      .text(function (d) {
        return d.id
      })
      .attr('x', 6)
      .attr('y', 3)
      .style('font-family', 'sans-serif')
      .style('font-size', '10px')

    // Let's list the force we wanna apply on the network
    d3.forceSimulation(props.data.nodes) // Force algorithm is applied to data.nodes
      .force(
        'link',
        d3
          .forceLink() // This force provides links between nodes
          .id(function (d) {
            return d.id
          }) // This provide  the id of a node
          .links(props.data.links), // and this the list of links
      )
      .force('charge', d3.forceManyBody().strength(-400)) // This adds repulsion between nodes. Play with the -400 for the repulsion strength
      .force('center', d3.forceCenter(width / 2, height / 2)) // This force attracts nodes to the center of the svg area
      .on('end', ticked)

    // This function is run at each iteration of the force algorithm, updating the nodes position.
    function ticked() {
      link
        .attr('x1', function (d) {
          return d.source.x
        })
        .attr('y1', function (d) {
          return d.source.y
        })
        .attr('x2', function (d) {
          return d.target.x
        })
        .attr('y2', function (d) {
          return d.target.y
        })

      node
        .attr('cx', function (d) {
          return d.x + 6
        })
        .attr('cy', function (d) {
          return d.y - 6
        })

      label
        .attr('cx', function (d) {
          return d.x + 6
        })
        .attr('cy', function (d) {
          return d.y + 3
        })
    }
  })
})
</script>

<template>
  <div id="plot" class="h-full" />
</template>
