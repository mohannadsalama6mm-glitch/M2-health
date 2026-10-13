-- Runtime Phase 8A/8B security verification. Every fixture is rolled back.
begin;

insert into auth.users(id) values
 ('10000000-0000-4000-8000-000000000001'),
 ('10000000-0000-4000-8000-000000000002'),
 ('10000000-0000-4000-8000-000000000003');
insert into app.organizations(id,slug,name) values
 ('20000000-0000-4000-8000-000000000001','phase8b-a','Phase 8B A'),
 ('20000000-0000-4000-8000-000000000002','phase8b-b','Phase 8B B');
insert into app.organization_memberships(organization_id,user_id,role_code) values
 ('20000000-0000-4000-8000-000000000001','10000000-0000-4000-8000-000000000001','OWNER'),
 ('20000000-0000-4000-8000-000000000001','10000000-0000-4000-8000-000000000002','MEMBER'),
 ('20000000-0000-4000-8000-000000000002','10000000-0000-4000-8000-000000000003','OWNER');
insert into app.branches(id,organization_id,code,name) values
 ('30000000-0000-4000-8000-000000000001','20000000-0000-4000-8000-000000000001','A1','A One'),
 ('30000000-0000-4000-8000-000000000002','20000000-0000-4000-8000-000000000001','A2','A Two'),
 ('30000000-0000-4000-8000-000000000003','20000000-0000-4000-8000-000000000002','B1','B One');
insert into app.branch_memberships(organization_id,branch_id,user_id,granted_by) values
 ('20000000-0000-4000-8000-000000000001','30000000-0000-4000-8000-000000000001','10000000-0000-4000-8000-000000000002','10000000-0000-4000-8000-000000000001');

set local role authenticated;
set local request.jwt.claims='{"sub":"10000000-0000-4000-8000-000000000002","role":"authenticated"}';

do $$ begin
  if (select count(*) from app.organizations)<>1 then
    raise exception 'tenant RLS leaked an organization';
  end if;
  if (select count(*) from app.branches)<>1 then
    raise exception 'branch RLS leaked an unauthorized branch';
  end if;
  begin
    insert into app.organizations(slug,name) values('forbidden','Forbidden');
    raise exception 'direct authenticated table mutation unexpectedly succeeded';
  exception when insufficient_privilege then null;
  end;
end $$;

select public.enroll_sync_device(
 '20000000-0000-4000-8000-000000000001',
 '40000000-0000-4000-8000-000000000001',
 'Phase 8B isolated device'
);

do $$ declare first_result jsonb; duplicate_result jsonb; pulled jsonb; begin
  first_result := public.push_sync_events(
    '40000000-0000-4000-8000-000000000001',
    '[{"eventId":"50000000-0000-4000-8000-000000000001","organizationId":"20000000-0000-4000-8000-000000000001","branchId":"30000000-0000-4000-8000-000000000001","deviceId":"40000000-0000-4000-8000-000000000001","actorUserId":"ffffffff-ffff-4fff-8fff-ffffffffffff","entityType":"inventory_movement","entityId":"movement-1","operation":"APPENDED","payload":{"quantityDelta":2},"payloadHash":"828787e6d42ac77be43e55c886004ea9c1a1a2a5e1da3f2ab8ee2e402fc7e9a2","schemaVersion":1,"causationId":null,"occurredAt":"2026-10-10T00:00:00Z","localSequence":1}]'::jsonb
  );
  if jsonb_array_length(first_result)<>1 or (first_result->0->>'duplicate')::boolean then
    raise exception 'first push did not create exactly one event';
  end if;
  duplicate_result := public.push_sync_events(
    '40000000-0000-4000-8000-000000000001',
    '[{"eventId":"50000000-0000-4000-8000-000000000001","organizationId":"20000000-0000-4000-8000-000000000001","branchId":"30000000-0000-4000-8000-000000000001","deviceId":"40000000-0000-4000-8000-000000000001","actorUserId":"another-local-id","entityType":"inventory_movement","entityId":"movement-1","operation":"APPENDED","payload":{"quantityDelta":2},"payloadHash":"828787e6d42ac77be43e55c886004ea9c1a1a2a5e1da3f2ab8ee2e402fc7e9a2","schemaVersion":1,"causationId":null,"occurredAt":"2026-10-10T00:00:00Z","localSequence":1}]'::jsonb
  );
  if not (duplicate_result->0->>'duplicate')::boolean then
    raise exception 'idempotent replay was not marked duplicate';
  end if;
  if (select actor_user_id from app.sync_events where event_id='50000000-0000-4000-8000-000000000001')
       <>'10000000-0000-4000-8000-000000000002'::uuid then
    raise exception 'server trusted the client actor';
  end if;
  pulled := public.pull_sync_events('40000000-0000-4000-8000-000000000001',0,200);
  if jsonb_array_length(pulled)<>1 then raise exception 'authorized pull failed'; end if;
  perform public.ack_sync_events(
    '40000000-0000-4000-8000-000000000001',
    jsonb_build_array(jsonb_build_object(
      'eventId','50000000-0000-4000-8000-000000000001',
      'serverSequence',(first_result->0->>'serverSequence')::bigint,
      'duplicate',false
    ))
  );
  if (select count(*) from app.sync_receipts)<>1 then raise exception 'receipt was not durable'; end if;
  begin
    perform public.push_sync_events(
      '40000000-0000-4000-8000-000000000001',
      '[{"eventId":"50000000-0000-4000-8000-000000000001","organizationId":"20000000-0000-4000-8000-000000000001","branchId":"30000000-0000-4000-8000-000000000001","deviceId":"40000000-0000-4000-8000-000000000001","actorUserId":"ignored","entityType":"inventory_movement","entityId":"movement-1","operation":"APPENDED","payload":{"quantityDelta":99},"payloadHash":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","schemaVersion":1,"causationId":null,"occurredAt":"2026-10-10T00:00:00Z","localSequence":1}]'::jsonb
    );
    raise exception 'conflicting event replay unexpectedly succeeded';
  exception when unique_violation then null;
  end;
  begin
    perform public.push_sync_events(
      '40000000-0000-4000-8000-000000000001',
      '[{"eventId":"50000000-0000-4000-8000-000000000002","organizationId":"20000000-0000-4000-8000-000000000001","branchId":"30000000-0000-4000-8000-000000000002","deviceId":"40000000-0000-4000-8000-000000000001","actorUserId":"ignored","entityType":"inventory_movement","entityId":"movement-2","operation":"APPENDED","payload":{},"payloadHash":"cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc","schemaVersion":1,"causationId":null,"occurredAt":"2026-10-10T00:00:00Z","localSequence":2}]'::jsonb
    );
    raise exception 'cross-branch push unexpectedly succeeded';
  exception when insufficient_privilege then null;
  end;
end $$;

reset role;
update app.organization_memberships set status='REVOKED',revoked_at=now()
 where organization_id='20000000-0000-4000-8000-000000000001'
   and user_id='10000000-0000-4000-8000-000000000002';
set local role authenticated;
do $$ begin
  begin
    perform public.pull_sync_events('40000000-0000-4000-8000-000000000001',0,200);
    raise exception 'revoked member retained sync access';
  exception when insufficient_privilege then null;
  end;
end $$;

rollback;
