# Smoke test of the Windows installer: silent install, launch, 1-minute focus
# via `dpomo --toggle`, `dpomo --quit`, then silent uninstall and cleanup check.
#   smoke-windows.ps1 <dpomo-setup.exe> <out dir>
param([string]$Setup, [string]$Out)
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path $Out).Path
$cfg = "$env:APPDATA\dpomo"
New-Item -ItemType Directory -Force $cfg | Out-Null
Copy-Item "$PSScriptRoot\config.toml" "$cfg\config.toml"

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
function Shot($name) {
    try {
        $b = [Windows.Forms.Screen]::PrimaryScreen.Bounds
        $bmp = New-Object Drawing.Bitmap $b.Width, $b.Height
        [Drawing.Graphics]::FromImage($bmp).CopyFromScreen($b.Location, [Drawing.Point]::Empty, $b.Size)
        $bmp.Save("$Out\$name.png")
    } catch { "screenshot failed: $_" }
}
function Dpomo($arg) { Start-Process $exe -ArgumentList $arg -Wait }

Start-Process $Setup -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -Wait
$exe = "$env:LOCALAPPDATA\Programs\dpomo\dpomo.exe"
if (-not (Test-Path $exe)) { throw "not installed at $exe" }

$p = Start-Process $exe -PassThru
Start-Sleep 8
Shot '1-started'
if ($p.HasExited) { throw 'dpomo exited at startup' }
"working set after startup: {0:N1} MB" -f ((Get-Process -Id $p.Id).WorkingSet64 / 1MB)

Dpomo '--toggle'
Start-Sleep 5
Shot '2-running'
Start-Sleep 60
Shot '3-break-started'

Dpomo '--quit'
Start-Sleep 3
if (-not $p.HasExited) { throw 'dpomo --quit did not exit' }
Copy-Item "$cfg\state.toml" $Out
if (-not (Select-String -Path "$cfg\state.toml" -Pattern '^sessions = 1' -Quiet)) { throw 'focus session not recorded' }
if (-not (Test-Path 'HKCU:\Software\Classes\AppUserModelId\dpomo.timer')) { throw 'toast AUMID not registered' }

Start-Process "$env:LOCALAPPDATA\Programs\dpomo\unins000.exe" -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES' -Wait
Start-Sleep 3
if (Test-Path $exe) { throw 'uninstall left dpomo.exe behind' }
if (Test-Path 'HKCU:\Software\Classes\AppUserModelId\dpomo.timer') { throw 'uninstall left the AUMID key' }
'smoke test passed'
