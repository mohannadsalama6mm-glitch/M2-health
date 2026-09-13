import { nativeRequest } from "./client";
export interface ImportProfile {
  path: string;
  hash: string;
  size: number;
  totalRows: number;
  columns: string[];
  profiledAt: string;
}
export interface ImportRow {
  number: number;
  fields: string[];
  fingerprint: string;
  identity: string;
  disposition: string;
  warnings: string[];
  priceMinor: number | null;
}
export interface ImportReport {
  id: string;
  mode: string;
  status: string;
  profile: ImportProfile | null;
  counts: Record<string, number>;
  lookupCreated: Record<string, number>;
  lookupReused: Record<string, number>;
  imported: number;
  processed: number;
  durationMs: number;
  rowsPerSecond: number;
  reportPath: string;
  backupPath: string | null;
  error: string | null;
  rows: ImportRow[];
}
export const profileCatalogSource = () =>
  nativeRequest<ImportProfile>("profile_catalog_source");
export const dryRunCatalogImport = () =>
  nativeRequest<ImportReport>("dry_run_catalog_import");
export const applyCatalogImport = (planId: string) =>
  nativeRequest<ImportReport>("apply_catalog_import", {
    planId,
    confirmed: true,
  });
export const getCatalogImportStatus = () =>
  nativeRequest<ImportReport | null>("get_catalog_import_status");
export const getCatalogImportReport = () =>
  nativeRequest<ImportReport | null>("get_catalog_import_report");
