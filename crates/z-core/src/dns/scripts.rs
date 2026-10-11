pub(super) const PREAMBLE: &str = "$ErrorActionPreference='Stop';$ProgressPreference='SilentlyContinue';$ConfirmPreference='None';[Console]::OutputEncoding=New-Object System.Text.UTF8Encoding $false;$OutputEncoding=[Console]::OutputEncoding;$warnings=@();";

pub(super) const HELPERS: &str = r#"
function Test-Ignored($a) {
  $n = (([string]$a.Name) + ' ' + ([string]$a.InterfaceDescription)).ToLowerInvariant()
  foreach ($k in 'loopback','wintun','wireguard','openvpn','tap','vpn','v2ray','xray','sing-box','clash','tun') {
    if ($n.Contains($k)) { return $true }
  }
  return $false
}
function Get-Targets {
  $all = @(Get-NetAdapter | Where-Object { $_.Status -eq 'Up' -and -not (Test-Ignored $_) })
  $hw = @($all | Where-Object { $_.HardwareInterface })
  if ($hw.Count -gt 0) { return $hw }
  return $all
}
# Индекс адаптера может поменяться после перезагрузки, поэтому ищем по GUID
function Resolve-Index($a) {
  $g = [string]$a.interface_guid
  if ($g) {
    $n = Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object { [string]$_.InterfaceGuid -eq $g } | Select-Object -First 1
    if ($n) { return [int]$n.ifIndex }
    return 0
  }
  return [int]$a.interface_index
}
function Get-RegDns($guid, $v6) {
  if (-not $guid) { return '' }
  $root = if ($v6) { 'Tcpip6' } else { 'Tcpip' }
  $path = "HKLM:\SYSTEM\CurrentControlSet\Services\$root\Parameters\Interfaces\$guid"
  try { return [string](Get-ItemProperty -LiteralPath $path -Name NameServer -ErrorAction Stop).NameServer } catch { return '' }
}
function Clear-RegDns($guid) {
  if (-not $guid) { return }
  foreach ($root in 'Tcpip','Tcpip6') {
    $path = "HKLM:\SYSTEM\CurrentControlSet\Services\$root\Parameters\Interfaces\$guid"
    if (Test-Path -LiteralPath $path) { try { Set-ItemProperty -LiteralPath $path -Name NameServer -Value '' } catch {} }
  }
}
# Остатки ручной настройки DoH для адаптера (нужно только при жёстком сбросе)
function Clear-DohInterfaceSettings($guid) {
  if (-not $guid) { return }
  $path = "HKLM:\SYSTEM\CurrentControlSet\Services\Dnscache\InterfaceSpecificParameters\$guid\DohInterfaceSettings"
  if (Test-Path -LiteralPath $path) { try { Remove-Item -LiteralPath $path -Recurse -Force | Out-Null } catch {} }
}
function Split-List($v) { @(([string]$v) -split '[,\s]+' | Where-Object { $_ -ne '' }) }
function Remove-OurDoh($p) {
  if (Get-Command Remove-DnsClientDohServerAddress -ErrorAction SilentlyContinue) {
    foreach ($d in @($p.doh_added)) {
      try {
        if (Get-DnsClientDohServerAddress -ServerAddress ([string]$d.server_address) -ErrorAction SilentlyContinue) {
          Remove-DnsClientDohServerAddress -ServerAddress ([string]$d.server_address) | Out-Null
        }
      }
      catch { $script:warnings += ('DoH ' + [string]$d.server_address + ' не удалён: ' + $_.Exception.Message) }
    }
  }
  if (Get-Command Set-DnsClientDohServerAddress -ErrorAction SilentlyContinue) {
    foreach ($m in @($p.doh_modified)) {
      if (@($p.doh_added | Where-Object { $_.server_address -eq $m.server_address }).Count -gt 0) { continue }
      try {
        Set-DnsClientDohServerAddress -ServerAddress ([string]$m.server_address) -DohTemplate ([string]$m.doh_template) -AllowFallbackToUdp ([bool]$m.allow_fallback) -AutoUpgrade ([bool]$m.auto_upgrade) | Out-Null
      } catch { $script:warnings += ('DoH ' + [string]$m.server_address + ' не восстановлен: ' + $_.Exception.Message) }
    }
  }
}
"#;

