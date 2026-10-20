import { getSupabase } from "../lib/supabase";
import type { LicenseEnvelopeResponse, PortalSnapshot } from "./types";

function unwrap<T>(data: unknown, error: { message: string } | null): T {
  if (error) throw new Error(error.message);
  return data as T;
}

export async function getPortalSnapshot(): Promise<PortalSnapshot> {
  const { data, error } = await getSupabase().rpc("get_portal_snapshot");
  return unwrap<PortalSnapshot>(data, error);
}

export async function startTrial(): Promise<PortalSnapshot> {
  const { data, error } = await getSupabase().rpc("start_early_access_trial");
  unwrap(data, error);
  return getPortalSnapshot();
}

export async function revokeDevice(deviceId: string): Promise<PortalSnapshot> {
  const { data, error } = await getSupabase().rpc("revoke_license_device", {
    p_device_id: deviceId,
  });
  unwrap(data, error);
  return getPortalSnapshot();
}

export async function requestActivation(message: string): Promise<PortalSnapshot> {
  const { data, error } = await getSupabase().rpc("request_manual_activation", {
    p_message: message,
  });
  unwrap(data, error);
  return getPortalSnapshot();
}

export async function updateProfile(fullName: string): Promise<PortalSnapshot> {
  const { data, error } = await getSupabase().rpc("update_customer_profile", {
    p_full_name: fullName,
  });
  unwrap(data, error);
  return getPortalSnapshot();
}

export async function issueDeviceEnvelope(
  deviceId: string,
  displayName: string,
): Promise<LicenseEnvelopeResponse> {
  const { data, error } = await getSupabase().functions.invoke<LicenseEnvelopeResponse>(
    "license-state",
    {
      body: {
        deviceId,
        displayName,
        appVersion: "0.4.0-beta.1",
        buildId: "20261020.1",
      },
    },
  );
  if (error) throw error;
  if (!data) throw new Error("The license service returned no response.");
  return data;
}
