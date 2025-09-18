/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./app/src/**/*.{rs,html,css}",
    "./public/**/*.html",
  ],
  theme: {
    extend: {
      // Bitsacco Design System Colors
      colors: {
        // Primary teal colors (Bitsacco brand)
        primary: {
          50: '#f0fdfa',
          100: '#ccfbf1',
          200: '#99f6e4',
          300: '#5eead4',
          400: '#2dd4bf',
          500: '#14b8a6',
          600: '#0d9488',
          700: '#0f766e',
          800: '#115e59',
          900: '#134e4a',
        },
        teal: {
          50: '#f0fdfa',
          100: '#ccfbf1',
          200: '#99f6e4',
          300: '#5eead4',
          400: '#2dd4bf',
          500: '#14b8a6',
          600: '#0d9488',
          700: '#0f766e',
          800: '#115e59',
          900: '#134e4a',
        },
        // Neutral colors
        neutral: {
          0: '#ffffff',
          50: '#fafafa',
          100: '#f5f5f5',
          200: '#e5e5e5',
          300: '#d4d4d4',
          400: '#a3a3a3',
          500: '#737373',
          600: '#525252',
          700: '#404040',
          800: '#262626',
          900: '#171717',
          950: '#0a0a0a',
        },
        // Slate colors for dark theme
        slate: {
          50: '#f8fafc',
          100: '#f1f5f9',
          200: '#e2e8f0',
          300: '#cbd5e1',
          400: '#94a3b8',
          500: '#64748b',
          600: '#475569',
          700: '#334155',
          800: '#1e293b',
          900: '#0f172a',
          950: '#020617',
        },
      },
      // Bitsacco Typography
      fontFamily: {
        // Bitsacco font families
        heading: ['Satoshi', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        body: ['Nunito', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        title: ['Poppins', 'system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'sans-serif'],
        sans: ['system-ui', '-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'Roboto', 'Oxygen', 'Ubuntu', 'Cantarell', 'sans-serif'],
      },
      // Enhanced spacing
      spacing: {
        "18": "4.5rem", // 72px
        "88": "22rem",  // 352px
        "100": "25rem", // 400px
        "104": "26rem", // 416px
        "112": "28rem", // 448px
        "128": "32rem", // 512px
      },
      // Enhanced border radius
      borderRadius: {
        "4xl": "2rem",
        "5xl": "2.5rem",
        "6xl": "3rem",
      },
      // Enhanced shadows
      boxShadow: {
        xs: "0 1px 2px 0 rgb(0 0 0 / 0.05)",
        sm: "0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)",
        md: "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)",
        lg: "0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)",
        xl: "0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)",
        "2xl": "0 25px 50px -12px rgb(0 0 0 / 0.25)",
        inner: "inset 0 2px 4px 0 rgb(0 0 0 / 0.05)",
      },
      // Bitsacco animations
      keyframes: {
        "fade-in": {
          "0%": { opacity: "0" },
          "100%": { opacity: "1" },
        },
        "fade-out": {
          "0%": { opacity: "1" },
          "100%": { opacity: "0" },
        },
        "slide-in-right": {
          "0%": { transform: "translateX(100%)" },
          "100%": { transform: "translateX(0)" },
        },
        "slide-in-left": {
          "0%": { transform: "translateX(-100%)" },
          "100%": { transform: "translateX(0)" },
        },
        "slide-in-up": {
          "0%": { transform: "translateY(100%)" },
          "100%": { transform: "translateY(0)" },
        },
        "slide-in-down": {
          "0%": { transform: "translateY(-100%)" },
          "100%": { transform: "translateY(0)" },
        },
      },
      animation: {
        "fade-in": "fade-in 0.2s ease-out",
        "fade-out": "fade-out 0.15s ease-in",
        "slide-in-right": "slide-in-right 0.3s ease-out",
        "slide-in-left": "slide-in-left 0.3s ease-out",
        "slide-in-up": "slide-in-up 0.3s ease-out",
        "slide-in-down": "slide-in-down 0.3s ease-out",
      },
    },
  },
  plugins: [
    require('@tailwindcss/forms'),
    require('@tailwindcss/typography'),
    // Bitsacco utility plugins
    function ({ addUtilities, addComponents, addBase, theme }) {
      // CSS Custom Properties for theme consistency
      addBase({
        ":root": {
          "--font-heading": "Satoshi, system-ui, -apple-system, sans-serif",
          "--font-body": "Nunito, system-ui, -apple-system, sans-serif",
          "--font-title": "Poppins, system-ui, -apple-system, sans-serif",
        },
      });

      // Scrollbar utilities
      addUtilities({
        ".scrollbar-none": {
          "scrollbar-width": "none",
          "-ms-overflow-style": "none",
          "&::-webkit-scrollbar": {
            display: "none",
          },
        },
        ".scrollbar-thin": {
          "scrollbar-width": "thin",
          "&::-webkit-scrollbar": {
            width: "6px",
            height: "6px",
          },
          "&::-webkit-scrollbar-track": {
            backgroundColor: theme('colors.neutral.100'),
          },
          "&::-webkit-scrollbar-thumb": {
            backgroundColor: theme('colors.neutral.400'),
            borderRadius: theme('borderRadius.full'),
          },
          "&::-webkit-scrollbar-thumb:hover": {
            backgroundColor: theme('colors.neutral.500'),
          },
        },
      });

      // Focus ring utilities
      addUtilities({
        ".focus-ring": {
          "&:focus": {
            outline: "none",
            boxShadow: `0 0 0 3px ${theme('colors.primary.400')}40`,
          },
        },
        ".focus-ring-error": {
          "&:focus": {
            outline: "none",
            boxShadow: `0 0 0 3px ${theme('colors.red.400')}40`,
          },
        },
      });

      // Button components
      addComponents({
        ".btn-primary": {
          backgroundColor: theme('colors.primary.500'),
          color: theme('colors.neutral.0'),
          "&:hover:not(:disabled)": {
            backgroundColor: theme('colors.primary.600'),
          },
          "&:active:not(:disabled)": {
            backgroundColor: theme('colors.primary.700'),
          },
        },
        ".btn-secondary": {
          backgroundColor: theme('colors.neutral.100'),
          color: theme('colors.neutral.900'),
          "&:hover:not(:disabled)": {
            backgroundColor: theme('colors.neutral.200'),
          },
          "&:active:not(:disabled)": {
            backgroundColor: theme('colors.neutral.300'),
          },
        },
      });
    }
  ],
}
