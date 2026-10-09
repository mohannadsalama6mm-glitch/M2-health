import type { Page } from "@playwright/test";

export function installAuthorizationDouble(page: Page, roleCode: "ADMIN" | "CASHIER" | "VIEWER" = "CASHIER") {
  return page.addInitScript(({ roleCode }) => {
    const branches = [
      { id: "branch-a", code: "A", name: "Branch A", phone: "", address: "", isActive: true, createdAt: "", updatedAt: "" },
      { id: "branch-b", code: "B", name: "Branch B", phone: "", address: "", isActive: true, createdAt: "", updatedAt: "" },
    ];
    const role = { id: `role-${roleCode}`, code: roleCode, name: roleCode, description: "", isSystem: true };
    const user = {
      id: "auth-user", username: "staff.local", displayName: "Branch Staff", isActive: true,
      revision: 1, createdAt: "", updatedAt: "", roles: [role],
      branches: branches.map((branch, index) => ({ branchId: branch.id, branchCode: branch.code, branchName: branch.name, isPrimary: index === 0 })),
    };
    let token = "";
    let invalidated = false;
    const calls: Array<{ command: string; args: Record<string, unknown> }> = [];
    Object.assign(window, {
      __phase7bCalls: calls,
      __invalidatePhase7bSession: () => { invalidated = true; },
      isTauri: true,
      __TAURI_INTERNALS__: {
        invoke: async (command: string, args: Record<string, unknown> = {}) => {
          calls.push({ command, args });
          if (command === "recovery_status") return { authEnabled: true, laterWorkflowsEnabled: true };
          if (command === "get_auth_status") return { initialized: true, sessionLifetimeSeconds: 28800 };
          if (command === "login") { token = "c".repeat(64); invalidated = false; return { token, expiresAt: 9999999999, user }; }
          if (command === "logout") { token = ""; return null; }
          if (invalidated || args.sessionToken !== token || !token) throw { code: "unauthorized", message: "Authentication is required. Please sign in again." };
          if (command === "list_branches") return branches;
          if (command === "list_users") {
            if (roleCode !== "ADMIN") throw { code: "forbidden", message: "You do not have permission to perform this operation." };
            return { items: [user], total: 1, limit: 20, offset: 0 };
          }
          if (command === "list_roles") return [role];
          if (command === "export_phase6_csv") {
            if (roleCode !== "ADMIN") throw { code: "forbidden", message: "You do not have permission to perform this operation." };
            return { fileName: "export.csv", mimeType: "text/csv", content: "", rowCount: 0 };
          }
          if (command === "list_products") return { items: [], total: 0, limit: 20, offset: 0 };
          if (["list_manufacturers", "list_categories", "list_routes", "list_active_ingredients"].includes(command)) return [];
          throw new Error(`Unexpected native command: ${command}`);
        },
      },
    });
  }, { roleCode });
}
