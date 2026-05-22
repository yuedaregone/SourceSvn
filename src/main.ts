import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { listen } from '@tauri-apps/api/event'
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

listen<{ type: string; message: string }>('hook-toast', (event) => {
  import('./stores/toastStore').then(({ useToastStore }) => {
    const toast = useToastStore()
    const type = event.payload.type as 'success' | 'error' | 'warning' | 'info'
    if (['success', 'error', 'warning', 'info'].includes(type)) {
      toast.addToast(event.payload.message, type)
    } else {
      toast.info(event.payload.message)
    }
  })
})

app.mount('#app')
