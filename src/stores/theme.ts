import { defineStore } from 'pinia'

export type ThemeMode = 'light' | 'dark' | 'system'
export type ColorScheme = 'midnight-blue' | 'slate-gray'

interface ThemeState {
  mode: ThemeMode
  colorScheme: ColorScheme
}

export const useThemeStore = defineStore('theme', {
  state: (): ThemeState => ({
    mode: (localStorage.getItem('theme-mode') as ThemeMode) || 'system',
    colorScheme: (localStorage.getItem('theme-color-scheme') as ColorScheme) || 'midnight-blue',
  }),

  getters: {
    isDark: (state) => {
      if (state.mode === 'system') {
        return window.matchMedia('(prefers-color-scheme: dark)').matches
      }
      return state.mode === 'dark'
    },
    actualMode: (state) => {
      if (state.mode === 'system') {
        return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
      }
      return state.mode
    },
  },

  actions: {
    setMode(mode: ThemeMode) {
      this.mode = mode
      localStorage.setItem('theme-mode', mode)
      this.applyTheme()
    },

    toggleMode() {
      // Cycle: light -> dark -> system -> light
      const modes: ThemeMode[] = ['light', 'dark', 'system']
      const currentIndex = modes.indexOf(this.mode)
      this.setMode(modes[(currentIndex + 1) % modes.length])
    },

    setColorScheme(scheme: ColorScheme) {
      this.colorScheme = scheme
      localStorage.setItem('theme-color-scheme', scheme)
      this.applyTheme()
    },

    applyTheme() {
      const actualMode = this.actualMode
      document.documentElement.setAttribute('data-theme', actualMode)
      document.documentElement.setAttribute('data-color-scheme', this.colorScheme)
    },

    initTheme() {
      // Apply initial theme
      this.applyTheme()

      // Listen for system theme changes
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      const handleChange = () => {
        if (this.mode === 'system') {
          this.applyTheme()
        }
      }

      mediaQuery.addEventListener('change', handleChange)
    },
  },
})