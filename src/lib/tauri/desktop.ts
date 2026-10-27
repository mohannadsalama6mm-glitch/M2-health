import { nativeRequest } from "./client";

export interface LocalSettings {
  pharmacyName: string;
  pharmacyPhone: string;
  pharmacyAddress: string;
  registrationNumber: string;
  timezone: "Africa/Cairo" | "UTC";
  receiptHeader: string;
  receiptFooter: string;
  updatedAt: string;
}
export interface AuditEvent {
  id: string;
  actorUserId: string;
  actorName: string;
  subjectUserId: string | null;
  subjectName: string | null;
  eventType: string;
  detail: string;
  createdAt: string;
}
export interface AuditQuery {
  search: string | null;
  eventType: string | null;
  fromDate: string | null;
  toDate: string | null;
  limit: number;
  offset: number;
}
export interface BackupRecord {
  id: string;
  createdAt: string;
  sizeBytes: number;
  schemaVersion: number;
  integrity: string;
}
export interface BackupStatus {
  managedLocation: string;
  backups: BackupRecord[];
}
export const getLocalSettings = () => nativeRequest<LocalSettings>("get_local_settings");
export const updateLocalSettings = (input: LocalSettings) => nativeRequest<LocalSettings>("update_local_settings", { input });
export const listAuditEvents = (query: AuditQuery) => nativeRequest<{items: AuditEvent[]; total: number}>("list_security_audit_events", { query });
export const getBackupStatus = () => nativeRequest<BackupStatus>("get_local_backup_status");
export const createBackup = () => nativeRequest<BackupRecord>("create_local_backup");
export const validateBackup = (id: string) => nativeRequest<BackupRecord>("validate_local_backup", { id });
export const restoreBackup = (id: string, confirmation: string, reason: string) => nativeRequest<{safetyBackupId:string; restartRequired:boolean}>("restore_local_backup", { id, confirmation, reason });
