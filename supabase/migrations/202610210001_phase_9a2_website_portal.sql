-- Phase 9A.2 website, customer portal, account trial, and license-device control.
-- Browser clients receive read access through forced RLS and mutate only through
-- narrowly scoped authenticated RPCs. Signing remains an Edge Function secret.
begin;

create table app.customer_profiles (
  user_id uuid primary key references auth.users(id) on delete restrict,
  full_name text not null default '' check (length(full_name) <= 160),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now()
);

create table app.license_accounts (
  id uuid primary key default gen_random_uuid(),
  owner_user_id uuid not null unique references auth.users(id) on delete restrict,
  plan_code text not null default 'EARLY_ACCESS' check (plan_code in ('EARLY_ACCESS','SINGLE_PHARMACY','MULTI_BRANCH')),
  status text not null default 'UNACTIVATED' check (status in ('UNACTIVATED','TRIAL','ACTIVE','EXPIRED','SUSPENDED')),
  entitlement_source text not null default 'TRIAL' check (entitlement_source in ('TRIAL','MANUAL','PAYMENT_FUTURE')),
  trial_started_at timestamptz,
  trial_ends_at timestamptz,
  license_started_at timestamptz,
  license_ends_at timestamptz,
  max_devices integer not null default 1 check (max_devices between 1 and 100),
  last_admin_note text check (last_admin_note is null or length(last_admin_note) <= 1000),
  created_at timestamptz not null default now(),
  updated_at timestamptz not null default now(),
  check ((trial_started_at is null and trial_ends_at is null) or trial_ends_at = trial_started_at + interval '7 days'),
  check (license_ends_at is null or (license_started_at is not null and license_ends_at > license_started_at)),
  check (status <> 'TRIAL' or (trial_started_at is not null and trial_ends_at is not null)),
  check (status <> 'ACTIVE' or license_started_at is not null)
);

create table app.license_devices (
  id uuid primary key,
  account_id uuid not null references app.license_accounts(id) on delete restrict,
  owner_user_id uuid not null references auth.users(id) on delete restrict,
  display_name text not null check (length(btrim(display_name)) between 2 and 120),
  status text not null default 'ACTIVE' check (status in ('ACTIVE','REVOKED')),
  activated_at timestamptz not null default now(),
  last_validated_at timestamptz not null default now(),
  app_version text check (app_version is null or length(app_version) <= 40),
  build_id text check (build_id is null or length(build_id) <= 80),
  revoked_at timestamptz,
  unique (account_id,id),
  check ((status='ACTIVE' and revoked_at is null) or (status='REVOKED' and revoked_at is not null))
);
create index license_devices_account on app.license_devices(account_id,status,activated_at);

create table app.activation_requests (
  id uuid primary key default gen_random_uuid(),
  account_id uuid not null references app.license_accounts(id) on delete restrict,
  requested_by uuid not null references auth.users(id) on delete restrict,
  status text not null default 'PENDING' check (status in ('PENDING','APPROVED','DENIED','FULFILLED')),
  message text not null check (length(btrim(message)) between 10 and 1000),
  created_at timestamptz not null default now(),
  resolved_at timestamptz,
  resolution_note text check (resolution_note is null or length(resolution_note) <= 1000),
  check ((status='PENDING' and resolved_at is null) or (status<>'PENDING' and resolved_at is not null))
);
create unique index activation_requests_one_pending on app.activation_requests(account_id) where status='PENDING';

create table app.license_admin_audit (
  id uuid primary key default gen_random_uuid(),
  account_id uuid not null references app.license_accounts(id) on delete restrict,
  operation text not null check (operation in ('ACTIVATE','SUSPEND','EXPIRE','PLAN_CHANGE')),
  previous_state jsonb not null,
  next_state jsonb not null,
  operator_subject text not null,
  note text not null check (length(btrim(note)) between 3 and 1000),
  occurred_at timestamptz not null default now()
);

create or replace function app.protect_trial_history()
returns trigger language plpgsql set search_path=pg_catalog,app as $$
begin
  if old.trial_started_at is not null and (new.trial_started_at is distinct from old.trial_started_at or new.trial_ends_at is distinct from old.trial_ends_at) then
    raise exception 'trial history is immutable' using errcode='42501';
  end if;
  new.updated_at := now();
  return new;
end $$;
create trigger license_accounts_protect_trial before update on app.license_accounts
for each row execute function app.protect_trial_history();

