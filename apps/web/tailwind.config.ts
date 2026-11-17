import type { Config } from "tailwindcss";
import { colors, spacing, radii, elevations, typography } from "./src/lib/tokens/index";

const config: Config = {
  content: ["./src/**/*.{html,js,svelte,ts}"],
  darkMode: "class",
  theme: {
    extend: {
      colors: {
        brand: colors.brand,
        accent: colors.accent,
        surface: colors.surface,
        status: {
          success: colors.status.success,
          "success-light": colors.status.successLight,
          "success-dark": colors.status.successDark,
          warning: colors.status.warning,
          "warning-light": colors.status.warningLight,
          "warning-dark": colors.status.warningDark,
          error: colors.status.error,
          "error-light": colors.status.errorLight,
          "error-dark": colors.status.errorDark,
          info: colors.status.info,
          "info-light": colors.status.infoLight,
          "info-dark": colors.status.infoDark,
        },
        budget: {
          safe: colors.tokenBudget.safe,
          warning: colors.tokenBudget.warning,
          critical: colors.tokenBudget.critical,
        },
      },
      fontFamily: {
        sans: [typography.fontFamily.sans],
        mono: [typography.fontFamily.mono],
      },
      spacing: spacing,
      borderRadius: radii,
      boxShadow: {
        "elevation-1": elevations[1],
        "elevation-2": elevations[2],
        "elevation-3": elevations[3],
        "elevation-4": elevations[4],
      },
    },
  },
  plugins: [],
};

export default config;
