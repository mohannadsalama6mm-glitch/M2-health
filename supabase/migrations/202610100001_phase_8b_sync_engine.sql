-- Phase 8B authenticated, tenant-scoped synchronization RPCs.
-- Direct table mutation remains denied to desktop clients. The RPCs are the
-- only write surface and derive the cloud actor from auth.uid().
begin;

alter table app.sync_events
  add column payload_hash text not null
    check (payload_hash ~ '^[0-9a-f]{64}$'),
  add column local_sequence bigint not null
    check (local_sequence > 0);

create unique index sync_events_device_sequence
  on app.sync_events(organization_id,device_id,local_sequence);

create or replace function public.enroll_sync_device(
  p_organization_id uuid,
  p_device_id uuid,
  p_display_name text
) returns jsonb
language plpgsql
security definer
set search_path=pg_catalog,app,auth,public
as $$
declare
  actor uuid := auth.uid();
  existing app.devices%rowtype;
begin
  if actor is null then raise exception 'authentication required' using errcode='42501'; end if;
  if length(btrim(coalesce(p_display_name,''))) not between 1 and 160 then
    raise exception 'invalid device name' using errcode='22023';
  end if;
  if not app.is_active_org_member(p_organization_id) then
    raise exception 'active organization membership required' using errcode='42501';
  end if;

  select * into existing from app.devices
   where organization_id=p_organization_id and id=p_device_id;
  if found then
    if existing.owner_user_id<>actor or existing.status<>'ENROLLED' then
      raise exception 'device identity is unavailable' using errcode='42501';
    end if;
  else
    insert into app.devices(id,organization_id,owner_user_id,display_name,status)
    values(p_device_id,p_organization_id,actor,btrim(p_display_name),'ENROLLED');
  end if;
  return jsonb_build_object(
    'deviceId',p_device_id,
    'organizationId',p_organization_id,
    'cloudUserId',actor,
    'status','ENROLLED'
  );
end $$;

create or replace function public.push_sync_events(
  p_device_id uuid,
  p_events jsonb
) returns jsonb
language plpgsql
security definer
set search_path=pg_catalog,app,auth,public
as $$
declare
  actor uuid := auth.uid();
  org uuid;
  item jsonb;
  event_uuid uuid;
  branch_uuid uuid;
  causation_uuid uuid;
  existing app.sync_events%rowtype;
  inserted_sequence bigint;
  result jsonb := '[]'::jsonb;
  was_duplicate boolean;
begin
  if actor is null then raise exception 'authentication required' using errcode='42501'; end if;
  select organization_id into org from app.devices
   where id=p_device_id and owner_user_id=actor and status='ENROLLED';
  if org is null or not app.is_active_org_member(org) then
    raise exception 'active enrolled device required' using errcode='42501';
  end if;
  if jsonb_typeof(p_events)<>'array' or jsonb_array_length(p_events)>100 then
    raise exception 'invalid sync batch' using errcode='22023';
  end if;

  for item in select value from jsonb_array_elements(p_events) loop
    begin
      event_uuid := (item->>'eventId')::uuid;
      branch_uuid := nullif(item->>'branchId','')::uuid;
      causation_uuid := nullif(item->>'causationId','')::uuid;
    exception when invalid_text_representation then
      raise exception 'invalid sync event identifier' using errcode='22023';
    end;
    if (item->>'organizationId')::uuid<>org
       or (item->>'deviceId')::uuid<>p_device_id
       or coalesce((item->>'localSequence')::bigint,0)<=0
       or coalesce((item->>'schemaVersion')::integer,0)<=0
       or coalesce(item->>'payloadHash','') !~ '^[0-9a-f]{64}$'
       or jsonb_typeof(item->'payload') is null
       or coalesce(length(item->>'entityType'),0) not between 1 and 80
       or coalesce(length(item->>'entityId'),0) not between 1 and 160
       or coalesce(item->>'operation','') not in ('CREATED','UPDATED','APPENDED','REVOKED') then
      raise exception 'invalid sync event envelope' using errcode='22023';
    end if;
    if branch_uuid is null then
      if not app.is_org_admin(org) then
        raise exception 'organization event requires administrator membership' using errcode='42501';
      end if;
    elsif not app.is_branch_member(org,branch_uuid) then
      raise exception 'branch access denied' using errcode='42501';
    end if;

    select * into existing from app.sync_events
     where organization_id=org and event_id=event_uuid;
    was_duplicate := found;
    if found then
      if existing.device_id<>p_device_id
         or existing.branch_id is distinct from branch_uuid
         or existing.entity_type<>item->>'entityType'
         or existing.entity_id<>item->>'entityId'
         or existing.operation<>item->>'operation'
         or existing.payload is distinct from item->'payload'
         or existing.payload_hash<>item->>'payloadHash'
         or existing.schema_version<>(item->>'schemaVersion')::integer
         or existing.causation_id is distinct from causation_uuid
         or existing.occurred_at<>(item->>'occurredAt')::timestamptz
         or existing.local_sequence<>(item->>'localSequence')::bigint then
        raise exception 'event identity reused with different content' using errcode='23505';
      end if;
      inserted_sequence := existing.server_sequence;
    else
      insert into app.sync_events(
        organization_id,event_id,branch_id,device_id,actor_user_id,
        entity_type,entity_id,operation,payload,payload_hash,schema_version,
        causation_id,occurred_at,local_sequence
      ) values (
        org,event_uuid,branch_uuid,p_device_id,actor,
        item->>'entityType',item->>'entityId',item->>'operation',item->'payload',
        item->>'payloadHash',(item->>'schemaVersion')::integer,causation_uuid,
        (item->>'occurredAt')::timestamptz,(item->>'localSequence')::bigint
      ) returning server_sequence into inserted_sequence;
    end if;
    result := result || jsonb_build_array(jsonb_build_object(
      'eventId',event_uuid,
      'serverSequence',inserted_sequence,
      'duplicate',was_duplicate
    ));
  end loop;
  return result;
