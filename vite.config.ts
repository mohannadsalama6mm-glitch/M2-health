import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [
    {
      name: "strip-browser-demo-fixtures",
      apply: "build",
      enforce: "pre",
      transform(_source, id) {
        const normalized = id.replaceAll("\\", "/");
        if (normalized.includes("/src/data/demo.ts"))
          return {
            code: `export const demoSales=[]; export const demoStock=[]; export const demoProducts=[]; export const demoExpiry=[]; export const demoChart={week:[],previous:[]}; export const demoInventory=[];`,
            map: null,
          };
        if (!normalized.includes("/src/mock/fixtures.ts")) return null;
        return {
          code: `export const demoDate=""; export const products=[]; export const suppliers=[]; export const customers=[]; export const purchases=[]; export const purchaseLines=[]; export const branches=[]; export const employees=[]; export const sales=[]; export const movementTypes=[]; export const movements=[]; export const countSessions=[]; export const notifications=[]; export const auditRows=[]; export const backupHistory=[]; export const syncQueue=[]; export const financeSections=[]; export const financeRows=[]; export const reportCategories=[]; export const reportRows=[]; export const settingsSections=[]; export const roles=[]; export const permissionGroups=[];`,
          map: null,
        };
      },
    },
    react(),
  ],
  clearScreen: false,

  server: {
    host: "127.0.0.1",
    port: 1420,
    strictPort: true,
    watch: {
      ignored: ["**/src-tauri/target/**"],
    },
  },

  build: {
    target: "es2022",
  },
});
