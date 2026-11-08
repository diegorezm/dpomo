# Smoke test of the Windows installer: silent install, launch, 1-minute focus
# via `pomolotl --toggle`, `pomolotl --quit`, then silent uninstall and cleanup check.
#   smoke-windows.ps1 <pomolotl-setup.exe> <out dir>
param([string]$Setup, [string]$Out)
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force $Out | Out-Null
$Out = (Resolve-Path $Out).Path
$cfg = "$env:APPDATA\pomolotl"
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
function Pomolotl($arg) { Start-Process $exe -ArgumentList $arg -Wait }

Start-Process $Setup -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -Wait
$exe = "$env:LOCALAPPDATA\Programs\pomolotl\pomolotl.exe"
if (-not (Test-Path $exe)) { throw "not installed at $exe" }

# stderr shows which renderer started (CI runners have no OpenGL → Direct3D 12 fallback).
$p = Start-Process $exe -PassThru -RedirectStandardError "$Out\pomolotl.log"
Start-Sleep 8
Shot '1-started'
if ($p.HasExited) { Get-Content "$Out\pomolotl.log"; throw 'pomolotl exited at startup' }
"working set after startup: {0:N1} MB" -f ((Get-Process -Id $p.Id).WorkingSet64 / 1MB)

Pomolotl '--toggle'
Start-Sleep 5
Shot '2-running'
Start-Sleep 60
Shot '3-break-started'

Pomolotl '--quit'
Start-Sleep 3
if (-not $p.HasExited) { throw 'pomolotl --quit did not exit' }
Copy-Item "$cfg\state.toml" $Out
if (-not (Select-String -Path "$cfg\state.toml" -Pattern '^sessions = 1' -Quiet)) { throw 'focus session not recorded' }
if (-not (Test-Path 'HKCU:\Software\Classes\AppUserModelId\pomolotl.timer')) { throw 'toast AUMID not registered' }

Start-Process "$env:LOCALAPPDATA\Programs\pomolotl\unins000.exe" -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES' -Wait
Start-Sleep 3
if (Test-Path $exe) { throw 'uninstall left pomolotl.exe behind' }
if (Test-Path 'HKCU:\Software\Classes\AppUserModelId\pomolotl.timer') { throw 'uninstall left the AUMID key' }
'smoke test passed'
