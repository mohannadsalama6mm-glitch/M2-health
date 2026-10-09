import { nativeRequest } from "./client";

export interface Role {
  id: string;
  code: string;
  name: string;
  description: string;
  isSystem: boolean;
}
export interface BranchMembership {
  branchId: string;
  branchCode: string;
  branchName: string;
  isPrimary: boolean;
}
export interface User {
  id: string;
  username: string;
  displayName: string;
  isActive: boolean;
  revision: number;
  createdAt: string;
  updatedAt: string;
  roles: Role[];
  branches: BranchMembership[];
}
export interface LoginResult {
  token: string;
  expiresAt: number;
  user: User;
}
export interface UserQuery {
  search?: string;
  isActive?: boolean;
  roleCode?: string;
  limit?: number;
  offset?: number;
}
export interface UserPage {
  items: User[];
  total: number;
  limit: number;
  offset: number;
}
export const authStatus = () =>
  nativeRequest<{ initialized: boolean; sessionLifetimeSeconds: number }>("get_auth_status");
export const bootstrapAdmin = (input: { username: string; displayName: string; password: string }) =>
  nativeRequest<LoginResult>("bootstrap_admin", { input });
export const login = (input: { username: string; password: string }) =>
  nativeRequest<LoginResult>("login", { input });
export const currentSession = (sessionToken: string) =>
  nativeRequest<User>("current_session", { sessionToken });
export const logout = (sessionToken: string) =>
  nativeRequest<void>("logout", { sessionToken });
export const listUsers = (sessionToken: string, query: UserQuery) =>
  nativeRequest<UserPage>("list_users", { sessionToken, query });
export const getUser = (sessionToken: string, id: string) =>
  nativeRequest<User>("get_user", { sessionToken, id });
export const createUser = (sessionToken: string, input: { username: string; displayName: string; password: string; roleCodes: string[]; branchIds: string[]; primaryBranchId: string | null }) =>
  nativeRequest<User>("create_user", { sessionToken, input });
export const updateUser = (sessionToken: string, input: { id: string; expectedRevision: number; username: string; displayName: string }) =>
  nativeRequest<User>("update_user", { sessionToken, input });
export const setUserActive = (sessionToken: string, input: { id: string; expectedRevision: number; active: boolean }) =>
  nativeRequest<User>("set_user_active", { sessionToken, input });
export const resetUserPassword = (sessionToken: string, input: { id: string; password: string }) =>
  nativeRequest<void>("reset_user_password", { sessionToken, input });
export const listRoles = (sessionToken: string) =>
  nativeRequest<Role[]>("list_roles", { sessionToken });
export const assignUserRole = (sessionToken: string, input: { userId: string; roleCode: string }) =>
  nativeRequest<User>("assign_user_role", { sessionToken, input });
export const removeUserRole = (sessionToken: string, input: { userId: string; roleCode: string }) =>
  nativeRequest<User>("remove_user_role", { sessionToken, input });
export const setUserBranches = (sessionToken: string, input: { userId: string; branchIds: string[]; primaryBranchId: string | null }) =>
  nativeRequest<User>("set_user_branches", { sessionToken, input });