create or replace function app.ensure_license_account(target_user uuid)
returns uuid language plpgsql security definer
set search_path=pg_catalog,app,auth as $$
declare account_id uuid;
begin
  if target_user is null then raise exception 'authentication required' using errcode='42501'; end if;
  insert into app.customer_profiles(user_id,full_name)
  select target_user,coalesce(raw_user_meta_data->>'full_name','') from auth.users where id=target_user
  on conflict(user_id) do nothing;
  insert into app.license_accounts(owner_user_id) values(target_user)
  on conflict(owner_user_id) do nothing;
  select id into account_id from app.license_accounts where owner_user_id=target_user;
  return account_id;
end $$;

create or replace function public.handle_early_access_user()
returns trigger language plpgsql security definer
set search_path=pg_catalog,app,auth as $$
begin
  perform app.ensure_license_account(new.id);
  return new;
end $$;
create trigger early_access_auth_user after insert on auth.users
for each row execute function public.handle_early_access_user();

insert into app.customer_profiles(user_id,full_name)
select id,coalesce(raw_user_meta_data->>'full_name','') from auth.users on conflict do nothing;
insert into app.license_accounts(owner_user_id)
select id from auth.users on conflict do nothing;

alter table app.customer_profiles enable row level security;
alter table app.customer_profiles force row level security;
alter table app.license_accounts enable row level security;
alter table app.license_accounts force row level security;
alter table app.license_devices enable row level security;
alter table app.license_devices force row level security;
alter table app.activation_requests enable row level security;
alter table app.activation_requests force row level security;
alter table app.license_admin_audit enable row level security;
alter table app.license_admin_audit force row level security;

create policy customer_profiles_self_read on app.customer_profiles for select to authenticated using(user_id=auth.uid());
create policy license_accounts_owner_read on app.license_accounts for select to authenticated using(owner_user_id=auth.uid());
create policy license_devices_owner_read on app.license_devices for select to authenticated using(owner_user_id=auth.uid());
create policy activation_requests_owner_read on app.activation_requests for select to authenticated using(requested_by=auth.uid());

revoke all on app.customer_profiles,app.license_accounts,app.license_devices,app.activation_requests,app.license_admin_audit from anon,authenticated;
grant select on app.customer_profiles,app.license_accounts,app.license_devices,app.activation_requests to authenticated;

create or replace function public.update_customer_profile(p_full_name text)
returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public as $$
declare actor uuid:=auth.uid(); account_id uuid;
begin
  if actor is null then raise exception 'authentication required' using errcode='42501'; end if;
  if length(btrim(coalesce(p_full_name,''))) not between 2 and 160 then raise exception 'invalid full name' using errcode='22023'; end if;
  account_id:=app.ensure_license_account(actor);
  update app.customer_profiles set full_name=btrim(p_full_name),updated_at=now() where user_id=actor;
  return jsonb_build_object('accountId',account_id,'fullName',btrim(p_full_name));
end $$;

create or replace function public.start_early_access_trial()
returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public as $$
declare actor uuid:=auth.uid(); account_id uuid; started timestamptz:=clock_timestamp(); changed integer;
begin
  if actor is null then raise exception 'authentication required' using errcode='42501'; end if;
  account_id:=app.ensure_license_account(actor);
  update app.license_accounts set status='TRIAL',entitlement_source='TRIAL',trial_started_at=started,trial_ends_at=started+interval '7 days'
  where id=account_id and trial_started_at is null and status='UNACTIVATED';
  get diagnostics changed=row_count;
  if changed<>1 then raise exception 'the seven-day trial can be started only once per account' using errcode='23505'; end if;
  return jsonb_build_object('accountId',account_id,'status','TRIAL','trialStartedAt',started,'trialEndsAt',started+interval '7 days');
end $$;

create or replace function public.get_portal_snapshot()
returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public as $$
declare actor uuid:=auth.uid(); account_id uuid; result jsonb;
begin
  if actor is null then raise exception 'authentication required' using errcode='42501'; end if;
  account_id:=app.ensure_license_account(actor);
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
    'activationRequests',coalesce((select jsonb_agg(jsonb_build_object('id',r.id,'status',r.status,'message',r.message,'createdAt',r.created_at) order by r.created_at desc) from (select * from app.activation_requests where account_id=a.id order by created_at desc limit 10) r),'[]'::jsonb)
  ) into result
  from app.license_accounts a join app.customer_profiles p on p.user_id=a.owner_user_id join auth.users u on u.id=a.owner_user_id
  where a.id=account_id and a.owner_user_id=actor;
  return result;
