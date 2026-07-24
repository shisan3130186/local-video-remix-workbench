param([switch]$SmokeTest)

$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Add-Type -AssemblyName PresentationFramework, PresentationCore, WindowsBase

$xamlPath = Join-Path $PSScriptRoot 'ManagerWindow.xaml'
$modulePath = Join-Path $PSScriptRoot 'SmartCutMembershipAdmin.psm1'
$xamlText = [IO.File]::ReadAllText($xamlPath, [Text.Encoding]::UTF8)
$xml = New-Object Xml.XmlDocument
$xml.PreserveWhitespace = $true
$xml.LoadXml($xamlText)
$reader = New-Object Xml.XmlNodeReader $xml
$window = [Windows.Markup.XamlReader]::Load($reader)
Import-Module $modulePath -Force

function Find-Control([string]$Name) {
  $control = $window.FindName($Name)
  if ($null -eq $control) { throw "UI control was not found: $Name" }
  return $control
}

$daysInput = Find-Control 'DaysInput'
$labelInput = Find-Control 'LabelInput'
$generateButton = Find-Control 'GenerateButton'
$copyButton = Find-Control 'CopyButton'
$codeText = Find-Control 'CodeText'
$refreshButton = Find-Control 'RefreshButton'
$filterCombo = Find-Control 'FilterCombo'
$codesGrid = Find-Control 'CodesGrid'
$disableButton = Find-Control 'DisableButton'
$enableButton = Find-Control 'EnableButton'
$statusText = Find-Control 'StatusText'
$lastRefreshText = Find-Control 'LastRefreshText'
$selectionHint = Find-Control 'SelectionHint'
$unusedCount = Find-Control 'UnusedCount'
$redeemedCount = Find-Control 'RedeemedCount'
$disabledCount = Find-Control 'DisabledCount'
$closeButton = Find-Control 'CloseButton'
$allCodes = @()
$latestCode = ''

function Convert-CodeRow($Code) {
  $statusMap = @{ unused = '未使用'; redeemed = '已使用'; disabled = '已停用' }
  $created = if ($Code.created_at) {
    [DateTimeOffset]::FromUnixTimeSeconds([long]$Code.created_at).LocalDateTime.ToString('yyyy-MM-dd HH:mm')
  } else { '-' }
  [pscustomobject]@{
    StatusText = $statusMap[[string]$Code.status]
    DaysText = "$($Code.duration_days)天"
    LabelText = if ([string]::IsNullOrWhiteSpace([string]$Code.label)) { '未填写' } else { [string]$Code.label }
    RedeemedBy = if ([string]::IsNullOrWhiteSpace([string]$Code.redeemed_by_email)) { '-' } else { [string]$Code.redeemed_by_email }
    CreatedAtText = $created
    RawId = [string]$Code.id
    RawStatus = [string]$Code.status
  }
}

function Update-Counts {
  $unusedCount.Text = @($allCodes | Where-Object { $_.RawStatus -eq 'unused' }).Count
  $redeemedCount.Text = @($allCodes | Where-Object { $_.RawStatus -eq 'redeemed' }).Count
  $disabledCount.Text = @($allCodes | Where-Object { $_.RawStatus -eq 'disabled' }).Count
}

function Update-Grid {
  $selectedTag = [string]$filterCombo.SelectedItem.Tag
  $visible = if ($selectedTag -eq 'all') { $allCodes } else { @($allCodes | Where-Object { $_.RawStatus -eq $selectedTag }) }
  $codesGrid.ItemsSource = @($visible)
  Update-Counts
}

function Update-SelectionActions {
  $selected = $codesGrid.SelectedItem
  $disableButton.IsEnabled = $null -ne $selected -and $selected.RawStatus -eq 'unused'
  $enableButton.IsEnabled = $null -ne $selected -and $selected.RawStatus -eq 'disabled'
  $selectionHint.Text = if ($null -eq $selected) { '选择一条记录后可管理未使用卡密。' } else { "当前选择：$($selected.StatusText) · $($selected.DaysText) · $($selected.LabelText)" }
}

function Set-Busy([bool]$Busy, [string]$Message) {
  $window.Cursor = if ($Busy) { [Windows.Input.Cursors]::Wait } else { [Windows.Input.Cursors]::Arrow }
  $generateButton.IsEnabled = -not $Busy
  $refreshButton.IsEnabled = -not $Busy
  $statusText.Text = $Message
  $window.Dispatcher.Invoke([action]{}, [Windows.Threading.DispatcherPriority]::Background)
}

