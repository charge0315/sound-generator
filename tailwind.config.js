/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        'pulse-neon': '#00f2ff',
        'pulse-deep': '#0a0b10',
      },
    },
  },
  plugins: [],
}