end $$;

create or replace function public.request_manual_activation(p_message text)
returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public as $$
declare actor uuid:=auth.uid(); account_id uuid; request_id uuid;
begin
  if actor is null then raise exception 'authentication required' using errcode='42501'; end if;
  if length(btrim(coalesce(p_message,''))) not between 10 and 1000 then raise exception 'activation request must contain 10 to 1000 characters' using errcode='22023'; end if;
  account_id:=app.ensure_license_account(actor);
  insert into app.activation_requests(account_id,requested_by,message) values(account_id,actor,btrim(p_message)) returning id into request_id;
  return jsonb_build_object('requestId',request_id,'accountId',account_id,'status','PENDING');
exception when unique_violation then
  raise exception 'an activation request is already pending' using errcode='23505';
end $$;

create or replace function public.revoke_license_device(p_device_id uuid)
returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public as $$
declare actor uuid:=auth.uid(); changed integer;
begin
  if actor is null then raise exception 'authentication required' using errcode='42501'; end if;
  update app.license_devices set status='REVOKED',revoked_at=now(),last_validated_at=now()
  where id=p_device_id and owner_user_id=actor and status='ACTIVE';
  get diagnostics changed=row_count;
  if changed<>1 then raise exception 'active device not found for this account' using errcode='42501'; end if;
  return jsonb_build_object('deviceId',p_device_id,'status','REVOKED');
end $$;

create or replace function public.prepare_license_claims(
  p_device_id uuid,p_display_name text,p_app_version text default null,p_build_id text default null
) returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public as $$
declare actor uuid:=auth.uid(); account_row app.license_accounts%rowtype; device_row app.license_devices%rowtype; effective_status text; active_count integer; issued timestamptz:=clock_timestamp(); envelope_state text; envelope_expiry timestamptz; claims jsonb;
begin
  if actor is null then raise exception 'authentication required' using errcode='42501'; end if;
  if length(btrim(coalesce(p_display_name,''))) not between 2 and 120 or length(coalesce(p_app_version,''))>40 or length(coalesce(p_build_id,''))>80 then raise exception 'invalid device metadata' using errcode='22023'; end if;
  perform app.ensure_license_account(actor);
  select * into account_row from app.license_accounts where owner_user_id=actor for update;
  effective_status:=case when account_row.status='TRIAL' and account_row.trial_ends_at<=issued then 'EXPIRED' when account_row.status='ACTIVE' and account_row.license_ends_at is not null and account_row.license_ends_at<=issued then 'EXPIRED' else account_row.status end;
  select * into device_row from app.license_devices where id=p_device_id;
  if found and (device_row.account_id<>account_row.id or device_row.owner_user_id<>actor) then raise exception 'device belongs to another account' using errcode='42501'; end if;
  if not found then
    if effective_status not in ('TRIAL','ACTIVE') then return jsonb_build_object('status',effective_status,'claims',null); end if;
    select count(*) into active_count from app.license_devices where account_id=account_row.id and status='ACTIVE';
    if active_count>=account_row.max_devices then raise exception 'device limit reached for this plan' using errcode='23514'; end if;
    insert into app.license_devices(id,account_id,owner_user_id,display_name,app_version,build_id) values(p_device_id,account_row.id,actor,btrim(p_display_name),nullif(p_app_version,''),nullif(p_build_id,'')) returning * into device_row;
  else
    update app.license_devices set display_name=btrim(p_display_name),last_validated_at=issued,app_version=nullif(p_app_version,''),build_id=nullif(p_build_id,'') where id=p_device_id returning * into device_row;
  end if;
  envelope_state:=case when device_row.status='REVOKED' then 'REVOKED' when effective_status='SUSPENDED' then 'SUSPENDED' when effective_status in ('TRIAL','ACTIVE') then 'ACTIVE' else null end;
  if envelope_state is null then return jsonb_build_object('status',effective_status,'claims',null); end if;
  envelope_expiry:=case when effective_status='TRIAL' then account_row.trial_ends_at when effective_status='ACTIVE' then account_row.license_ends_at else null end;
  claims:=jsonb_build_object('version',1,'licenseId',account_row.id,'deviceId',device_row.id,'state',envelope_state,'issuedAt',floor(extract(epoch from issued))::bigint,'expiresAt',case when envelope_expiry is null then null else floor(extract(epoch from envelope_expiry))::bigint end,'revalidateAfter',floor(extract(epoch from (issued+interval '30 days')))::bigint,'offlineGraceDays',3);
  return jsonb_build_object('status',case when device_row.status='REVOKED' then 'REVOKED' else effective_status end,'claims',claims);
