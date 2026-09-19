import { nativeRequest } from "./client";
export type CountStatus =
  "DRAFT" | "IN_PROGRESS" | "REVIEW" | "COMPLETED" | "CANCELLED";
export interface CountSession {
  id: string;
  sessionNumber: string;
  branchId: string;
  status: CountStatus;
  scopeType: string;
  notes: string | null;
  createdBy: string | null;
  revision: number;
  snapshotWatermark: number | null;
  createdAt: string;
  startedAt: string | null;
  reviewedAt: string | null;
  completedAt: string | null;
  cancelledAt: string | null;
}
export interface CountLine {
  id: string;
  sessionId: string;
  productPackageId: string;
  batchId: string;
  expectedQuantity: number;
  countedQuantity: number | null;
  notes: string | null;
  excludedReason: string | null;
  countedAt: string | null;
  revision: number;
  productLabel: string;
  packageLabel: string;
  lotNumber: string | null;
  expiryDate: string | null;
  currentLive: number;
  snapshotVariance: number | null;
  correctionDelta: number | null;
  interveningMovements: number;
  movementId: string | null;
}
export interface CountSummary {
  total: number;
  counted: number;
  uncounted: number;
  excluded: number;
  matches: number;
  positive: number;
  negative: number;
  corrections: number;
  intervening: number;
}
export interface CountDetail {
  session: CountSession;
  lines: CountLine[];
  summary: CountSummary;
  reviewToken: string;
  limit: number;
  offset: number;
}
export interface CountTransition {
  sessionId: string;
  expectedRevision: number;
}
export interface CountEntry extends CountTransition {
  lineId: string;
  lineRevision: number;
  countedQuantity: number | null;
  notes: string | null;
  excludedReason: string | null;
}
export interface CountDiscovery extends CountTransition {
  productPackageId: string;
  batchId: string | null;
  lotNumber: string | null;
  expiryDate: string | null;
  notes: string;
}
export const countTransition = (d: CountDetail): CountTransition => ({
  sessionId: d.session.id,
  expectedRevision: d.session.revision,
});
export const createCount = (branchId: string, notes: string | null) =>
  nativeRequest<CountDetail>("create_count_session", {
    input: { branchId, notes, createdBy: null },
  });
export const getCount = (id: string, offset = 0) =>
  nativeRequest<CountDetail>("get_count_session", { id, limit: 20, offset });
export const listCounts = (query: {
  branchId: string;
  status: CountStatus | null;
  dateFrom: string | null;
  dateTo: string | null;
  limit: number;
  offset: number;
}) =>
  nativeRequest<{
    items: (CountSession & { summary: CountSummary })[];
    total: number;
  }>("list_count_sessions", { query });
export const startCount = (input: CountTransition) =>
  nativeRequest<CountDetail>("start_count_session", { input });
export const recordCount = (input: CountEntry) =>
  nativeRequest<CountDetail>("record_count_entry", { input });
export const discoverCount = (input: CountDiscovery) =>
  nativeRequest<CountDetail>("discover_count_stock", { input });
export const reviewCount = (input: CountTransition) =>
  nativeRequest<CountDetail>("review_count_session", { input });
export const resumeCount = (input: CountTransition) =>
  nativeRequest<CountDetail>("resume_count_session", { input });
export const cancelCount = (input: CountTransition) =>
  nativeRequest<CountDetail>("cancel_count_session", { input });
export const completeCount = (d: CountDetail) => {
  const key = `m2:count:${d.session.id}:${d.reviewToken}`;
  let requestId = localStorage.getItem(key);
  if (!requestId) {
    requestId = crypto.randomUUID();
    localStorage.setItem(key, requestId);
  }
  return nativeRequest<CountDetail>("complete_count_session", {
    input: { ...countTransition(d), requestId, reviewToken: d.reviewToken },
  });
};
