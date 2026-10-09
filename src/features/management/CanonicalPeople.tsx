import { useCallback, useEffect, useState, type FormEvent } from "react";
import { Link, useParams } from "react-router-dom";
import { Alert, BackLink, Badge, Button, Card, EmptyState, FeaturePage, Input, Select, Status } from "../../design-system";
import { useAuth } from "../../app/AuthContext";
import { useBranch } from "../../app/BranchContext";
import * as api from "../../lib/tauri/auth";

const errorText = (error: unknown) => error instanceof Error ? error.message : "The request could not be completed.";
const roleNames = (user: api.User) => user.roles.map((role) => role.name).join(", ") || "No roles";

export function CanonicalEmployees() {
  const { token } = useAuth();
  const [page, setPage] = useState<api.UserPage | null>(null);
  const [roles, setRoles] = useState<api.Role[]>([]);
  const [search, setSearch] = useState("");
  const [roleCode, setRoleCode] = useState("");
  const [active, setActive] = useState("");
  const [offset, setOffset] = useState(0);
  const [error, setError] = useState("");
  const [creating, setCreating] = useState(false);
  const load = useCallback(async () => {
    try {
      const [users, availableRoles] = await Promise.all([
        api.listUsers(token, { search: search || undefined, roleCode: roleCode || undefined, isActive: active === "" ? undefined : active === "active", limit: 20, offset }),
        api.listRoles(token),
      ]);
      setPage(users); setRoles(availableRoles); setError("");
    } catch (e) { setError(errorText(e)); }
  }, [token, search, roleCode, active, offset]);
  useEffect(() => {
    const timer = window.setTimeout(() => { void load(); }, 0);
    return () => window.clearTimeout(timer);
  }, [load]);
  return <FeaturePage title="Users" description="Local accounts, role assignments, and branch memberships." actions={<><Link className="button secondary" to="/roles">Roles</Link><Button variant="primary" onClick={() => setCreating((value) => !value)}>{creating ? "Cancel" : "Add user"}</Button></>}>
    {error && <Alert title="Users unavailable" tone="danger">{error}</Alert>}
    {creating && <CreateUser roles={roles} onCreated={() => { setCreating(false); void load(); }} />}
    <Card>
      <div className="identity-filters">
        <Input label="Search users" value={search} placeholder="Name or username" onChange={(e) => { setSearch(e.target.value); setOffset(0); }} />
        <Select showLabel label="Role" value={roleCode} onChange={(e) => { setRoleCode(e.target.value); setOffset(0); }}><option value="">All roles</option>{roles.map((role) => <option key={role.code} value={role.code}>{role.name}</option>)}</Select>
        <Select showLabel label="Status" value={active} onChange={(e) => { setActive(e.target.value); setOffset(0); }}><option value="">All statuses</option><option value="active">Active</option><option value="inactive">Inactive</option></Select>
      </div>
      {!page ? <p role="status">Loading users…</p> : page.items.length === 0 ? <EmptyState title="No users found" description="Change the filters or create a local user." /> : <div className="table-scroll"><table><thead><tr><th>User</th><th>Roles</th><th>Branches</th><th>Status</th><th>Updated</th></tr></thead><tbody>{page.items.map((user) => <tr key={user.id}><td><Link className="text-link" to={`/employees/${user.id}`}><strong>{user.displayName}</strong><span className="cell-secondary">@{user.username}</span></Link></td><td>{roleNames(user)}</td><td>{user.branches.map((branch) => branch.branchCode).join(", ") || "All / unscoped"}</td><td><Status value={user.isActive ? "Active" : "Inactive"} /></td><td>{new Date(user.updatedAt).toLocaleDateString()}</td></tr>)}</tbody></table></div>}
      {page && <div className="identity-pagination"><span>{page.total} user{page.total === 1 ? "" : "s"}</span><div><Button variant="secondary" disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - 20))}>Previous</Button><Button variant="secondary" disabled={offset + 20 >= page.total} onClick={() => setOffset(offset + 20)}>Next</Button></div></div>}
    </Card>
  </FeaturePage>;
}

