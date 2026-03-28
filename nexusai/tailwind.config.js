/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        nexus: {
          bg: "#05060a",
          surface: "#0d0f17",
          surface2: "#141722",
          surface3: "#1a1e2e",
          accent: "#6ee7b7",
          accent2: "#818cf8",
          accent3: "#f472b6",
          warn: "#fbbf24",
          text: "#e2e8f0",
          dim: "#64748b",
          border: "#1e293b",
        },
      },
      fontFamily: {
        sans: ["Sora", "system-ui", "sans-serif"],
        mono: ["DM Mono", "Fira Code", "monospace"],
      },
    },
  },
  plugins: [],
};