end $$;

create or replace function public.pull_sync_events(
  p_device_id uuid,
  p_after_sequence bigint,
  p_limit integer
) returns jsonb
language plpgsql
security definer
set search_path=pg_catalog,app,auth,public
as $$
declare
  actor uuid := auth.uid();
  org uuid;
  result jsonb;
begin
  if actor is null then raise exception 'authentication required' using errcode='42501'; end if;
  select organization_id into org from app.devices
   where id=p_device_id and owner_user_id=actor and status='ENROLLED';
  if org is null or not app.is_active_org_member(org) then
    raise exception 'active enrolled device required' using errcode='42501';
  end if;
  if p_after_sequence<0 or p_limit not between 1 and 200 then
    raise exception 'invalid pull window' using errcode='22023';
  end if;
  select coalesce(jsonb_agg(jsonb_build_object(
    'organizationId',e.organization_id,'eventId',e.event_id,
    'branchId',e.branch_id,'deviceId',e.device_id,'actorUserId',e.actor_user_id,
    'entityType',e.entity_type,'entityId',e.entity_id,'operation',e.operation,
    'payload',e.payload,'payloadHash',e.payload_hash,'schemaVersion',e.schema_version,
    'causationId',e.causation_id,'occurredAt',e.occurred_at,
    'serverSequence',e.server_sequence
  ) order by e.server_sequence),'[]'::jsonb) into result
  from (
    select * from app.sync_events e
    where e.organization_id=org and e.server_sequence>p_after_sequence
      and ((e.branch_id is null and app.is_org_admin(org))
        or (e.branch_id is not null and app.is_branch_member(org,e.branch_id)))
    order by e.server_sequence limit p_limit
  ) e;
  return result;
end $$;

create or replace function public.ack_sync_events(
  p_device_id uuid,
  p_receipts jsonb
) returns jsonb
language plpgsql
security definer
set search_path=pg_catalog,app,auth,public
as $$
declare
  actor uuid := auth.uid();
  org uuid;
  item jsonb;
  event_uuid uuid;
  sequence bigint;
  affected integer := 0;
begin
  if actor is null then raise exception 'authentication required' using errcode='42501'; end if;
  select organization_id into org from app.devices
   where id=p_device_id and owner_user_id=actor and status='ENROLLED';
  if org is null or not app.is_active_org_member(org) then
    raise exception 'active enrolled device required' using errcode='42501';
  end if;
  if jsonb_typeof(p_receipts)<>'array' or jsonb_array_length(p_receipts)>100 then
    raise exception 'invalid receipt batch' using errcode='22023';
  end if;
  for item in select value from jsonb_array_elements(p_receipts) loop
    event_uuid := (item->>'eventId')::uuid;
    sequence := (item->>'serverSequence')::bigint;
    if not exists(select 1 from app.sync_events e where e.organization_id=org
      and e.event_id=event_uuid and e.server_sequence=sequence and e.device_id=p_device_id) then
      raise exception 'receipt does not match a device event' using errcode='22023';
    end if;
    insert into app.sync_receipts(organization_id,event_id,device_id,acknowledged_sequence)
    values(org,event_uuid,p_device_id,sequence)
    on conflict(organization_id,event_id,device_id) do update
      set acknowledged_sequence=excluded.acknowledged_sequence,
          acknowledged_at=now();
    affected := affected+1;
  end loop;
  return jsonb_build_object('acknowledged',affected);
end $$;

revoke all on function public.enroll_sync_device(uuid,uuid,text) from public,anon;
revoke all on function public.push_sync_events(uuid,jsonb) from public,anon;
revoke all on function public.pull_sync_events(uuid,bigint,integer) from public,anon;
revoke all on function public.ack_sync_events(uuid,jsonb) from public,anon;
grant execute on function public.enroll_sync_device(uuid,uuid,text) to authenticated;
grant execute on function public.push_sync_events(uuid,jsonb) to authenticated;
grant execute on function public.pull_sync_events(uuid,bigint,integer) to authenticated;
grant execute on function public.ack_sync_events(uuid,jsonb) to authenticated;

commit;
