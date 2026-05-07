import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import './styles/design-system.css'
import './styles/components.css'
import './style.css'

// Prevent Windows trackpad swipe from triggering browser back/forward navigation
history.pushState(null, '', location.href)
window.addEventListener('popstate', () => {
  history.pushState(null, '', location.href)
})

const app = createApp(App)
app.use(createPinia())
app.mount('#app')
