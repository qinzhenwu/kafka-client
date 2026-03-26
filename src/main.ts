import { createApp } from 'vue'
import naive from 'naive-ui'
import App from './App.vue'
import router from './router'
import { pinia } from './stores'
import i18n from './i18n'
import './styles/theme.css'
import './styles/form.css'

const app = createApp(App)

app.use(naive)
app.use(router)
app.use(pinia)
app.use(i18n)

app.mount('#app')