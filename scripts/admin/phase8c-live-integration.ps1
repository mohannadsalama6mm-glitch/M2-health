param([string]$ProjectRef)
$ErrorActionPreference = "Stop"

function Invoke-Json([string]$Method,[string]$Uri,[hashtable]$Headers,[object]$Body=$null) {
  $arguments = @{ Method=$Method; Uri=$Uri; Headers=$Headers; ContentType="application/json"; UserAgent="m2-health-phase8c-admin/1.0" }
  if ($null -ne $Body) { $arguments.Body = ($Body | ConvertTo-Json -Depth 20 -Compress) }
  Invoke-RestMethod @arguments
}
function Expect-Failure([scriptblock]$Operation,[string]$Name) {
  try { & $Operation; throw "$Name unexpectedly succeeded" }
  catch { if ($_.Exception.Message -eq "$Name unexpectedly succeeded") { throw } }
}

if ([string]::IsNullOrWhiteSpace($ProjectRef)) {
  $linkedRefPath = Join-Path $PSScriptRoot "..\..\supabase\.temp\project-ref"
  if (Test-Path -LiteralPath $linkedRefPath) {
    $ProjectRef = (Get-Content -LiteralPath $linkedRefPath -Raw).Trim()
  }
}
if ([string]::IsNullOrWhiteSpace($ProjectRef)) { throw "Link the development project or pass -ProjectRef." }
$supabaseUrl = "https://$ProjectRef.supabase.co"
$keyJson = (& npx --yes supabase@latest projects api-keys --project-ref $ProjectRef --reveal --output json 2>$null | Out-String) | ConvertFrom-Json
$adminKey = ($keyJson | Where-Object { $_.type -eq "secret" } | Select-Object -First 1).api_key
$publishableKey = ($keyJson | Where-Object { $_.type -eq "publishable" } | Select-Object -First 1).api_key
if ([string]::IsNullOrWhiteSpace($adminKey)) { throw "A transient administrative key was unavailable." }
if ([string]::IsNullOrWhiteSpace($publishableKey)) { throw "A publishable key was unavailable." }
$adminHeaders = @{ apikey=$adminKey; Authorization="Bearer $adminKey" }
$publicHeaders = @{ apikey=$publishableKey }
$suffix = [guid]::NewGuid().ToString('N')
$password = "Phase8C!" + [guid]::NewGuid().ToString('N') + "z9"
$users = @()
$organizations = @()
$cleanupFile = Join-Path $env:TEMP ("m2-phase8c-cleanup-" + $suffix + ".sql")
try {
  foreach ($index in 1..3) {
    $created = Invoke-Json POST ($supabaseUrl + "/auth/v1/admin/users") $adminHeaders @{ email=("phase8c-"+$suffix+"-"+$index+"@example.invalid"); password=$password; email_confirm=$true }
    $users += $created.id
  }
  $sessions = @()
  foreach ($index in 1..3) {
    $sessions += Invoke-Json POST ($supabaseUrl + "/auth/v1/token?grant_type=password") $publicHeaders @{ email=("phase8c-"+$suffix+"-"+$index+"@example.invalid"); password=$password }
  }
  $auth = @($sessions | ForEach-Object { @{ apikey=$publishableKey; Authorization=("Bearer "+$_.access_token) } })
  $org1=[guid]::NewGuid(); $org2=[guid]::NewGuid(); $branch1=[guid]::NewGuid(); $branch2=[guid]::NewGuid();
  $device1=[guid]::NewGuid(); $device2=[guid]::NewGuid(); $device3=[guid]::NewGuid();
  $organizations=@($org1,$org2)
  Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/bootstrap_sync_organization") $auth[0] @{ p_organization_id=$org1; p_slug=("phase8c-test-"+$suffix.Substring(0,12)+"-a"); p_name="Phase 8C A"; p_branch_id=$branch1; p_branch_code="A"; p_branch_name="A"; p_device_id=$device1; p_device_name="Device A" } | Out-Null
  Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/bootstrap_sync_organization") $auth[2] @{ p_organization_id=$org2; p_slug=("phase8c-test-"+$suffix.Substring(0,12)+"-b"); p_name="Phase 8C B"; p_branch_id=$branch2; p_branch_code="B"; p_branch_name="B"; p_device_id=$device3; p_device_name="Device C" } | Out-Null
  Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/grant_sync_membership") $auth[0] @{ p_organization_id=$org1; p_user_id=$users[1]; p_role_code="MEMBER"; p_branch_ids=@($branch1) } | Out-Null
  Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/enroll_sync_device") $auth[1] @{ p_organization_id=$org1; p_device_id=$device2; p_display_name="Device B" } | Out-Null

  $eventId=[guid]::NewGuid(); $payload=@{ quantityDelta=2 }; $canonical='{"quantityDelta":2}'
  $sha256=[System.Security.Cryptography.SHA256]::Create(); try { $sha=$sha256.ComputeHash([Text.Encoding]::UTF8.GetBytes($canonical)) } finally { $sha256.Dispose() }; $hash=($sha | ForEach-Object { $_.ToString('x2') }) -join ''
  $event=@{ eventId=$eventId; organizationId=$org1; branchId=$branch1; deviceId=$device1; actorUserId=[guid]::NewGuid(); entityType="inventory_movement"; entityId="live-test-movement"; operation="APPENDED"; payload=$payload; payloadHash=$hash; schemaVersion=1; causationId=$null; occurredAt="2026-10-10T00:00:00Z"; localSequence=1 }
  $first=Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/push_sync_events") $auth[0] @{ p_device_id=$device1; p_events=@($event) }
  $duplicate=Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/push_sync_events") $auth[0] @{ p_device_id=$device1; p_events=@($event) }
  $pulled=Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/pull_sync_events") $auth[1] @{ p_device_id=$device2; p_after_sequence=0; p_limit=200 }
  if ($first.Count -ne 1 -or $duplicate[0].duplicate -ne $true -or $pulled.Count -ne 1 -or $pulled[0].actorUserId -ne $users[0]) { throw "Authenticated push/pull identity assertions failed." }
  Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/ack_sync_events") $auth[0] @{ p_device_id=$device1; p_receipts=@($first) } | Out-Null

  $refreshed=Invoke-Json POST ($supabaseUrl+"/auth/v1/token?grant_type=refresh_token") $publicHeaders @{ refresh_token=$sessions[0].refresh_token }
  if ([string]::IsNullOrWhiteSpace($refreshed.access_token)) { throw "Token refresh failed." }
  $auth[0]=@{ apikey=$publishableKey; Authorization=("Bearer "+$refreshed.access_token) }
  Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/pull_sync_events") $auth[0] @{ p_device_id=$device1; p_after_sequence=0; p_limit=200 } | Out-Null

  $foreign=$event.Clone(); $foreign.eventId=[guid]::NewGuid(); $foreign.organizationId=$org1; $foreign.deviceId=$device3; $foreign.localSequence=2
  Expect-Failure { Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/push_sync_events") $auth[2] @{ p_device_id=$device3; p_events=@($foreign) } | Out-Null } "Cross-tenant push"
  $badHash=$event.Clone(); $badHash.eventId=[guid]::NewGuid(); $badHash.localSequence=2; $badHash.payloadHash=('0'*64)
  Expect-Failure { Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/push_sync_events") $auth[0] @{ p_device_id=$device1; p_events=@($badHash) } | Out-Null } "Malformed payload hash"
  Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/revoke_sync_device") $auth[1] @{ p_device_id=$device2 } | Out-Null
  Expect-Failure { Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/pull_sync_events") $auth[1] @{ p_device_id=$device2; p_after_sequence=0; p_limit=200 } | Out-Null } "Revoked device pull"
  Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/revoke_sync_member") $auth[0] @{ p_organization_id=$org1; p_user_id=$users[1] } | Out-Null
  Expect-Failure { Invoke-Json POST ($supabaseUrl+"/rest/v1/rpc/enroll_sync_device") $auth[1] @{ p_organization_id=$org1; p_device_id=[guid]::NewGuid(); p_display_name="Denied" } | Out-Null } "Revoked membership enrollment"
  [ordered]@{ usersAuthenticated=3; organizations=2; devices=3; pushed=1; duplicateReplay=$true; secondDevicePull=1; serverActorDerived=$true; tokenRefresh=$true; crossTenantDenied=$true; malformedHashDenied=$true; revokedDeviceDenied=$true; revokedMemberDenied=$true } | ConvertTo-Json -Compress
}
finally {
  if ($organizations.Count -gt 0) {
    $orgList=($organizations | ForEach-Object { "'"+$_.ToString()+"'::uuid" }) -join ','
    @"
begin;
delete from app.sync_receipts where organization_id in ($orgList);
delete from app.sync_events where organization_id in ($orgList);
delete from app.local_identity_links where organization_id in ($orgList);
delete from app.devices where organization_id in ($orgList);
delete from app.branch_memberships where organization_id in ($orgList);
delete from app.branches where organization_id in ($orgList);
delete from app.organization_memberships where organization_id in ($orgList);
delete from app.organizations where id in ($orgList);
commit;
"@ | Set-Content -LiteralPath $cleanupFile -Encoding utf8
    & npx --yes supabase@latest db query --linked --file $cleanupFile 2>$null | Out-Null
  }
  foreach ($user in $users) { try { Invoke-RestMethod -Method Delete -Uri ($supabaseUrl+"/auth/v1/admin/users/"+$user) -Headers $adminHeaders -UserAgent "m2-health-phase8c-admin/1.0" | Out-Null } catch { Write-Warning "A synthetic Auth user requires manual cleanup." } }
  Remove-Item -LiteralPath $cleanupFile -Force -ErrorAction SilentlyContinue
  $adminKey=$null; $publishableKey=$null; $password=$null
}
