-- Phase 9A.2 account/trial/device/RLS verification. All fixtures roll back.
begin;

insert into auth.users(id,email,raw_user_meta_data) values
 ('91000000-0000-4000-8000-000000000001','phase9a2-a@example.invalid','{"full_name":"Portal A"}'),
 ('91000000-0000-4000-8000-000000000002','phase9a2-b@example.invalid','{"full_name":"Portal B"}');

set local role authenticated;
set local request.jwt.claims='{"sub":"91000000-0000-4000-8000-000000000001","role":"authenticated"}';

do $$ declare first_trial jsonb; claims jsonb; begin
  if (public.get_portal_snapshot()->>'status')<>'UNACTIVATED' then raise exception 'new account was not unactivated'; end if;
  first_trial:=public.start_early_access_trial();
  if first_trial->>'status'<>'TRIAL' then raise exception 'eligible account did not start trial'; end if;
  begin
    perform public.start_early_access_trial();
    raise exception 'second trial unexpectedly succeeded';
  exception when unique_violation then null;
  end;
  claims:=public.prepare_license_claims('93000000-0000-4000-8000-000000000001','Counter A','0.4.0-beta.1','20261020.1');
  if claims->>'status'<>'TRIAL' or claims->'claims'->>'state'<>'ACTIVE' then raise exception 'trial license claims were not prepared'; end if;
  if claims->'claims'->>'deviceId'<>'93000000-0000-4000-8000-000000000001' then raise exception 'claims were not device-bound'; end if;
  if (select count(*) from app.license_accounts)<>1 or (select count(*) from app.license_devices)<>1 then raise exception 'RLS exposed another account'; end if;
  begin
    update app.license_accounts set status='ACTIVE';
    raise exception 'direct license mutation unexpectedly succeeded';
  exception when insufficient_privilege then null;
  end;
  begin
    perform public.admin_list_activation_requests();
    raise exception 'authenticated user reached admin RPC';
  exception when insufficient_privilege then null;
  end;
end $$;

set local request.jwt.claims='{"sub":"91000000-0000-4000-8000-000000000002","role":"authenticated"}';
select public.start_early_access_trial();

do $$ begin
  begin
    perform public.prepare_license_claims('93000000-0000-4000-8000-000000000001','Stolen device','0.4.0-beta.1','20261020.1');
    raise exception 'cross-account device claim unexpectedly succeeded';
  exception when insufficient_privilege then null;
  end;
  begin
    perform public.revoke_license_device('93000000-0000-4000-8000-000000000001');
    raise exception 'cross-account revoke unexpectedly succeeded';
  exception when insufficient_privilege then null;
  end;
end $$;

select public.prepare_license_claims('93000000-0000-4000-8000-000000000002','Counter B','0.4.0-beta.1','20261020.1');
select public.revoke_license_device('93000000-0000-4000-8000-000000000002');

do $$ declare revoked jsonb; begin
  revoked:=public.prepare_license_claims('93000000-0000-4000-8000-000000000002','Counter B','0.4.0-beta.1','20261020.1');
  if revoked->>'status'<>'REVOKED' or revoked->'claims'->>'state'<>'REVOKED' then raise exception 'revocation did not propagate to validation'; end if;
  if has_function_privilege('authenticated','public.rls_auto_enable()','EXECUTE') then raise exception 'rls_auto_enable revoke regressed'; end if;
end $$;

reset role;
set local role service_role;
set local request.jwt.claims='{"role":"service_role"}';

do $$ declare target uuid; result jsonb; begin
  select id into target from app.license_accounts where owner_user_id='91000000-0000-4000-8000-000000000001';
  result:=public.admin_set_license_state(target,'ACTIVE','SINGLE_PHARMACY',now(),now()+interval '30 days',2,'Synthetic Phase 9A.2 activation');
  if result->>'status'<>'ACTIVE' then raise exception 'server admin activation failed'; end if;
end $$;

rollback;
