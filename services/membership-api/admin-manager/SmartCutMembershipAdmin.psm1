$script:ApiUrl = 'https://smartcut-membership-d8bfdff54769-1457945645.ap-shanghai.app.tcloudbase.com/smartcut-membership'
$script:TokenFile = Join-Path (Split-Path -Parent $PSScriptRoot) '.cloudbase-admin-token.local.txt'

function Invoke-WithAdminToken {
  param([Parameter(Mandatory = $true)][scriptblock]$Action)

  if (-not (Test-Path -LiteralPath $script:TokenFile -PathType Leaf)) {
    throw 'Encrypted admin token file was not found. Run configure-cloudbase-secrets.ps1 first.'
  }

  $secureToken = Get-Content -LiteralPath $script:TokenFile -Raw | ConvertTo-SecureString
  $tokenPointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secureToken)
  try {
    $token = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($tokenPointer)
    return & $Action $token
  }
  finally {
    if ($tokenPointer -ne [IntPtr]::Zero) {
      [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($tokenPointer)
    }
    Remove-Variable token -ErrorAction SilentlyContinue
  }
}

function Invoke-SmartCutAdminRequest {
  param(
    [Parameter(Mandatory = $true)][ValidateSet('GET', 'POST', 'PATCH')][string]$Method,
    [Parameter(Mandatory = $true)][string]$Path,
    [object]$Body
  )

  return Invoke-WithAdminToken {
    param($token)
    $request = @{
      Method = $Method
      Uri = "$($script:ApiUrl.TrimEnd('/'))$Path"
      Headers = @{ Authorization = "Bearer $token" }
      TimeoutSec = 30
    }
    if ($null -ne $Body) {
      $request.ContentType = 'application/json'
      $request.Body = $Body | ConvertTo-Json -Compress -Depth 5
    }
    Invoke-RestMethod @request
  }
}

function Get-SmartCutCodes {
  $result = Invoke-SmartCutAdminRequest -Method GET -Path '/v1/admin/codes'
  return @($result.codes)
}

function New-SmartCutCode {
  param(
    [Parameter(Mandatory = $true)][ValidateRange(1, 3650)][int]$Days,
    [string]$Label = ''
  )

  return Invoke-SmartCutAdminRequest -Method POST -Path '/v1/admin/codes' -Body @{
    durationDays = $Days
    label = $Label.Trim()
  }
}

function Set-SmartCutCodeStatus {
  param(
    [Parameter(Mandatory = $true)][string]$CodeId,
    [Parameter(Mandatory = $true)][ValidateSet('disable', 'enable')][string]$Action
  )

  return Invoke-SmartCutAdminRequest -Method PATCH -Path "/v1/admin/codes/$CodeId" -Body @{
    action = $Action
  }
}

Export-ModuleMember -Function Get-SmartCutCodes, New-SmartCutCode, Set-SmartCutCodeStatus
