module.exports = {
  content: ['./src/**/*.svelte'],
  theme: {
    gridTemplateAreas: {
      layout: [
        'bus        gen     branch',
        'graph      graph   graph',
        'graph      graph   graph',
        'graph      graph   graph',
        '.          .       .',
      ],
    },
    extend: {},
  },
  plugins: [require('@savvywombat/tailwindcss-grid-areas')],
}
