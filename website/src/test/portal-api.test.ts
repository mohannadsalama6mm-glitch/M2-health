import { beforeEach, expect, vi } from "vitest";
import { issueDeviceEnvelope, revokeDevice, startTrial } from "../portal/api";

const rpc = vi.fn();
const invoke = vi.fn();
vi.mock("../lib/supabase", () => ({
  getSupabase: () => ({ rpc, functions: { invoke } }),
}));

describe("portal server boundaries", () => {
  beforeEach(() => { rpc.mockReset(); invoke.mockReset(); });

  it("starts a trial through the one-time server RPC", async () => {
    rpc.mockResolvedValueOnce({ data: { status: "TRIAL" }, error: null });
    rpc.mockResolvedValueOnce({ data: { status: "TRIAL", devices: [] }, error: null });
    const result = await startTrial();
    expect(rpc).toHaveBeenNthCalledWith(1, "start_early_access_trial");
    expect(result.status).toBe("TRIAL");
  });

  it("revolves device revocation around the authenticated RPC", async () => {
    rpc.mockResolvedValueOnce({ data: { status: "REVOKED" }, error: null });
    rpc.mockResolvedValueOnce({ data: { status: "ACTIVE", devices: [] }, error: null });
    await revokeDevice("93000000-0000-4000-8000-000000000001");
    expect(rpc).toHaveBeenNthCalledWith(1, "revoke_license_device", { p_device_id: "93000000-0000-4000-8000-000000000001" });
  });

  it("requests a signed envelope only through the authenticated Edge Function", async () => {
    invoke.mockResolvedValue({ data: { status: "TRIAL", activationCode: "payload.signature", expiresAt: 42, revalidateAfter: 40 }, error: null });
    const result = await issueDeviceEnvelope("93000000-0000-4000-8000-000000000001", "Counter A");
    expect(invoke).toHaveBeenCalledWith("license-state", { body: expect.objectContaining({ deviceId: "93000000-0000-4000-8000-000000000001", buildId: "20261020.1" }) });
    expect(result.activationCode).toBe("payload.signature");
  });
});
