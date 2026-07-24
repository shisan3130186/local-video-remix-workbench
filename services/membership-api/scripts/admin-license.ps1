param(
  [Parameter(Mandatory = $true)][string]$ApiUrl,
  [Parameter(Mandatory = $true)][ValidateSet('list','create','disable','enable')][string]$Action,
  [string]$CodeId,
  [string]$TokenFile,
  [int]$Days = 30,
  [string]$Label = ''
)

$secureToken = if (-not [string]::IsNullOrWhiteSpace($TokenFile)) {
  if (-not (Test-Path -LiteralPath $TokenFile -PathType Leaf)) { throw 'Admin token file was not found.' }
  Get-Content -LiteralPath $TokenFile -Raw | ConvertTo-SecureString
} else {
  Read-Host 'Enter admin token' -AsSecureString
}

$tokenPointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secureToken)
try {
  $token = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($tokenPointer)
  $headers = @{ Authorization = "Bearer $token" }
  $baseUrl = $ApiUrl.TrimEnd('/')

  if ($Action -eq 'list') {
    $result = Invoke-RestMethod -Method Get -Uri "$baseUrl/v1/admin/codes" -Headers $headers
    $result.codes | Select-Object id,status,duration_days,label,redeemed_by_email,redeemed_at | Format-Table -AutoSize
    return
  }

  if ($Action -eq 'create') {
    $body = @{ durationDays = $Days; label = $Label } | ConvertTo-Json
    $result = Invoke-RestMethod -Method Post -Uri "$baseUrl/v1/admin/codes" -Headers $headers -ContentType 'application/json' -Body $body
    Write-Host "Redemption code: $($result.redemptionCode)" -ForegroundColor Green
    Write-Host "Code ID: $($result.codeId)"
    Write-Host "Membership days: $($result.durationDays)"
    return
  }

  if ([string]::IsNullOrWhiteSpace($CodeId)) { throw 'This action requires -CodeId.' }
  $body = @{ action = $Action } | ConvertTo-Json
  $result = Invoke-RestMethod -Method Patch -Uri "$baseUrl/v1/admin/codes/$CodeId" -Headers $headers -ContentType 'application/json' -Body $body
  $result | ConvertTo-Json -Depth 5
}
finally {
  if ($tokenPointer -ne [IntPtr]::Zero) { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($tokenPointer) }
  Remove-Variable token -ErrorAction SilentlyContinue
}