pub(super) const SNAPSHOT: &str = r#"
$rows = @()
foreach ($a in @(Get-Targets)) {
  $idx = [int]$a.ifIndex
  $guid = [string]$a.InterfaceGuid
  $v4m = Get-RegDns $guid $false
  $v6m = Get-RegDns $guid $true
  $v4 = Get-DnsClientServerAddress -InterfaceIndex $idx -AddressFamily IPv4 -ErrorAction SilentlyContinue
  $rows += [pscustomobject]@{
    interface_index = $idx
    interface_alias = [string]$a.Name
    interface_guid = $guid
    ipv4 = if ($v4m) { @(Split-List $v4m) } else { @($v4.ServerAddresses) }
    ipv6 = if ($v6m) { @(Split-List $v6m) } else { @() }
    ipv4_manual = [bool]$v4m
    ipv6_manual = [bool]$v6m
  }
}
ConvertTo-Json -InputObject @($rows) -Compress -Depth 4
"#;

pub(super) const APPLY: &str = r#"
$p = $env:ZAPRET_DNS_PAYLOAD | ConvertFrom-Json
$v4servers = @(@($p.ipv4) | Where-Object { $_ })
$v6servers = @(@($p.ipv6) | Where-Object { $_ })
$servers = @($v4servers + $v6servers)

$errs = @(); $n = 0
foreach ($a in @($p.adapters)) {
  $idx = Resolve-Index $a
  if ($idx -le 0) { continue }
  $n++
  try { Set-DnsClientServerAddress -InterfaceIndex $idx -ServerAddresses $servers | Out-Null }
  catch { $errs += ([string]$a.interface_alias + ': ' + $_.Exception.Message) }
}
if ($n -eq 0 -or $errs.Count -ge $n) { throw ('DNS не применён: ' + ($errs -join '; ')) }
if ($errs.Count -gt 0) { $warnings += ('DNS применён не на всех адаптерах: ' + ($errs -join '; ')) }

$added = @(); $modified = @(); $reg = @()
if ($p.doh) {
  $url = [string]$p.doh
  if (-not (Get-Command Add-DnsClientDohServerAddress -ErrorAction SilentlyContinue)) {
    $build = [Environment]::OSVersion.Version.Build
    $warnings += ('DoH не включён: в этой версии Windows (сборка ' + $build + ') нет системной поддержки DoH, нужна Windows 11. Обычный DNS применён.')
  } else {
    foreach ($ip in $servers) {
      try {
        $old = Get-DnsClientDohServerAddress -ServerAddress ([string]$ip) -ErrorAction SilentlyContinue
        if ($old) {
          $modified += [pscustomobject]@{
            server_address = [string]$ip
            doh_template = [string]$old.DohTemplate
            allow_fallback = [bool]$old.AllowFallbackToUdp
            auto_upgrade = [bool]$old.AutoUpgrade
          }
          Set-DnsClientDohServerAddress -ServerAddress ([string]$ip) -DohTemplate $url -AllowFallbackToUdp $true -AutoUpgrade $true | Out-Null
        } else {
          Add-DnsClientDohServerAddress -ServerAddress ([string]$ip) -DohTemplate $url -AllowFallbackToUdp $true -AutoUpgrade $true | Out-Null
          $added += [pscustomobject]@{ server_address = [string]$ip; doh_template = $url }
        }
        $reg += [string]$ip
      } catch { $warnings += ('DoH для ' + [string]$ip + ': ' + $_.Exception.Message) }
    }
    if ($reg.Count -eq 0) { $warnings += 'DoH не зарегистрирован ни для одного адреса.' }
  }
}
try { Clear-DnsClientCache | Out-Null } catch { $warnings += ('Не удалось очистить кэш DNS: ' + $_.Exception.Message) }
ConvertTo-Json -InputObject ([pscustomobject]@{ added = @($added); modified = @($modified); registered = @($reg); warnings = @($warnings) }) -Compress -Depth 4
"#;

