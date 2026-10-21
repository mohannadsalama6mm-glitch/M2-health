param([string]$ProjectRef)

$ErrorActionPreference = "Stop"

function Invoke-Json(
  [string]$Method,
  [string]$Uri,
  [hashtable]$Headers,
  [object]$Body = $null
) {
  $arguments = @{
    Method      = $Method
    Uri         = $Uri
    Headers     = $Headers
    ContentType = "application/json"
    UserAgent   = "m2-health-phase9a2-admin/1.0"
  }

  if ($null -ne $Body) {
    $arguments.Body = ($Body | ConvertTo-Json -Depth 20 -Compress)
  }

  Invoke-RestMethod @arguments
}

function Expect-Failure(
  [scriptblock]$Operation,
  [string]$Name
) {
  try {
    & $Operation
    throw "$Name unexpectedly succeeded"
  }
  catch {
    if ($_.Exception.Message -eq "$Name unexpectedly succeeded") {
      throw
    }
  }
}

# ------------------------------------------------------------
# Resolve project
# ------------------------------------------------------------

if ([string]::IsNullOrWhiteSpace($ProjectRef)) {
  $linkedRefPath = Join-Path `
    $PSScriptRoot `
    "..\..\supabase\.temp\project-ref"

  if (Test-Path -LiteralPath $linkedRefPath) {
    $ProjectRef = (
      Get-Content -LiteralPath $linkedRefPath -Raw
    ).Trim()
  }
}

if ([string]::IsNullOrWhiteSpace($ProjectRef)) {
  throw "Link the development project or pass -ProjectRef."
}

$supabaseUrl = "https://$ProjectRef.supabase.co"

# ------------------------------------------------------------
# Read modern Supabase keys from current PowerShell session
# ------------------------------------------------------------

$adminKey = $env:M2_SUPABASE_SECRET_KEY
$publishableKey = $env:M2_SUPABASE_PUBLISHABLE_KEY

if (
  [string]::IsNullOrWhiteSpace($adminKey) -or
  [string]::IsNullOrWhiteSpace($publishableKey)
) {
  throw "M2 Supabase environment keys are unavailable."
}

if (-not $adminKey.StartsWith("sb_secret_")) {
  throw "M2_SUPABASE_SECRET_KEY is not a modern Supabase secret key."
}

if (-not $publishableKey.StartsWith("sb_publishable_")) {
  throw "M2_SUPABASE_PUBLISHABLE_KEY is not a modern Supabase publishable key."
}

# ------------------------------------------------------------
# Headers
# ------------------------------------------------------------

$adminHeaders = @{
  apikey = $adminKey
}

$publicHeaders = @{
  apikey = $publishableKey
}

# ------------------------------------------------------------
# Synthetic test data
# ------------------------------------------------------------

$suffix = [guid]::NewGuid().ToString("N")
$password = "Phase9A2!" + [guid]::NewGuid().ToString("N") + "z9"

$users = @()
$cleanupFile = Join-Path $env:TEMP ("m2-phase9a3-cleanup-" + $suffix + ".sql")