function CreateUser({ roles, onCreated }: { roles: api.Role[]; onCreated(): void }) {
  const { token } = useAuth();
  const { branches } = useBranch();
  const [form, setForm] = useState({ username: "", displayName: "", password: "", roleCode: "PHARMACIST", branchId: "" });
  const [error, setError] = useState(""); const [busy, setBusy] = useState(false);
  const submit = async (event: FormEvent) => { event.preventDefault(); setBusy(true); setError(""); try { await api.createUser(token, { username: form.username, displayName: form.displayName, password: form.password, roleCodes: form.roleCode ? [form.roleCode] : [], branchIds: form.branchId ? [form.branchId] : [], primaryBranchId: form.branchId || null }); onCreated(); } catch (e) { setError(errorText(e)); } finally { setBusy(false); } };
  return <Card><form className="identity-form" onSubmit={submit}><h2>Create local user</h2>{error && <Alert title="User not created" tone="danger">{error}</Alert>}<Input label="Display name" required value={form.displayName} onChange={(e) => setForm({ ...form, displayName: e.target.value })} /><Input label="Username" required value={form.username} onChange={(e) => setForm({ ...form, username: e.target.value })} /><Input label="Temporary password" type="password" required minLength={12} value={form.password} onChange={(e) => setForm({ ...form, password: e.target.value })} /><Select showLabel label="Initial role" value={form.roleCode} onChange={(e) => setForm({ ...form, roleCode: e.target.value })}><option value="">No role</option>{roles.map((role) => <option key={role.code} value={role.code}>{role.name}</option>)}</Select><Select showLabel label="Primary branch" value={form.branchId} onChange={(e) => setForm({ ...form, branchId: e.target.value })}><option value="">No branch restriction</option>{branches.filter((branch) => branch.isActive).map((branch) => <option key={branch.id} value={branch.id}>{branch.name}</option>)}</Select><Button variant="primary" type="submit" disabled={busy}>{busy ? "Creating…" : "Create user"}</Button></form></Card>;
}

