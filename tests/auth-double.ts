import type { Page } from "@playwright/test";

export function installAuthDouble(page: Page, initialized = false, initialLicense = "TRIAL") {
  return page.addInitScript(({ initialized, initialLicense }) => {
    type Role = { id: string; code: string; name: string; description: string; isSystem: boolean };
    type User = { id: string; username: string; displayName: string; isActive: boolean; revision: number; createdAt: string; updatedAt: string; roles: Role[]; branches: unknown[] };
    interface NativeInput { username: string; displayName: string; password: string; roleCodes: string[]; branchIds: string[]; primaryBranchId: string | null; userId: string; roleCode: string; id: string; active: boolean }
    interface NativeArgs { input?: Partial<NativeInput>; query?: { search?: string; isActive?: boolean; roleCode?: string; limit?: number; offset?: number }; sessionToken?: string; id?: string; activationCode?: string }
    const roles: Role[] = [
      ["ADMIN", "Administrator", "Manages local users and identity settings."],
      ["PHARMACIST", "Pharmacist", "Pharmacy operations role."],
      ["CASHIER", "Cashier", "Point-of-sale role."],
      ["INVENTORY_MANAGER", "Inventory Manager", "Inventory operations role."],
      ["VIEWER", "Viewer", "Read-only role foundation."],
    ].map(([code, name, description]) => ({ id: `role-${code}`, code, name, description, isSystem: true }));
    const branch = { id: "branch-main", code: "MAIN", name: "Main Pharmacy", phone: "", address: "", isActive: true, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString() };
    const license = { state: initialLicense, daysRemaining: initialLicense === "TRIAL" ? 7 : null, canOperate: ["TRIAL", "ACTIVE", "OFFLINE_GRACE"].includes(initialLicense), readOnly: ["EXPIRED", "SUSPENDED", "REVOKED", "INVALID"].includes(initialLicense) };
    const licenseStatus = () => ({ ...license, deviceId: "device-early-access", activatedAt: license.state === "UNACTIVATED" ? null : 1_800_000_000, expiresAt: license.state === "TRIAL" ? 1_800_604_800 : null, lastValidatedAt: license.state === "ACTIVE" ? 1_800_000_000 : null, nextRevalidationAt: null, offlineGraceEndsAt: null, requiresActivation: license.state === "UNACTIVATED", reason: license.state === "UNACTIVATED" ? "Start the seven-day trial or enter a signed manual activation code." : "Test license state." });
    const state: { initialized: boolean; users: User[]; passwords: Record<string, string>; token: string } = { initialized, users: [], passwords: {}, token: "" };
    if (initialized) {
      state.users.push({ id: "admin-id", username: "admin.local", displayName: "Local Administrator", isActive: true, revision: 1, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(), roles: [roles[0]], branches: [] });
      state.passwords["admin.local"] = "VeryStrong!Pass2026";
    }
    const requireSession = (token: unknown) => { if (token !== state.token || !state.token) throw { code: "validation", message: "Session is invalid or expired." }; };
    Object.assign(window, { isTauri: true, __TAURI_INTERNALS__: { invoke: async (command: string, args: NativeArgs = {}) => {
      if (command === "recovery_status") return { authEnabled: true, laterWorkflowsEnabled: true };
      if (command === "get_license_status") return licenseStatus();
      if (command === "start_license_trial") { if (license.state !== "UNACTIVATED") throw { code: "conflict", message: "The trial can be started only once on this device." }; Object.assign(license, { state: "TRIAL", daysRemaining: 7, canOperate: true, readOnly: false }); return licenseStatus(); }
      if (command === "activate_manual_license" || command === "revalidate_manual_license") { if (args.activationCode !== "valid-manual-license") throw { code: "validation", message: "The activation code signature could not be verified." }; Object.assign(license, { state: "ACTIVE", daysRemaining: null, canOperate: true, readOnly: false }); return licenseStatus(); }
      if (command === "get_release_info") return { version: "0.4.0-beta.1", channel: "early-access", buildId: "test", schemaVersion: 23, updaterEnabled: false, signedBuild: false };
      if (command === "get_auth_status") return { initialized: state.initialized, sessionLifetimeSeconds: 28800 };
      if (command === "ensure_default_branch") return branch;
      if (command === "list_branches") return [branch];
      if (command === "bootstrap_admin") {
        if (state.initialized) throw { code: "conflict", message: "Initial administrator setup is already complete." };
        const input = args.input!; const user: User = { id: "admin-id", username: input.username!.toLowerCase(), displayName: input.displayName!, isActive: true, revision: 1, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(), roles: [roles[0]], branches: [] };
        state.users.push(user); state.passwords[user.username] = input.password!; state.initialized = true; state.token = "a".repeat(64); return { token: state.token, expiresAt: Math.floor(Date.now() / 1000) + 28800, user };
      }
      if (command === "login") {
        const name = String(args.input?.username).toLowerCase(); const user = state.users.find((item) => item.username === name && item.isActive);
        if (!user || state.passwords[name] !== args.input?.password) throw { code: "validation", message: "Invalid username or password." };
        state.token = "b".repeat(64); return { token: state.token, expiresAt: Math.floor(Date.now() / 1000) + 28800, user };
      }
      if (command === "logout") { state.token = ""; return null; }
      requireSession(args.sessionToken);
      if (command === "export_diagnostics") return "C:\\M2 Health\\diagnostics\\m2-health-diagnostics-test.json";
      if (command === "list_roles") return roles;
      if (command === "list_users") {
        const q = args.query ?? {}; let items = state.users.filter((user) => (!q.search || `${user.username} ${user.displayName}`.toLowerCase().includes(String(q.search).toLowerCase())) && (q.isActive === undefined || user.isActive === q.isActive) && (!q.roleCode || user.roles.some((role) => role.code === q.roleCode)));
        const total = items.length; items = items.slice(q.offset ?? 0, (q.offset ?? 0) + (q.limit ?? 20)); return { items, total, limit: q.limit ?? 20, offset: q.offset ?? 0 };
      }
      if (command === "get_user") { const user = state.users.find((item) => item.id === args.id); if (!user) throw { code: "notFound", message: "User not found." }; return user; }
      if (command === "create_user") { const input = args.input!; const user: User = { id: `user-${state.users.length}`, username: input.username!.toLowerCase(), displayName: input.displayName!, isActive: true, revision: 1, createdAt: new Date().toISOString(), updatedAt: new Date().toISOString(), roles: roles.filter((role) => input.roleCodes!.includes(role.code)), branches: input.branchIds!.map((id: string) => ({ branchId: id, branchCode: branch.code, branchName: branch.name, isPrimary: id === input.primaryBranchId })) }; state.users.push(user); state.passwords[user.username] = input.password!; return user; }
      if (command === "assign_user_role" || command === "remove_user_role") { const user = state.users.find((item) => item.id === args.input.userId)!; const role = roles.find((item) => item.code === args.input.roleCode)!; user.roles = command === "assign_user_role" ? [...user.roles, role] : user.roles.filter((item) => item.code !== role.code); user.revision++; return user; }
      if (command === "set_user_active") { const user = state.users.find((item) => item.id === args.input.id)!; user.isActive = args.input.active; user.revision++; return user; }
      if (command === "reset_user_password") { const user = state.users.find((item) => item.id === args.input.id)!; state.passwords[user.username] = args.input.password; return null; }
      if (command === "set_user_branches") { const user = state.users.find((item) => item.id === args.input.userId)!; user.branches = args.input.branchIds.map((id: string) => ({ branchId: id, branchCode: branch.code, branchName: branch.name, isPrimary: id === args.input.primaryBranchId })); user.revision++; return user; }
      throw new Error(`Unexpected native command: ${command}`);
    } } });
  }, { initialized, initialLicense });
}