try {
  # ----------------------------------------------------------
  # Create synthetic users
  # ----------------------------------------------------------

  foreach ($index in 1..2) {
    $created = Invoke-Json `
      POST `
      ($supabaseUrl + "/auth/v1/admin/users") `
      $adminHeaders `
      @{
        email = (
          "phase9a2-" +
          $suffix +
          "-" +
          $index +
          "@example.invalid"
        )
        password      = $password
        email_confirm = $true
        user_metadata = @{
          full_name = ("Synthetic Portal " + $index)
        }
      }

    $users += $created.id
  }

  # ----------------------------------------------------------
  # Authenticate users
  # ----------------------------------------------------------

  $sessions = @()

  foreach ($index in 1..2) {
    $sessions += Invoke-Json `
      POST `
      ($supabaseUrl + "/auth/v1/token?grant_type=password") `
      $publicHeaders `
      @{
        email = (
          "phase9a2-" +
          $suffix +
          "-" +
          $index +
          "@example.invalid"
        )
        password = $password
      }
  }

  $auth = @(
    $sessions |
    ForEach-Object {
      @{
        apikey        = $publishableKey
        Authorization = ("Bearer " + $_.access_token)
      }
    }
  )

  # ----------------------------------------------------------
  # Initial state
  # ----------------------------------------------------------

  $snapshotA = Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/get_portal_snapshot") `
    $auth[0] `
    @{}

  $snapshotB = Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/get_portal_snapshot") `
    $auth[1] `
    @{}

  if (
    $snapshotA.status -ne "UNACTIVATED" -or
    $snapshotB.status -ne "UNACTIVATED"
  ) {
    throw "Synthetic accounts did not start unactivated."
  }

  # ----------------------------------------------------------
  # Trial
  # ----------------------------------------------------------

  Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/start_early_access_trial") `
    $auth[0] `
    @{} |
    Out-Null

  Expect-Failure {
    Invoke-Json `
      POST `
      ($supabaseUrl + "/rest/v1/rpc/start_early_access_trial") `
      $auth[0] `
      @{} |
      Out-Null
  } "Second account trial"

  # ----------------------------------------------------------
  # Devices
  # ----------------------------------------------------------

  $deviceA = [guid]::NewGuid()
  $deviceB = [guid]::NewGuid()

  $trialClaims = Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/prepare_license_claims") `
    $auth[0] `
    @{
      p_device_id    = $deviceA
      p_display_name = "Synthetic Counter A"
      p_app_version  = "0.4.0-beta.1"
      p_build_id     = "20261020.1"
    }

  if (
    $trialClaims.status -ne "TRIAL" -or
    $trialClaims.claims.state -ne "ACTIVE" -or
    $trialClaims.claims.deviceId -ne $deviceA
  ) {
    throw "Trial device claims failed."
  }

  Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/start_early_access_trial") `
    $auth[1] `
    @{} |
    Out-Null

  # Cross-account device registration must fail
  Expect-Failure {
    Invoke-Json `
      POST `
      ($supabaseUrl + "/rest/v1/rpc/prepare_license_claims") `
      $auth[1] `
      @{
        p_device_id    = $deviceA
        p_display_name = "Cross account"
        p_app_version  = "0.4.0-beta.1"
        p_build_id     = "20261020.1"
      } |
      Out-Null
  } "Cross-account device registration"

  # Cross-account revoke must fail
  Expect-Failure {
    Invoke-Json `
      POST `
      ($supabaseUrl + "/rest/v1/rpc/revoke_license_device") `
      $auth[1] `
      @{
        p_device_id = $deviceA
      } |
      Out-Null
  } "Cross-account device revoke"

  # Register user B device
  Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/prepare_license_claims") `
    $auth[1] `
    @{
      p_device_id    = $deviceB
      p_display_name = "Synthetic Counter B"
      p_app_version  = "0.4.0-beta.1"
      p_build_id     = "20261020.1"
    } |
    Out-Null

  # Revoke user B device
  Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/revoke_license_device") `
    $auth[1] `
    @{
      p_device_id = $deviceB
    } |
    Out-Null

  $revoked = Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/prepare_license_claims") `
    $auth[1] `
    @{
      p_device_id    = $deviceB
      p_display_name = "Synthetic Counter B"
      p_app_version  = "0.4.0-beta.1"
      p_build_id     = "20261020.1"
    }

  if (
    $revoked.status -ne "REVOKED" -or
    $revoked.claims.state -ne "REVOKED"
  ) {
    throw "Device revocation did not propagate."
  }

  # ----------------------------------------------------------
  # Manual activation request
  # ----------------------------------------------------------

  Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/request_manual_activation") `
    $auth[0] `
    @{
      p_message = "Synthetic Early Access activation request"
    } |
    Out-Null

  $pending = Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/admin_list_activation_requests") `
    $adminHeaders `
    @{}

  if (
    (
      $pending |
      Where-Object {
        $_.accountId -eq $snapshotA.accountId
      }
    ).Count -ne 1
  ) {
    throw "Admin request listing failed."
  }

  # ----------------------------------------------------------
  # Unauthorized admin attempt must fail
  # ----------------------------------------------------------

  Expect-Failure {
    Invoke-Json `
      POST `
      ($supabaseUrl + "/rest/v1/rpc/admin_set_license_state") `
      $auth[0] `
      @{
        p_account_id  = $snapshotA.accountId
        p_state       = "ACTIVE"
        p_plan_code   = "SINGLE_PHARMACY"
        p_starts_at   = (Get-Date).ToUniversalTime().ToString("o")
        p_ends_at     = $null
        p_max_devices = 2
        p_note        = "Unauthorized synthetic attempt"
      } |
      Out-Null
  } "Unauthorized admin activation"

  Expect-Failure {
    Invoke-Json `
      POST `
      ($supabaseUrl + "/rest/v1/rpc/admin_list_activation_requests") `
      $publicHeaders `
      @{} |
      Out-Null
  } "Anonymous admin listing"

  # ----------------------------------------------------------
  # Admin activation
  # ----------------------------------------------------------

  $starts = (Get-Date).ToUniversalTime()
  $ends = $starts.AddDays(30)

  Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/admin_set_license_state") `
    $adminHeaders `
    @{
      p_account_id  = $snapshotA.accountId
      p_state       = "ACTIVE"
      p_plan_code   = "SINGLE_PHARMACY"
      p_starts_at   = $starts.ToString("o")
      p_ends_at     = $ends.ToString("o")
      p_max_devices = 2
      p_note        = "Synthetic live activation"
    } |
    Out-Null

  $active = Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/get_portal_snapshot") `
    $auth[0] `
    @{}

  if (
    $active.status -ne "ACTIVE" -or
    $active.maxDevices -ne 2 -or
    $active.activationRequests[0].status -ne "FULFILLED"
  ) {
    throw "Manual activation did not reach the portal."
  }

  # ----------------------------------------------------------
  # Suspension
  # ----------------------------------------------------------

  Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/admin_set_license_state") `
    $adminHeaders `
    @{
      p_account_id  = $snapshotA.accountId
      p_state       = "SUSPENDED"
      p_plan_code   = "SINGLE_PHARMACY"
      p_starts_at   = $starts.ToString("o")
      p_ends_at     = $ends.ToString("o")
      p_max_devices = 2
      p_note        = "Synthetic suspension check"
    } |
    Out-Null

  $suspended = Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/prepare_license_claims") `
    $auth[0] `
    @{
      p_device_id    = $deviceA
      p_display_name = "Synthetic Counter A"
      p_app_version  = "0.4.0-beta.1"
      p_build_id     = "20261020.1"
    }

  if (
    $suspended.status -ne "SUSPENDED" -or
    $suspended.claims.state -ne "SUSPENDED"
  ) {
    throw "Suspension did not propagate."
  }

  # ----------------------------------------------------------
  # Expiry
  # ----------------------------------------------------------

  Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/admin_set_license_state") `
    $adminHeaders `
    @{
      p_account_id  = $snapshotB.accountId
      p_state       = "EXPIRED"
      p_plan_code   = "EARLY_ACCESS"
      p_starts_at   = $null
      p_ends_at     = $null
      p_max_devices = 1
      p_note        = "Synthetic expiry check"
    } |
    Out-Null

  $expired = Invoke-Json `
    POST `
    ($supabaseUrl + "/rest/v1/rpc/get_portal_snapshot") `
    $auth[1] `
    @{}

  if ($expired.status -ne "EXPIRED") {
    throw "Expired state did not reach the portal."
  }

  # ----------------------------------------------------------
  # PASS
  # ----------------------------------------------------------

  [ordered]@{
    usersAuthenticated             = 2
    accountIsolation               = $true
    firstTrialStarted              = $true
    secondTrialDenied              = $true
    deviceRegistered               = $true
    crossAccountRegistrationDenied = $true
    crossAccountRevokeDenied       = $true
    revokedStatePropagated         = $true
    activationRequestListed        = $true
    unauthorizedAdminDenied        = $true
    anonymousAdminDenied            = $true
    manualActivationPropagated     = $true
    suspendedStatePropagated       = $true
    expiredStatePropagated         = $true
  } |
  ConvertTo-Json -Compress
}
finally {
  # ----------------------------------------------------------
  # Cleanup synthetic database rows directly through the linked development
  # database. The explicit order preserves production restrict constraints.
  # ----------------------------------------------------------

  if ($users.Count -gt 0) {
    $userList = ($users | ForEach-Object { "'" + ([guid]$_).ToString() + "'::uuid" }) -join ','
    @"
begin;
delete from app.license_admin_audit where account_id in (select id from app.license_accounts where owner_user_id in ($userList));
delete from app.activation_requests where requested_by in ($userList) or account_id in (select id from app.license_accounts where owner_user_id in ($userList));
delete from app.license_devices where owner_user_id in ($userList) or account_id in (select id from app.license_accounts where owner_user_id in ($userList));
delete from app.license_accounts where owner_user_id in ($userList);
delete from app.customer_profiles where user_id in ($userList);
commit;
"@ | Set-Content -LiteralPath $cleanupFile -Encoding utf8
    & npx --yes supabase@latest db query --linked --file $cleanupFile 2>$null | Out-Null
    if ($LASTEXITCODE -ne 0) { Write-Warning "Synthetic Phase 9A.3 database rows require manual cleanup." }
  }

  # ----------------------------------------------------------
  # Remove synthetic Auth users
  # ----------------------------------------------------------

  foreach ($user in $users) {
    try {
      Invoke-RestMethod `
        -Method Delete `
        -Uri ($supabaseUrl + "/auth/v1/admin/users/" + $user) `
        -Headers $adminHeaders `
        -UserAgent "m2-health-phase9a2-admin/1.0" |
        Out-Null
    }
    catch {
      Write-Warning "A synthetic Phase 9A.2 Auth user requires manual cleanup."
    }
  }

  Remove-Item -LiteralPath $cleanupFile -Force -ErrorAction SilentlyContinue

  # ----------------------------------------------------------
  # Clear sensitive session values
  # ----------------------------------------------------------

  $adminKey = $null
  $publishableKey = $null
  $password = $null
}
