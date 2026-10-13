-- Phase 8A cloud control-plane schema. Deploy only after Supabase Auth exists.
-- Direct client mutations are intentionally unavailable until the trusted
-- enrollment and synchronization service is implemented in Phase 8B.
begin;

create schema if not exists app;

create table app.organizations (
  id uuid primary key default gen_random_uuid(),
  slug text not null unique check (length(btrim(slug)) between 3 and 80),
  name text not null check (length(btrim(name)) between 1 and 160),
  status text not null default 'ACTIVE' check (status in ('ACTIVE','SUSPENDED')),
  created_at timestamptz not null default now()
);

create table app.organization_memberships (
  organization_id uuid not null references app.organizations(id) on delete restrict,
  user_id uuid not null references auth.users(id) on delete restrict,
  role_code text not null check (role_code in ('OWNER','ADMIN','MEMBER')),
  status text not null default 'ACTIVE' check (status in ('ACTIVE','REVOKED')),
  granted_by uuid references auth.users(id) on delete restrict,
  granted_at timestamptz not null default now(),
  revoked_at timestamptz,
  primary key (organization_id,user_id),
  check ((status='ACTIVE' and revoked_at is null) or (status='REVOKED' and revoked_at is not null))
);
create index organization_memberships_user on app.organization_memberships(user_id,organization_id) where status='ACTIVE';

create table app.branches (
  id uuid primary key,
  organization_id uuid not null references app.organizations(id) on delete restrict,
  code text not null check (length(btrim(code)) between 1 and 32),
  name text not null check (length(btrim(name)) between 1 and 160),
  status text not null default 'ACTIVE' check (status in ('ACTIVE','DISABLED')),
  created_at timestamptz not null default now(),
  unique (organization_id,id),
  unique (organization_id,code)
);

create table app.branch_memberships (
  organization_id uuid not null,
  branch_id uuid not null,
  user_id uuid not null,
  granted_by uuid not null references auth.users(id) on delete restrict,
  granted_at timestamptz not null default now(),
  primary key (organization_id,branch_id,user_id),
  foreign key (organization_id,branch_id) references app.branches(organization_id,id) on delete restrict,
  foreign key (organization_id,user_id) references app.organization_memberships(organization_id,user_id) on delete restrict
);
create index branch_memberships_user on app.branch_memberships(user_id,organization_id,branch_id);

create table app.local_identity_links (
  organization_id uuid not null,
  cloud_user_id uuid not null references auth.users(id) on delete restrict,
  local_user_id uuid not null,
  device_id uuid not null,
  status text not null check (status in ('ACTIVE','REVOKED')),
  linked_at timestamptz not null default now(),
  revoked_at timestamptz,
  primary key (organization_id,local_user_id,device_id),
  unique (organization_id,cloud_user_id,device_id),
  foreign key (organization_id,cloud_user_id) references app.organization_memberships(organization_id,user_id) on delete restrict,
  check ((status='ACTIVE' and revoked_at is null) or (status='REVOKED' and revoked_at is not null))
);

create table app.devices (
  id uuid primary key,
  organization_id uuid not null references app.organizations(id) on delete restrict,
  owner_user_id uuid not null references auth.users(id) on delete restrict,
  display_name text not null check (length(btrim(display_name)) between 1 and 160),
  status text not null check (status in ('ENROLLED','REVOKED')),
  enrolled_at timestamptz not null default now(),
  revoked_at timestamptz,
  unique (organization_id,id),
  foreign key (organization_id,owner_user_id) references app.organization_memberships(organization_id,user_id) on delete restrict,
  check ((status='ENROLLED' and revoked_at is null) or (status='REVOKED' and revoked_at is not null))
);
create index devices_owner on app.devices(owner_user_id,organization_id) where status='ENROLLED';

create table app.sync_events (
  organization_id uuid not null,
  event_id uuid not null,
  branch_id uuid,
  device_id uuid not null,
  actor_user_id uuid not null,
  entity_type text not null check (length(entity_type) between 1 and 80),
  entity_id text not null check (length(entity_id) between 1 and 160),
  operation text not null check (operation in ('CREATED','UPDATED','APPENDED','REVOKED')),
  payload jsonb not null,
  schema_version integer not null check (schema_version>0),
  causation_id uuid,
  occurred_at timestamptz not null,
  server_sequence bigint generated always as identity,
  received_at timestamptz not null default now(),
  primary key (organization_id,event_id),
  unique (organization_id,server_sequence),
  foreign key (organization_id,branch_id) references app.branches(organization_id,id) on delete restrict,
  foreign key (organization_id,device_id) references app.devices(organization_id,id) on delete restrict,
  foreign key (organization_id,actor_user_id) references app.organization_memberships(organization_id,user_id) on delete restrict
);
create index sync_events_pull on app.sync_events(organization_id,server_sequence);
create index sync_events_branch on app.sync_events(organization_id,branch_id,server_sequence) where branch_id is not null;

