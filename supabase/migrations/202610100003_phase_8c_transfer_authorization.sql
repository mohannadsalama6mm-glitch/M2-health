-- Phase 8C paired transfer authorization. A transfer event is visible and
-- writable only when the authenticated device owner belongs to both branches.
begin;

create or replace function app.verify_transfer_event_scope()
returns trigger language plpgsql security definer
set search_path=pg_catalog,app,auth
as $$
declare destination uuid;
begin
  if new.entity_type='inventory_transfer' then
    begin destination := (new.payload->>'destinationBranchId')::uuid;
    exception when invalid_text_representation then
      raise exception 'invalid transfer destination' using errcode='22023';
    end;
    if new.branch_id is null or destination is null or destination=new.branch_id
       or not app.is_branch_member(new.organization_id,new.branch_id)
       or not app.is_branch_member(new.organization_id,destination) then
      raise exception 'both transfer branches are required' using errcode='42501';
    end if;
  end if;
  return new;
end $$;

create trigger sync_transfer_scope before insert on app.sync_events
for each row execute function app.verify_transfer_event_scope();

create or replace function public.pull_sync_events(
  p_device_id uuid,p_after_sequence bigint,p_limit integer
) returns jsonb
language plpgsql security definer
set search_path=pg_catalog,app,auth,public
as $$
declare actor uuid:=auth.uid(); org uuid; result jsonb;
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
        or (e.branch_id is not null and app.is_branch_member(org,e.branch_id)
          and (e.entity_type<>'inventory_transfer'
            or app.is_branch_member(org,(e.payload->>'destinationBranchId')::uuid))))
    order by e.server_sequence limit p_limit
  ) e;
  return result;
end $$;

revoke all on function app.verify_transfer_event_scope() from public,anon,authenticated;

commit;
