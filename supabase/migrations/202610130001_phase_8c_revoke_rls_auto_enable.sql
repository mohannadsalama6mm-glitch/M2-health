-- Event-trigger helper is not an application RPC. Keep it callable only by
-- privileged database roles that own DDL event triggers.
revoke all on function public.rls_auto_enable() from public, anon, authenticated;
