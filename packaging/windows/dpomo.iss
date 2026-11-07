; Inno Setup script for the Windows installer.
; Built by .github/workflows/release.yml:
;   iscc /DAppVersion=0.1.0 packaging\windows\dpomo.iss
; Output: dist\dpomo-setup.exe

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
; Never change AppId: it's how upgrades/uninstall find the existing install.
AppId={{8C3F1C2A-6E7B-4D7E-9B1A-5D2F0A9C4E11}
AppName=dpomo
AppVersion={#AppVersion}
AppVerName=dpomo {#AppVersion}
AppPublisher=diegorezm
AppPublisherURL=https://github.com/diegorezm/dpomo
AppSupportURL=https://github.com/diegorezm/dpomo/issues
; Per-user install: no admin prompt.
PrivilegesRequired=lowest
DefaultDirName={autopf}\dpomo
DisableProgramGroupPage=yes
DisableDirPage=auto
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\..\dist
OutputBaseFilename=dpomo-setup
SetupIconFile=..\..\assets\icon.ico
UninstallDisplayIcon={app}\dpomo.exe
UninstallDisplayName=dpomo
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ShowLanguageDialog=no
; We close dpomo ourselves (see [Code]); it lives in the tray, which the
; Restart Manager doesn't handle well.
CloseApplications=no

[Languages]
; First entry is the fallback; the OS UI language picks the match automatically.
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\target\release\dpomo.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion

[Icons]
; AppUserModelID matches system.rs so notifications are grouped with the app.
Name: "{autoprograms}\dpomo"; Filename: "{app}\dpomo.exe"; AppUserModelID: "dpomo.timer"
Name: "{autodesktop}\dpomo"; Filename: "{app}\dpomo.exe"; AppUserModelID: "dpomo.timer"; Tasks: desktopicon

[Run]
Filename: "{app}\dpomo.exe"; Description: "{cm:LaunchProgram,dpomo}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM dpomo.exe"; Flags: runhidden; RunOnceId: "StopDpomo"

[Registry]
; Created by the app at runtime; only removed here on uninstall.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: none; ValueName: "dpomo"; Flags: dontcreatekey uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\AppUserModelId\dpomo.timer"; Flags: dontcreatekey uninsdeletekey

[Code]
// Stop a running dpomo (it may be hidden in the tray) so its .exe can be replaced.
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Code: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM dpomo.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  Result := '';
end;
