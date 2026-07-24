param(
  [string]$EnvironmentId = 'smartcut-membership-d8bfdff54769',
  [string]$FunctionName = 'smartcut-membership-api',
  [switch]$Rotate
)

$ErrorActionPreference = 'Stop'
$serviceRoot = Split-Path -Parent $PSScriptRoot
$privateKeyFile = Join-Path $serviceRoot '.cloudbase-signing-private.local.txt'
$publicKeyFile = Join-Path $serviceRoot '.cloudbase-signing-public.local.txt'
$adminTokenFile = Join-Path $serviceRoot '.cloudbase-admin-token.local.txt'

function Protect-LocalSecret([string]$Value, [string]$Path) {
  $secure = ConvertTo-SecureString $Value -AsPlainText -Force
  $encrypted = ConvertFrom-SecureString $secure
  [System.IO.File]::WriteAllText($Path, $encrypted, [System.Text.UTF8Encoding]::new($false))
}

function Unprotect-LocalSecret([string]$Path) {
  $secure = Get-Content -LiteralPath $Path -Raw | ConvertTo-SecureString
  $pointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
  try {
    return [Runtime.InteropServices.Marshal]::PtrToStringBSTR($pointer)
  }
  finally {
    if ($pointer -ne [IntPtr]::Zero) { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($pointer) }
  }
}

if ($Rotate -or -not (Test-Path -LiteralPath $privateKeyFile -PathType Leaf) -or -not (Test-Path -LiteralPath $publicKeyFile -PathType Leaf)) {
  $keyPair = (& node (Join-Path $PSScriptRoot 'generate-signing-key.mjs') --json) | ConvertFrom-Json
  Protect-LocalSecret -Value $keyPair.privateKey -Path $privateKeyFile
  [System.IO.File]::WriteAllText($publicKeyFile, $keyPair.publicKey, [System.Text.UTF8Encoding]::new($false))
}

if ($Rotate -or -not (Test-Path -LiteralPath $adminTokenFile -PathType Leaf)) {
  $tokenBytes = New-Object byte[] 32
  $random = [Security.Cryptography.RandomNumberGenerator]::Create()
  $random.GetBytes($tokenBytes)
  $random.Dispose()
  $adminToken = [Convert]::ToBase64String($tokenBytes).TrimEnd('=').Replace('+', '-').Replace('/', '_')
  Protect-LocalSecret -Value $adminToken -Path $adminTokenFile
  [Array]::Clear($tokenBytes, 0, $tokenBytes.Length)
  Remove-Variable adminToken -ErrorAction SilentlyContinue
}

$privateKey = Unprotect-LocalSecret -Path $privateKeyFile
$adminToken = Unprotect-LocalSecret -Path $adminTokenFile
try {
  $variables = @(
    @{ Key = 'SIGNING_PRIVATE_KEY_PKCS8_BASE64'; Value = $privateKey },
    @{ Key = 'ADMIN_TOKEN'; Value = $adminToken },
    @{ Key = 'OFFLINE_GRACE_HOURS'; Value = '72' }
  )
  $body = @{
    FunctionName = $FunctionName
    Namespace = $EnvironmentId
    Environment = @{ Variables = $variables }
  } | ConvertTo-Json -Compress -Depth 6
  $escapedBody = $body.Replace('"', '\"')
  $previousErrorPreference = $ErrorActionPreference
  $ErrorActionPreference = 'Continue'
  try {
    $output = & corepack pnpm --package=@cloudbase/cli@latest dlx tcb api scf UpdateFunctionConfiguration `
      -e $EnvironmentId --api-version 2018-04-16 --body $escapedBody --json 2>&1
    $commandExitCode = $LASTEXITCODE
  }
  finally {
    $ErrorActionPreference = $previousErrorPreference
  }
  if ($commandExitCode -ne 0) { throw ($output -join [Environment]::NewLine) }
}
finally {
  Remove-Variable privateKey, adminToken, body, escapedBody, variables, output -ErrorAction SilentlyContinue
}

Write-Host 'CloudBase membership secrets configured.' -ForegroundColor Green
Write-Host "Desktop public key file: $publicKeyFile"
Write-Host "Encrypted admin token file: $adminTokenFile"
