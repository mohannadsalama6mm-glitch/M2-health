-- Phase 8C trusted tenant bootstrap, membership administration, revocation,
-- and server-side payload integrity. Desktop clients receive no table writes.
begin;

create or replace function app.canonical_jsonb(value jsonb)
returns text language sql immutable strict
set search_path=pg_catalog,app
as $$
  select case jsonb_typeof(value)
    when 'object' then coalesce((
      select '{'||string_agg(to_jsonb(key)::text||':'||app.canonical_jsonb(val),',' order by key)||'}'
      from jsonb_each(value) as item(key,val)
    ),'{}')
    when 'array' then coalesce((
      select '['||string_agg(app.canonical_jsonb(val),',' order by position)||']'
      from jsonb_array_elements(value) with ordinality as item(val,position)
    ),'[]')
    else value::text
  end
$$;

create or replace function app.verify_sync_payload_hash()
returns trigger language plpgsql security definer
set search_path=pg_catalog,app,extensions
as $$
begin
  if new.payload_hash<>encode(extensions.digest(convert_to(app.canonical_jsonb(new.payload),'UTF8'),'sha256'),'hex') then
    raise exception 'payload hash does not match canonical payload' using errcode='22023';
  end if;
  return new;
end $$;

create trigger sync_events_payload_integrity before insert or update of payload,payload_hash
on app.sync_events for each row execute function app.verify_sync_payload_hash();

create or replace function public.bootstrap_sync_organization(
  p_organization_id uuid,
  p_slug text,
  p_name text,
  p_branch_id uuid,
  p_branch_code text,
  p_branch_name text,
  p_device_id uuid,
  p_device_name text
) returns jsonb
language plpgsql security definer
set search_path=pg_catalog,app,auth,public
as $$
declare actor uuid := auth.uid(); existing_owner uuid;
begin
  if actor is null then raise exception 'authentication required' using errcode='42501'; end if;
  if length(btrim(coalesce(p_slug,''))) not between 3 and 80
     or p_slug !~ '^[a-z0-9][a-z0-9-]*[a-z0-9]$'
     or length(btrim(coalesce(p_name,''))) not between 1 and 160
     or length(btrim(coalesce(p_branch_code,''))) not between 1 and 32
     or length(btrim(coalesce(p_branch_name,''))) not between 1 and 160
     or length(btrim(coalesce(p_device_name,''))) not between 1 and 160 then
    raise exception 'invalid organization bootstrap input' using errcode='22023';
  end if;
  select user_id into existing_owner from app.organization_memberships
   where organization_id=p_organization_id and role_code='OWNER' and status='ACTIVE';
  if existing_owner is not null and existing_owner<>actor then
    raise exception 'organization identity is unavailable' using errcode='42501';
  end if;
  if existing_owner is null and exists(
    select 1 from app.organization_memberships where user_id=actor and status='ACTIVE' and role_code='OWNER'
  ) then
    raise exception 'user already owns an organization' using errcode='42501';
  end if;
  insert into app.organizations(id,slug,name) values(p_organization_id,btrim(p_slug),btrim(p_name))
    on conflict(id) do nothing;
  if not exists(select 1 from app.organizations where id=p_organization_id and slug=btrim(p_slug) and name=btrim(p_name) and status='ACTIVE') then
    raise exception 'organization identity conflicts with existing data' using errcode='23505';
  end if;
  insert into app.organization_memberships(organization_id,user_id,role_code,granted_by)
    values(p_organization_id,actor,'OWNER',actor)
    on conflict(organization_id,user_id) do update set status='ACTIVE',role_code='OWNER',revoked_at=null;
  insert into app.branches(id,organization_id,code,name)
    values(p_branch_id,p_organization_id,btrim(p_branch_code),btrim(p_branch_name))
    on conflict(id) do nothing;
  if not exists(select 1 from app.branches where id=p_branch_id and organization_id=p_organization_id and code=btrim(p_branch_code) and status='ACTIVE') then
    raise exception 'branch identity conflicts with existing data' using errcode='23505';
  end if;
  insert into app.branch_memberships(organization_id,branch_id,user_id,granted_by)
    values(p_organization_id,p_branch_id,actor,actor) on conflict do nothing;
  insert into app.devices(id,organization_id,owner_user_id,display_name,status)
    values(p_device_id,p_organization_id,actor,btrim(p_device_name),'ENROLLED')
    on conflict(id) do nothing;
  if not exists(select 1 from app.devices where id=p_device_id and organization_id=p_organization_id and owner_user_id=actor and status='ENROLLED') then
    raise exception 'device identity conflicts with existing data' using errcode='42501';
  end if;
  return jsonb_build_object('organizationId',p_organization_id,'branchId',p_branch_id,'deviceId',p_device_id,'cloudUserId',actor,'status','ENROLLED');
