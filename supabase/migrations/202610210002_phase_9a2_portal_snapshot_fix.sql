-- Forward-only Phase 9A.2 fix: qualify the activation request account column
-- so it cannot collide with the PL/pgSQL account_id variable.
begin;

create or replace function public.get_portal_snapshot()
returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public as $$
declare actor uuid:=auth.uid(); selected_account_id uuid; result jsonb;
begin
  if actor is null then raise exception 'authentication required' using errcode='42501'; end if;
  selected_account_id:=app.ensure_license_account(actor);
  select jsonb_build_object(
    'accountId',a.id,'fullName',p.full_name,'email',u.email,'planCode',a.plan_code,
    'status',case when a.status='TRIAL' and a.trial_ends_at<=now() then 'EXPIRED' when a.status='ACTIVE' and a.license_ends_at is not null and a.license_ends_at<=now() then 'EXPIRED' else a.status end,
    'trialStartedAt',a.trial_started_at,'trialEndsAt',a.trial_ends_at,'licenseStartsAt',a.license_started_at,'licenseEndsAt',a.license_ends_at,
    'daysRemaining',case
      when a.status='TRIAL' and a.trial_ends_at>now() then greatest(0,ceil(extract(epoch from (a.trial_ends_at-now()))/86400.0)::integer)
      when a.status='ACTIVE' and a.license_ends_at is not null and a.license_ends_at>now() then greatest(0,ceil(extract(epoch from (a.license_ends_at-now()))/86400.0)::integer)
      else null end,
    'maxDevices',a.max_devices,
    'devices',coalesce((select jsonb_agg(jsonb_build_object('id',d.id,'displayName',d.display_name,'status',d.status,'activatedAt',d.activated_at,'lastValidatedAt',d.last_validated_at,'appVersion',d.app_version,'buildId',d.build_id) order by d.activated_at desc) from app.license_devices d where d.account_id=a.id),'[]'::jsonb),
    'activationRequests',coalesce((select jsonb_agg(jsonb_build_object('id',r.id,'status',r.status,'message',r.message,'createdAt',r.created_at) order by r.created_at desc) from (select ar.* from app.activation_requests ar where ar.account_id=a.id order by ar.created_at desc limit 10) r),'[]'::jsonb)
  ) into result
  from app.license_accounts a join app.customer_profiles p on p.user_id=a.owner_user_id join auth.users u on u.id=a.owner_user_id
  where a.id=selected_account_id and a.owner_user_id=actor;
  return result;
end $$;

revoke all on function public.get_portal_snapshot() from public,anon;
grant execute on function public.get_portal_snapshot() to authenticated;

commit;
