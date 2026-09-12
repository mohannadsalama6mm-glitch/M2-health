import { nativeRequest } from "./client";
/** Real native DTO, intentionally separate from the Phase 1 demo Branch view model. */
export interface LocalBranch {
  id: string;
  code: string;
  name: string;
  phone: string;
  address: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface CreateLocalBranch {
  code: string;
  name: string;
  phone?: string;
  address?: string;
}
export const ensureDefaultBranch = () =>
  nativeRequest<LocalBranch>("ensure_default_branch");
export const listBranches = () => nativeRequest<LocalBranch[]>("list_branches");
export const getBranch = (id: string) =>
  nativeRequest<LocalBranch>("get_branch", { id });
// Available for future foundation consumers; no Phase 1 branch form is connected.
export const createBranch = (input: CreateLocalBranch) =>
  nativeRequest<LocalBranch>("create_branch", { input });
