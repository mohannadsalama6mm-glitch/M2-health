-- Forward-only Phase 9A.3 compatibility fix for modern Supabase secret keys.
--
-- A hosted sb_secret_ key is authenticated by Supabase's API gateway and the
-- request reaches PostgREST as the built-in service_role database role. The
-- key is not itself a JWT, so authorization must not depend on auth.role()
-- reading a role claim from that opaque key. EXECUTE grants are the complete
-- boundary for these SECURITY DEFINER functions.
begin;

create or replace function public.admin_set_license_state(
  p_account_id uuid,
  p_state text,
  p_plan_code text,
  p_starts_at timestamptz,
  p_ends_at timestamptz,
  p_max_devices integer,
  p_note text
) returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public as $$
declare before_row app.license_accounts%rowtype; after_row app.license_accounts%rowtype; op text;
begin
  if p_state not in ('ACTIVE','SUSPENDED','EXPIRED') or p_plan_code not in ('EARLY_ACCESS','SINGLE_PHARMACY','MULTI_BRANCH') or p_max_devices not between 1 and 100 or length(btrim(coalesce(p_note,''))) not between 3 and 1000 then raise exception 'invalid administrative license change' using errcode='22023'; end if;
  if p_state='ACTIVE' and (p_starts_at is null or (p_ends_at is not null and p_ends_at<=p_starts_at)) then raise exception 'active license dates are invalid' using errcode='22023'; end if;
  select * into before_row from app.license_accounts where id=p_account_id for update;
  if not found then raise exception 'license account not found' using errcode='P0002'; end if;
  update app.license_accounts set status=p_state,plan_code=p_plan_code,entitlement_source='MANUAL',license_started_at=coalesce(p_starts_at,license_started_at),license_ends_at=p_ends_at,max_devices=p_max_devices,last_admin_note=btrim(p_note) where id=p_account_id returning * into after_row;
  op:=case p_state when 'ACTIVE' then 'ACTIVATE' when 'SUSPENDED' then 'SUSPEND' else 'EXPIRE' end;
  insert into app.license_admin_audit(account_id,operation,previous_state,next_state,operator_subject,note) values(p_account_id,op,to_jsonb(before_row),to_jsonb(after_row),'supabase-secret-server',btrim(p_note));
  update app.activation_requests set status='FULFILLED',resolved_at=now(),resolution_note=btrim(p_note) where account_id=p_account_id and status='PENDING';
  return jsonb_build_object('accountId',p_account_id,'status',p_state,'planCode',p_plan_code,'maxDevices',p_max_devices);
end $$;

create or replace function public.admin_list_activation_requests()
returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public as $$
begin
  return coalesce((
    select jsonb_agg(jsonb_build_object('requestId',r.id,'accountId',r.account_id,'email',u.email,'message',r.message,'createdAt',r.created_at) order by r.created_at)
    from app.activation_requests r join app.license_accounts a on a.id=r.account_id join auth.users u on u.id=a.owner_user_id
    where r.status='PENDING'
  ),'[]'::jsonb);
end $$;

revoke all on function public.admin_set_license_state(uuid,text,text,timestamptz,timestamptz,integer,text) from public,anon,authenticated;
grant execute on function public.admin_set_license_state(uuid,text,text,timestamptz,timestamptz,integer,text) to service_role;
revoke all on function public.admin_list_activation_requests() from public,anon,authenticated;
grant execute on function public.admin_list_activation_requests() to service_role;

commit;
