import { nativeRequest } from "./client";
import type {
  Customer,
  CustomerPage,
  Supplier,
  SupplierPage,
  PartnerQuery,
  CreateCustomerInput,
  CreateSupplierInput,
  UpdateCustomerInput,
  UpdateSupplierInput,
} from "./partners.types";

export const createCustomer = (input: CreateCustomerInput) =>
  nativeRequest<Customer>("create_customer", { input });
export const listCustomers = (query: PartnerQuery = {}) =>
  nativeRequest<CustomerPage>("list_customers", { query });
export const getCustomer = (id: string) =>
  nativeRequest<Customer>("get_customer", { id });
export const updateCustomer = (input: UpdateCustomerInput) =>
  nativeRequest<Customer>("update_customer", { input });
export const setCustomerActive = (id: string, active: boolean) =>
  nativeRequest<Customer>("set_customer_active", { id, active });
export const createSupplier = (input: CreateSupplierInput) =>
  nativeRequest<Supplier>("create_supplier", { input });
export const listSuppliers = (query: PartnerQuery = {}) =>
  nativeRequest<SupplierPage>("list_suppliers", { query });
export const getSupplier = (id: string) =>
  nativeRequest<Supplier>("get_supplier", { id });
export const updateSupplier = (input: UpdateSupplierInput) =>
  nativeRequest<Supplier>("update_supplier", { input });
export const setSupplierActive = (id: string, active: boolean) =>
  nativeRequest<Supplier>("set_supplier_active", { id, active });