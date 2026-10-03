import { nativeRequest } from "./client";

export interface Supplier {
  id: string;
  code: string;
  displayName: string;
  contactPerson: string | null;
  primaryPhone: string | null;
  secondaryPhone: string | null;
  email: string | null;
  address: string | null;
  taxRegistrationId: string | null;
  notes: string | null;
  isActive: boolean;
  revision: number;
  createdAt: string;
  updatedAt: string;
}
export interface Customer {
  id: string;
  code: string;
  displayName: string;
  primaryPhone: string | null;
  secondaryPhone: string | null;
  email: string | null;
  address: string | null;
  notes: string | null;
  isActive: boolean;
  revision: number;
  createdAt: string;
  updatedAt: string;
}
export interface SupplierFields {
  displayName: string;
  contactPerson?: string | null;
  primaryPhone?: string | null;
  secondaryPhone?: string | null;
  email?: string | null;
  address?: string | null;
  taxRegistrationId?: string | null;
  notes?: string | null;
}
export interface CustomerFields {
  displayName: string;
  primaryPhone?: string | null;
  secondaryPhone?: string | null;
  email?: string | null;
  address?: string | null;
  notes?: string | null;
}
export interface PartnerQuery {
  search?: string | null;
  isActive?: boolean | null;
  limit?: number;
  offset?: number;
}
export interface Page<T> {
  items: T[];
  total: number;
  limit: number;
  offset: number;
}

export const canonicalPartners = {
  listSuppliers: (query: PartnerQuery) =>
    nativeRequest<Page<Supplier>>("list_canonical_suppliers", { query }),
  getSupplier: (id: string) =>
    nativeRequest<Supplier>("get_canonical_supplier", { id }),
  createSupplier: (input: SupplierFields) =>
    nativeRequest<Supplier>("create_canonical_supplier", { input }),
  updateSupplier: (
    input: SupplierFields & { id: string; expectedRevision: number },
  ) => nativeRequest<Supplier>("update_canonical_supplier", { input }),
  setSupplierActive: (input: {
    id: string;
    expectedRevision: number;
    active: boolean;
  }) => nativeRequest<Supplier>("set_canonical_supplier_active", { input }),
  listCustomers: (query: PartnerQuery) =>
    nativeRequest<Page<Customer>>("list_canonical_customers", { query }),
  getCustomer: (id: string) =>
    nativeRequest<Customer>("get_canonical_customer", { id }),
  createCustomer: (input: CustomerFields) =>
    nativeRequest<Customer>("create_canonical_customer", { input }),
  updateCustomer: (
    input: CustomerFields & { id: string; expectedRevision: number },
  ) => nativeRequest<Customer>("update_canonical_customer", { input }),
  setCustomerActive: (input: {
    id: string;
    expectedRevision: number;
    active: boolean;
  }) => nativeRequest<Customer>("set_canonical_customer_active", { input }),
};