create table app.sync_receipts (
  organization_id uuid not null,
  event_id uuid not null,
  device_id uuid not null,
  acknowledged_sequence bigint not null check (acknowledged_sequence>=0),
  acknowledged_at timestamptz not null default now(),
  primary key (organization_id,event_id,device_id),
  foreign key (organization_id,event_id) references app.sync_events(organization_id,event_id) on delete restrict,
  foreign key (organization_id,device_id) references app.devices(organization_id,id) on delete restrict
);

create or replace function app.is_active_org_member(target_org uuid)
returns boolean language sql stable security definer set search_path=app,pg_catalog as $$
  select exists(select 1 from app.organization_memberships m
    where m.organization_id=target_org and m.user_id=auth.uid() and m.status='ACTIVE');
$$;
create or replace function app.is_org_admin(target_org uuid)
returns boolean language sql stable security definer set search_path=app,pg_catalog as $$
  select exists(select 1 from app.organization_memberships m
    where m.organization_id=target_org and m.user_id=auth.uid() and m.status='ACTIVE'
      and m.role_code in ('OWNER','ADMIN'));
$$;
create or replace function app.is_branch_member(target_org uuid,target_branch uuid)
returns boolean language sql stable security definer set search_path=app,pg_catalog as $$
  select app.is_org_admin(target_org) or exists(select 1 from app.branch_memberships b
    where b.organization_id=target_org and b.branch_id=target_branch and b.user_id=auth.uid());
$$;
create or replace function app.owns_active_device(target_org uuid,target_device uuid)
returns boolean language sql stable security definer set search_path=app,pg_catalog as $$
  select exists(select 1 from app.devices d
    where d.organization_id=target_org and d.id=target_device and d.owner_user_id=auth.uid() and d.status='ENROLLED');
$$;

alter table app.organizations enable row level security;
alter table app.organizations force row level security;
alter table app.organization_memberships enable row level security;
alter table app.organization_memberships force row level security;
alter table app.branches enable row level security;
alter table app.branches force row level security;
alter table app.branch_memberships enable row level security;
alter table app.branch_memberships force row level security;
alter table app.local_identity_links enable row level security;
alter table app.local_identity_links force row level security;
alter table app.devices enable row level security;
alter table app.devices force row level security;
alter table app.sync_events enable row level security;
alter table app.sync_events force row level security;
alter table app.sync_receipts enable row level security;
alter table app.sync_receipts force row level security;

create policy organizations_member_read on app.organizations for select to authenticated
  using (app.is_active_org_member(id));
create policy memberships_self_or_admin_read on app.organization_memberships for select to authenticated
  using (user_id=auth.uid() or app.is_org_admin(organization_id));
create policy branches_member_read on app.branches for select to authenticated
  using (app.is_branch_member(organization_id,id));
create policy branch_memberships_self_or_admin_read on app.branch_memberships for select to authenticated
  using (user_id=auth.uid() or app.is_org_admin(organization_id));
create policy identity_links_owner_or_admin_read on app.local_identity_links for select to authenticated
  using (cloud_user_id=auth.uid() or app.is_org_admin(organization_id));
create policy devices_owner_or_admin_read on app.devices for select to authenticated
  using (owner_user_id=auth.uid() or app.is_org_admin(organization_id));
create policy sync_events_tenant_branch_read on app.sync_events for select to authenticated
  using ((branch_id is null and app.is_org_admin(organization_id))
    or (branch_id is not null and app.is_branch_member(organization_id,branch_id)));
create policy sync_receipts_device_owner_read on app.sync_receipts for select to authenticated
  using (app.owns_active_device(organization_id,device_id) or app.is_org_admin(organization_id));

revoke all on all tables in schema app from anon, authenticated;
grant usage on schema app to authenticated;
grant select on app.organizations,app.organization_memberships,app.branches,
  app.branch_memberships,app.local_identity_links,app.devices,
  app.sync_events,app.sync_receipts to authenticated;
revoke all on all functions in schema app from public, anon;
grant execute on function app.is_active_org_member(uuid),app.is_org_admin(uuid),
  app.is_branch_member(uuid,uuid),app.owns_active_device(uuid,uuid) to authenticated;

commit;