end $$;

create or replace function public.grant_sync_membership(
  p_organization_id uuid,p_user_id uuid,p_role_code text,p_branch_ids uuid[]
) returns jsonb
language plpgsql security definer
set search_path=pg_catalog,app,auth,public
as $$
declare actor uuid:=auth.uid(); branch_id uuid;
begin
  if actor is null or not exists(select 1 from app.organization_memberships where organization_id=p_organization_id and user_id=actor and status='ACTIVE' and role_code='OWNER') then
    raise exception 'active organization owner required' using errcode='42501';
  end if;
  if p_role_code not in ('ADMIN','MEMBER') or coalesce(array_length(p_branch_ids,1),0)=0 then
    raise exception 'invalid membership grant' using errcode='22023';
  end if;
  if not exists(select 1 from auth.users where id=p_user_id) then raise exception 'cloud user not found' using errcode='22023'; end if;
  insert into app.organization_memberships(organization_id,user_id,role_code,status,granted_by,revoked_at)
    values(p_organization_id,p_user_id,p_role_code,'ACTIVE',actor,null)
    on conflict(organization_id,user_id) do update set role_code=excluded.role_code,status='ACTIVE',granted_by=actor,granted_at=now(),revoked_at=null;
  foreach branch_id in array p_branch_ids loop
    if not exists(select 1 from app.branches where organization_id=p_organization_id and id=branch_id and status='ACTIVE') then
      raise exception 'branch is unavailable' using errcode='22023';
    end if;
    insert into app.branch_memberships(organization_id,branch_id,user_id,granted_by)
      values(p_organization_id,branch_id,p_user_id,actor) on conflict do nothing;
  end loop;
  return jsonb_build_object('organizationId',p_organization_id,'userId',p_user_id,'roleCode',p_role_code,'status','ACTIVE');
end $$;

create or replace function public.revoke_sync_device(p_device_id uuid)
returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public
as $$
declare actor uuid:=auth.uid(); org uuid; owner_id uuid;
begin
  select organization_id,owner_user_id into org,owner_id from app.devices where id=p_device_id and status='ENROLLED';
  if actor is null or org is null or (actor<>owner_id and not app.is_org_admin(org)) then
    raise exception 'device revocation denied' using errcode='42501';
  end if;
  update app.devices set status='REVOKED',revoked_at=now() where id=p_device_id;
  return jsonb_build_object('deviceId',p_device_id,'status','REVOKED');
end $$;

create or replace function public.revoke_sync_member(p_organization_id uuid,p_user_id uuid)
returns jsonb language plpgsql security definer
set search_path=pg_catalog,app,auth,public
as $$
declare actor uuid:=auth.uid(); target_role text;
begin
  if actor is null or not exists(select 1 from app.organization_memberships where organization_id=p_organization_id and user_id=actor and status='ACTIVE' and role_code='OWNER') then
    raise exception 'active organization owner required' using errcode='42501';
  end if;
  select role_code into target_role from app.organization_memberships where organization_id=p_organization_id and user_id=p_user_id and status='ACTIVE';
  if target_role is null or target_role='OWNER' then raise exception 'owner membership cannot be revoked here' using errcode='42501'; end if;
  update app.organization_memberships set status='REVOKED',revoked_at=now() where organization_id=p_organization_id and user_id=p_user_id;
  update app.devices set status='REVOKED',revoked_at=now() where organization_id=p_organization_id and owner_user_id=p_user_id and status='ENROLLED';
  return jsonb_build_object('organizationId',p_organization_id,'userId',p_user_id,'status','REVOKED');
end $$;

revoke all on function app.canonical_jsonb(jsonb),app.verify_sync_payload_hash() from public,anon,authenticated;
revoke all on function public.bootstrap_sync_organization(uuid,text,text,uuid,text,text,uuid,text) from public,anon;
revoke all on function public.grant_sync_membership(uuid,uuid,text,uuid[]) from public,anon;
revoke all on function public.revoke_sync_device(uuid) from public,anon;
revoke all on function public.revoke_sync_member(uuid,uuid) from public,anon;
grant execute on function public.bootstrap_sync_organization(uuid,text,text,uuid,text,text,uuid,text) to authenticated;
grant execute on function public.grant_sync_membership(uuid,uuid,text,uuid[]) to authenticated;
grant execute on function public.revoke_sync_device(uuid) to authenticated;
grant execute on function public.revoke_sync_member(uuid,uuid) to authenticated;

commit;