function Show-OperationError($ErrorRecord) {
  $message = if ($ErrorRecord.Exception.Message) { $ErrorRecord.Exception.Message } else { [string]$ErrorRecord }
  $statusText.Text = "操作失败：$message"
  [Windows.MessageBox]::Show($message, 'SmartCut', 'OK', 'Error') | Out-Null
}

function Refresh-Codes {
  Set-Busy $true '正在刷新兑换码记录...'
  try {
    $script:allCodes = @(Get-SmartCutCodes | ForEach-Object { Convert-CodeRow $_ })
    Update-Grid
    Update-SelectionActions
    $lastRefreshText.Text = "最后刷新：$(Get-Date -Format 'HH:mm:ss')"
    $statusText.Text = "刷新成功，共 $($script:allCodes.Count) 条记录。"
  }
  catch { Show-OperationError $_ }
  finally { Set-Busy $false $statusText.Text }
}

function Select-Days([int]$Days) {
  $daysInput.Text = [string]$Days
  $labelInput.Text = if ($Days -eq 1) { '1天测试会员' } else { "$($Days)天会员" }
}

foreach ($preset in @(
  @{ Name = 'Day1Button'; Days = 1 }, @{ Name = 'Day7Button'; Days = 7 },
  @{ Name = 'Day30Button'; Days = 30 }, @{ Name = 'Day90Button'; Days = 90 },
  @{ Name = 'Day365Button'; Days = 365 }
)) {
  $button = Find-Control $preset.Name
  $days = [int]$preset.Days
  $button.Add_Click({ Select-Days $days }.GetNewClosure())
}

(Find-Control 'ClearButton').Add_Click({ $daysInput.Text = ''; $labelInput.Text = ''; $daysInput.Focus() })
$closeButton.Add_Click({ $window.Close() })
$copyButton.Add_Click({
  if (-not [string]::IsNullOrWhiteSpace($script:latestCode)) {
    [Windows.Clipboard]::SetText($script:latestCode)
    $statusText.Text = '兑换码已复制到剪贴板。'
  }
})
$filterCombo.Add_SelectionChanged({ if ($null -ne $filterCombo.SelectedItem) { Update-Grid; Update-SelectionActions } })
$codesGrid.Add_SelectionChanged({ Update-SelectionActions })
$refreshButton.Add_Click({ Refresh-Codes })

$generateButton.Add_Click({
  $days = 0
  if (-not [int]::TryParse($daysInput.Text.Trim(), [ref]$days) -or $days -lt 1 -or $days -gt 3650) {
    [Windows.MessageBox]::Show('会员天数必须是1到3650之间的整数。', 'SmartCut', 'OK', 'Warning') | Out-Null
    $daysInput.Focus()
    return
  }
  Set-Busy $true '正在生成兑换码...'
  try {
    $result = New-SmartCutCode -Days $days -Label $labelInput.Text
    $script:latestCode = [string]$result.redemptionCode
    $codeText.Text = $script:latestCode
    $copyButton.IsEnabled = $true
    [Windows.Clipboard]::SetText($script:latestCode)
    $statusText.Text = "生成成功：$days 天会员码已复制。"
    Refresh-Codes
  }
  catch { Show-OperationError $_ }
  finally { Set-Busy $false $statusText.Text }
})

$disableButton.Add_Click({
  $selected = $codesGrid.SelectedItem
  if ($null -eq $selected -or $selected.RawStatus -ne 'unused') { return }
  $confirm = [Windows.MessageBox]::Show('确定停用这条尚未使用的兑换码吗？', 'SmartCut', 'YesNo', 'Question')
  if ($confirm -ne 'Yes') { return }
  Set-Busy $true '正在停用兑换码...'
  try { Set-SmartCutCodeStatus -CodeId $selected.RawId -Action disable | Out-Null; Refresh-Codes }
  catch { Show-OperationError $_ }
  finally { Set-Busy $false $statusText.Text }
})

$enableButton.Add_Click({
  $selected = $codesGrid.SelectedItem
  if ($null -eq $selected -or $selected.RawStatus -ne 'disabled') { return }
  Set-Busy $true '正在恢复兑换码...'
  try { Set-SmartCutCodeStatus -CodeId $selected.RawId -Action enable | Out-Null; Refresh-Codes }
  catch { Show-OperationError $_ }
  finally { Set-Busy $false $statusText.Text }
})

if ($SmokeTest) {
  $codes = @(Get-SmartCutCodes)
  [pscustomobject]@{
    xamlLoaded = $null -ne $window
    controlsLoaded = $null -ne $codesGrid -and $null -ne $generateButton
    adminApiConnected = $true
    codeCount = $codes.Count
  } | ConvertTo-Json -Compress
  $window.Close()
  exit 0
}

$window.Add_ContentRendered({ Refresh-Codes })
$window.ShowDialog() | Out-Null
