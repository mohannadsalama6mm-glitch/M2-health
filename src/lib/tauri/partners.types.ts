// Phase 5 customer/supplier contracts. Mirrors src-tauri/src/db/models/{customers,suppliers}.rs.
export interface Customer {
  id: string;
  code: string;
  name: string;
  phone: string;
  email: string;
  address: string;
  notes: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface Supplier {
  id: string;
  code: string;
  name: string;
  phone: string;
  email: string;
  address: string;
  notes: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}
export interface CustomerPage {
  items: Customer[];
  total: number;
  limit: number;
  offset: number;
}
export interface SupplierPage {
  items: Supplier[];
  total: number;
  limit: number;
  offset: number;
}
export interface PartnerQuery {
  search?: string | null;
  isActive?: boolean | null;
  limit?: number | null;
  offset?: number | null;
}
export interface CreateCustomerInput {
  name: string;
  phone?: string;
  email?: string;
  address?: string;
  notes?: string;
}
export interface CreateSupplierInput {
  name: string;
  phone?: string;
  email?: string;
  address?: string;
  notes?: string;
}
export interface UpdateCustomerInput extends CreateCustomerInput {
  id: string;
}
export interface UpdateSupplierInput extends CreateSupplierInput {
  id: string;
}