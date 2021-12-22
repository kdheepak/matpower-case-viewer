import { createApp } from 'vue'
import App from './App.vue'
import Home from './components/Home.vue'
import About from './components/About.vue'
import './index.css'
import { createRouter, createWebHashHistory } from 'vue-router'

const routes = [
  { path: '/', component: Home },
  { path: '/about', component: About },
]

const router = createRouter({
  history: createWebHashHistory(),
  routes,
  linkActiveClass: 'active', // active class for non-exact links.
  linkExactActiveClass: 'active', // active class for *exact* links.
})

import store from './store'

const app = createApp(App)
app.use(router)
app.use(store)
app.mount('#app')