end $$;

create or replace function public.admin_set_license_state(
  p_account_id uuid,p_state text,p_plan_code text,p_starts_at timestamptz,p_ends_at timestamptz,p_max_devices integer,p_note text
) returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public as $$
declare before_row app.license_accounts%rowtype; after_row app.license_accounts%rowtype; op text; operator_text text;
begin
  if auth.role()<>'service_role' then raise exception 'server administrator credential required' using errcode='42501'; end if;
  if p_state not in ('ACTIVE','SUSPENDED','EXPIRED') or p_plan_code not in ('EARLY_ACCESS','SINGLE_PHARMACY','MULTI_BRANCH') or p_max_devices not between 1 and 100 or length(btrim(coalesce(p_note,''))) not between 3 and 1000 then raise exception 'invalid administrative license change' using errcode='22023'; end if;
  if p_state='ACTIVE' and (p_starts_at is null or (p_ends_at is not null and p_ends_at<=p_starts_at)) then raise exception 'active license dates are invalid' using errcode='22023'; end if;
  select * into before_row from app.license_accounts where id=p_account_id for update;
  if not found then raise exception 'license account not found' using errcode='P0002'; end if;
  update app.license_accounts set status=p_state,plan_code=p_plan_code,entitlement_source='MANUAL',license_started_at=coalesce(p_starts_at,license_started_at),license_ends_at=p_ends_at,max_devices=p_max_devices,last_admin_note=btrim(p_note) where id=p_account_id returning * into after_row;
  op:=case p_state when 'ACTIVE' then 'ACTIVATE' when 'SUSPENDED' then 'SUSPEND' else 'EXPIRE' end;
  operator_text:=coalesce(auth.uid()::text,current_setting('request.jwt.claim.sub',true),'service_role');
  insert into app.license_admin_audit(account_id,operation,previous_state,next_state,operator_subject,note) values(p_account_id,op,to_jsonb(before_row),to_jsonb(after_row),operator_text,btrim(p_note));
  update app.activation_requests set status='FULFILLED',resolved_at=now(),resolution_note=btrim(p_note) where account_id=p_account_id and status='PENDING';
  return jsonb_build_object('accountId',p_account_id,'status',p_state,'planCode',p_plan_code,'maxDevices',p_max_devices);
end $$;

create or replace function public.admin_list_activation_requests()
returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public as $$
begin
  if auth.role()<>'service_role' then raise exception 'server administrator credential required' using errcode='42501'; end if;
  return coalesce((
    select jsonb_agg(jsonb_build_object('requestId',r.id,'accountId',r.account_id,'email',u.email,'message',r.message,'createdAt',r.created_at) order by r.created_at)
    from app.activation_requests r join app.license_accounts a on a.id=r.account_id join auth.users u on u.id=a.owner_user_id
    where r.status='PENDING'
  ),'[]'::jsonb);
end $$;

revoke all on function app.protect_trial_history(),app.ensure_license_account(uuid),public.handle_early_access_user() from public,anon,authenticated;
revoke all on function public.update_customer_profile(text),public.start_early_access_trial(),public.get_portal_snapshot(),public.request_manual_activation(text),public.revoke_license_device(uuid),public.prepare_license_claims(uuid,text,text,text) from public,anon;
grant execute on function public.update_customer_profile(text),public.start_early_access_trial(),public.get_portal_snapshot(),public.request_manual_activation(text),public.revoke_license_device(uuid),public.prepare_license_claims(uuid,text,text,text) to authenticated;
revoke all on function public.admin_set_license_state(uuid,text,text,timestamptz,timestamptz,integer,text) from public,anon,authenticated;
grant execute on function public.admin_set_license_state(uuid,text,text,timestamptz,timestamptz,integer,text) to service_role;
revoke all on function public.admin_list_activation_requests() from public,anon,authenticated;
grant execute on function public.admin_list_activation_requests() to service_role;

commit;
