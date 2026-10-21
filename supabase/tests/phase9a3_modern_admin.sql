-- Phase 9A.3 modern-secret admin authorization verification. All fixtures roll back.
begin;

do $$ begin
  if has_function_privilege('anon','public.admin_list_activation_requests()','EXECUTE')
    or has_function_privilege('authenticated','public.admin_list_activation_requests()','EXECUTE')
    or not has_function_privilege('service_role','public.admin_list_activation_requests()','EXECUTE') then
    raise exception 'admin list RPC grants are unsafe';
  end if;
  if has_function_privilege('anon','public.admin_set_license_state(uuid,text,text,timestamptz,timestamptz,integer,text)','EXECUTE')
    or has_function_privilege('authenticated','public.admin_set_license_state(uuid,text,text,timestamptz,timestamptz,integer,text)','EXECUTE')
    or not has_function_privilege('service_role','public.admin_set_license_state(uuid,text,text,timestamptz,timestamptz,integer,text)','EXECUTE') then
    raise exception 'admin state RPC grants are unsafe';
  end if;
end $$;

insert into auth.users(id,email,raw_user_meta_data) values
 ('94000000-0000-4000-8000-000000000001','phase9a3-a@example.invalid','{"full_name":"Portal A"}'),
 ('94000000-0000-4000-8000-000000000002','phase9a3-b@example.invalid','{"full_name":"Portal B"}');

set local role authenticated;
set local request.jwt.claims='{"sub":"94000000-0000-4000-8000-000000000001","role":"authenticated"}';
select public.start_early_access_trial();
select public.prepare_license_claims('94000000-0000-4000-8000-000000000001','Counter A','0.4.0-beta.1','20261020.1');

do $$ begin
  begin
    perform public.admin_list_activation_requests();
    raise exception 'authenticated user reached admin list RPC';
  exception when insufficient_privilege then null;
  end;
  begin
    perform public.admin_set_license_state('00000000-0000-4000-8000-000000000000','ACTIVE','SINGLE_PHARMACY',now(),now()+interval '30 days',1,'Denied authenticated attempt');
    raise exception 'authenticated user reached admin state RPC';
  exception when insufficient_privilege then null;
  end;
end $$;

set local request.jwt.claims='{"sub":"94000000-0000-4000-8000-000000000002","role":"authenticated"}';
select public.start_early_access_trial();

do $$ begin
  begin
    perform public.prepare_license_claims('94000000-0000-4000-8000-000000000001','Cross-account device','0.4.0-beta.1','20261020.1');
    raise exception 'cross-account device claim unexpectedly succeeded';
  exception when insufficient_privilege then null;
  end;
  begin
    perform public.revoke_license_device('94000000-0000-4000-8000-000000000001');
    raise exception 'cross-account revoke unexpectedly succeeded';
  exception when insufficient_privilege then null;
  end;
end $$;

reset role;
do $$ begin
  perform set_config('m2.test_account_id',(select id::text from app.license_accounts where owner_user_id='94000000-0000-4000-8000-000000000001'),true);
end $$;
set local role anon;
set local request.jwt.claims='{"role":"anon"}';

do $$ begin
  begin
    perform public.admin_list_activation_requests();
    raise exception 'anonymous caller reached admin list RPC';
  exception when insufficient_privilege then null;
  end;
  begin
    perform public.admin_set_license_state('00000000-0000-4000-8000-000000000000','ACTIVE','SINGLE_PHARMACY',now(),now()+interval '30 days',1,'Denied anonymous attempt');
    raise exception 'anonymous caller reached admin state RPC';
  exception when insufficient_privilege then null;
  end;
end $$;

reset role;
set local role service_role;
-- Modern sb_secret_ requests do not supply a caller JWT. This deliberately
-- proves the approved database role works with no legacy role claim.
set local request.jwt.claims='{}';

do $$ declare target uuid; result jsonb; begin
  perform public.admin_list_activation_requests();
  target:=current_setting('m2.test_account_id')::uuid;
  result:=public.admin_set_license_state(target,'ACTIVE','SINGLE_PHARMACY',now(),now()+interval '30 days',2,'Synthetic Phase 9A.3 activation');
  if result->>'status'<>'ACTIVE' then raise exception 'modern server admin activation failed'; end if;
end $$;

rollback;