export function CanonicalEmployeeDetails() {
  const { id = "" } = useParams(); const { token, user: signedIn } = useAuth(); const { branches } = useBranch();
  const [user, setUser] = useState<api.User | null>(null); const [roles, setRoles] = useState<api.Role[]>([]); const [error, setError] = useState(""); const [password, setPassword] = useState("");
  const [profile, setProfile] = useState({ username: "", displayName: "" });
  const load = useCallback(async () => { try { const [next, allRoles] = await Promise.all([api.getUser(token, id), api.listRoles(token)]); setUser(next); setProfile({ username: next.username, displayName: next.displayName }); setRoles(allRoles); setError(""); } catch (e) { setError(errorText(e)); } }, [token, id]);
  useEffect(() => {
    const timer = window.setTimeout(() => { void load(); }, 0);
    return () => window.clearTimeout(timer);
  }, [load]);
  const run = async (action: () => Promise<unknown>) => { try { await action(); await load(); } catch (e) { setError(errorText(e)); } };
  if (!user && !error) return <p role="status">Loading user…</p>;
  if (!user) return <><BackLink to="/employees" label="Users" /><EmptyState title="User unavailable" description={error} /></>;
  return <><BackLink to="/employees" label="Users" /><FeaturePage title={user.displayName} description={`@${user.username}`} actions={<Button variant={user.isActive ? "danger" : "primary"} disabled={signedIn?.id === user.id && user.isActive} onClick={() => void run(() => api.setUserActive(token, { id: user.id, expectedRevision: user.revision, active: !user.isActive }))}>{user.isActive ? "Deactivate" : "Activate"}</Button>}>
    {error && <Alert title="Change not saved" tone="danger">{error}</Alert>}
    <div className="identity-detail-grid"><Card><h2>Account</h2><div className="identity-password"><Input label="Display name" value={profile.displayName} onChange={(e) => setProfile({ ...profile, displayName: e.target.value })} /><Input label="Username" value={profile.username} onChange={(e) => setProfile({ ...profile, username: e.target.value })} /><Button variant="secondary" onClick={() => void run(() => api.updateUser(token, { id: user.id, expectedRevision: user.revision, ...profile }))}>Save profile</Button></div><dl className="identity-details"><dt>Status</dt><dd><Status value={user.isActive ? "Active" : "Inactive"} /></dd><dt>Created</dt><dd>{new Date(user.createdAt).toLocaleString()}</dd><dt>Revision</dt><dd>{user.revision}</dd></dl><div className="identity-password"><Input label="New password" type="password" minLength={12} value={password} onChange={(e) => setPassword(e.target.value)} /><Button variant="secondary" disabled={password.length < 12} onClick={() => void run(async () => { await api.resetUserPassword(token, { id: user.id, password }); setPassword(""); })}>Reset password</Button></div></Card>
      <Card><h2>Roles</h2><p className="field-help">Role assignments are enforced by the native command authorization boundary.</p><div className="identity-check-list">{roles.map((role) => { const checked = user.roles.some((item) => item.code === role.code); return <label key={role.code}><input type="checkbox" checked={checked} onChange={() => void run(() => checked ? api.removeUserRole(token, { userId: user.id, roleCode: role.code }) : api.assignUserRole(token, { userId: user.id, roleCode: role.code }))} /><span><strong>{role.name}</strong><small>{role.description}</small></span></label>; })}</div></Card>
      <Card><h2>Branch memberships</h2><p className="field-help">Memberships define the branches this user can read or change.</p><div className="identity-check-list">{branches.filter((branch) => branch.isActive).map((branch) => { const checked = user.branches.some((item) => item.branchId === branch.id); return <label key={branch.id}><input type="checkbox" checked={checked} onChange={() => { const ids = checked ? user.branches.filter((item) => item.branchId !== branch.id).map((item) => item.branchId) : [...user.branches.map((item) => item.branchId), branch.id]; const primary = ids.includes(user.branches.find((item) => item.isPrimary)?.branchId ?? "") ? user.branches.find((item) => item.isPrimary)?.branchId ?? null : ids[0] ?? null; void run(() => api.setUserBranches(token, { userId: user.id, branchIds: ids, primaryBranchId: primary })); }} /><span><strong>{branch.name}</strong><small>{branch.code}{user.branches.find((item) => item.branchId === branch.id)?.isPrimary ? " · Primary" : ""}</small></span></label>; })}</div>{user.branches.length > 1 && <Select showLabel label="Primary branch" value={user.branches.find((item) => item.isPrimary)?.branchId ?? ""} onChange={(e) => void run(() => api.setUserBranches(token, { userId: user.id, branchIds: user.branches.map((item) => item.branchId), primaryBranchId: e.target.value || null }))}><option value="">No primary branch</option>{user.branches.map((branch) => <option key={branch.branchId} value={branch.branchId}>{branch.branchName}</option>)}</Select>}</Card></div>
  </FeaturePage></>;
}

export function CanonicalRoles() {
  const { token } = useAuth(); const [roles, setRoles] = useState<api.Role[]>([]); const [error, setError] = useState("");
  useEffect(() => { api.listRoles(token).then(setRoles).catch((e) => setError(errorText(e))); }, [token]);
  return <FeaturePage title="Roles" description="System role identities for local accounts."><Alert title="Authorization boundary" tone="info">These durable roles are enforced for every registered native command.</Alert>{error && <Alert title="Roles unavailable" tone="danger">{error}</Alert>}<div className="identity-role-grid">{roles.map((role) => <Card key={role.code}><div className="section-header"><h2>{role.name}</h2><Badge>{role.code}</Badge></div><p>{role.description}</p><small>Protected system role</small></Card>)}</div></FeaturePage>;
}
