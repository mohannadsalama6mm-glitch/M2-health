# Tenancy Model

The ownership chain is `organization -> branch -> organization membership -> branch membership -> enrolled device`.

Organizations use stable UUIDs. Existing branches keep their stable UUIDs, but remain unowned local records after migration 018. `branch_tenant_ownership` is the only local adoption mapping and makes each branch belong to at most one organization. Its composite key lets dependent records prove that a branch and organization match.

`organization_memberships` links an existing local user to one organization with `OWNER`, `ADMIN`, or `MEMBER` cloud-foundation status. These codes are separate from local operational roles. A local `ADMIN` receives no cloud membership automatically. Membership can be marked `REVOKED`; outbox insertion requires an active membership.

`organization_branch_memberships` requires both a valid organization membership and ownership of that branch by the same organization. Composite foreign keys reject cross-organization branch grants. Deleting a branch grant removes future branch authority; historical events remain immutable.

No tenant adoption command exists in Phase 8A. Future adoption must be a trusted, explicit operation that verifies cloud authority, maps existing branch IDs, records the approving cloud actor, and completes atomically. Existing local records must never be inferred into a newly fabricated organization.

Organization-wide records have `organization_id` and no `branch_id`. Branch records carry both values so the database can enforce ownership. Cross-tenant identifiers supplied by a client are never sufficient authorization.