pub(super) const PREPARE_DOH: &str = r#"
$p = $env:ZAPRET_DNS_PAYLOAD | ConvertFrom-Json
$added = @(); $modified = @()
if ($p.doh -and (Get-Command Add-DnsClientDohServerAddress -ErrorAction SilentlyContinue)) {
  foreach ($ip in @($p.ipv4) + @($p.ipv6)) {
    $old = Get-DnsClientDohServerAddress -ServerAddress ([string]$ip) -ErrorAction SilentlyContinue
    if ($old) {
      $modified += [pscustomobject]@{ server_address = [string]$ip; doh_template = [string]$old.DohTemplate; allow_fallback = [bool]$old.AllowFallbackToUdp; auto_upgrade = [bool]$old.AutoUpgrade }
    } else {
      $added += [pscustomobject]@{ server_address = [string]$ip; doh_template = [string]$p.doh }
    }
  }
}
ConvertTo-Json -InputObject ([pscustomobject]@{ added = @($added); modified = @($modified) }) -Compress -Depth 4
"#;

pub(super) const RESTORE: &str = r#"
$p = $env:ZAPRET_DNS_PAYLOAD | ConvertFrom-Json
$total = 0; $ok = 0
foreach ($a in @($p.adapters)) {
  $idx = Resolve-Index $a
  if ($idx -le 0) { $warnings += ('Адаптер не найден: ' + [string]$a.interface_alias); continue }
  $total++
  $m4 = [bool]$a.ipv4_manual
  $m6 = [bool]$a.ipv6_manual
  try {
    Set-DnsClientServerAddress -InterfaceIndex $idx -ResetServerAddresses | Out-Null
    $list = @(@(if ($m4) { $a.ipv4 }) + @(if ($m6) { $a.ipv6 }) | Where-Object { $_ })
    if ($list.Count -gt 0) {
      Set-DnsClientServerAddress -InterfaceIndex $idx -ServerAddresses $list | Out-Null
    }
    $ok++
  } catch { $warnings += ('Адаптер ' + [string]$a.interface_alias + ': ' + $_.Exception.Message) }
}
Remove-OurDoh $p
Clear-DnsClientCache | Out-Null
if ($total -eq 0 -or $ok -lt $total -or $warnings.Count -gt 0) { throw ('DNS восстановлен не полностью; снимок сохранён для повторной попытки: ' + ($warnings -join '; ')) }
ConvertTo-Json -InputObject ([pscustomobject]@{ warnings = @($warnings) }) -Compress
"#;

pub(super) const RESET: &str = r#"
$p = $env:ZAPRET_DNS_PAYLOAD | ConvertFrom-Json
$errs = @(); $n = 0
foreach ($a in @(Get-Targets)) {
  $n++
  try {
    Set-DnsClientServerAddress -InterfaceIndex ([int]$a.ifIndex) -ResetServerAddresses | Out-Null
    Clear-RegDns ([string]$a.InterfaceGuid)
    Clear-DohInterfaceSettings ([string]$a.InterfaceGuid)
  } catch { $errs += ([string]$a.Name + ': ' + $_.Exception.Message) }
}
if ($n -eq 0 -or $errs.Count -gt 0) { throw ('Сброс DNS не завершён; снимок сохранён: ' + ($errs -join '; ')) }
Remove-OurDoh $p
if ($warnings.Count -gt 0) { throw ('Не удалось восстановить DoH; снимок сохранён: ' + ($warnings -join '; ')) }
Clear-DnsClientCache | Out-Null
ConvertTo-Json -InputObject ([pscustomobject]@{ warnings = @($warnings) }) -Compress
"#;

pub(super) const DOH_LIST: &str = r#"
if (-not (Get-Command Get-DnsClientDohServerAddress -ErrorAction SilentlyContinue)) {
  ConvertTo-Json -InputObject @() -Compress
} else {
  $rows = @(Get-DnsClientDohServerAddress | ForEach-Object {
    [pscustomobject]@{ server_address = [string]$_.ServerAddress; doh_template = [string]$_.DohTemplate }
  })
  ConvertTo-Json -InputObject @($rows) -Compress
}
"#;
