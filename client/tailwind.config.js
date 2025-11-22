/** @type {import('tailwindcss').Config} */
export default {
  content: ['./src/**/*.{html,js,svelte,ts}'],
  theme: {
    extend: {
      colors: {
        'primary-bg': 'var(--bg-color-primary)',
        'secondary-bg': 'var(--bg-color-secondary)',
        'tertiary-bg': 'var(--bg-color-tertiary)',
        'primary-text': 'var(--text-color-primary)',
        'secondary-text': 'var(--text-color-secondary)',
        'border-color': 'var(--border-color)',
        'accent-color': 'var(--accent-color)',
      },
    },
  },
  plugins: [],
}