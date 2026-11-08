; Inno Setup script for the Windows installer.
; Built by .github/workflows/release.yml:
;   iscc /DAppVersion=0.1.0 packaging\windows\pomolotl.iss
; Output: dist\pomolotl-setup.exe

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif

[Setup]
; Never change AppId: it's how upgrades/uninstall find the existing install.
AppId={{73852F4F-6F68-41B1-9A59-5D5E13805E15}
AppName=pomolotl
AppVersion={#AppVersion}
AppVerName=pomolotl {#AppVersion}
; File metadata (SignPath checks product name/version when signing).
VersionInfoVersion={#AppVersion}
VersionInfoProductName=pomolotl
VersionInfoProductVersion={#AppVersion}
AppPublisher=diegorezm
AppPublisherURL=https://github.com/diegorezm/pomolotl
AppSupportURL=https://github.com/diegorezm/pomolotl/issues
; Per-user install: no admin prompt.
PrivilegesRequired=lowest
DefaultDirName={autopf}\pomolotl
DisableProgramGroupPage=yes
DisableDirPage=auto
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir=..\..\dist
OutputBaseFilename=pomolotl-setup
SetupIconFile=..\..\assets\icon.ico
UninstallDisplayIcon={app}\pomolotl.exe
UninstallDisplayName=pomolotl
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ShowLanguageDialog=no
; We close pomolotl ourselves (see [Code]); it lives in the tray, which the
; Restart Manager doesn't handle well.
CloseApplications=no

[Languages]
; First entry is the fallback; the OS UI language picks the match automatically.
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[CustomMessages]
english.Privacy=Privacy
english.CheckUpdates=Check for new versions once a day (asks api.github.com for the latest version number; nothing about you is sent)
brazilianportuguese.Privacy=Privacidade
brazilianportuguese.CheckUpdates=Procurar novas versões uma vez por dia (pergunta ao api.github.com o número da versão mais nova; nada sobre você é enviado)

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "checkupdates"; Description: "{cm:CheckUpdates}"; GroupDescription: "{cm:Privacy}"; Check: IsFirstInstall

[Files]
Source: "..\..\target\release\pomolotl.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion

[Icons]
; AppUserModelID matches system.rs so notifications are grouped with the app.
Name: "{autoprograms}\pomolotl"; Filename: "{app}\pomolotl.exe"; AppUserModelID: "pomolotl.timer"
Name: "{autodesktop}\pomolotl"; Filename: "{app}\pomolotl.exe"; AppUserModelID: "pomolotl.timer"; Tasks: desktopicon

[Run]
Filename: "{app}\pomolotl.exe"; Description: "{cm:LaunchProgram,pomolotl}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\taskkill.exe"; Parameters: "/F /IM pomolotl.exe"; Flags: runhidden; RunOnceId: "StopPomolotl"

[Registry]
; Created by the app at runtime; only removed here on uninstall.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: none; ValueName: "pomolotl"; Flags: dontcreatekey uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\AppUserModelId\pomolotl.timer"; Flags: dontcreatekey uninsdeletekey

[Code]
// Stop a running pomolotl (it may be hidden in the tray) so its .exe can be replaced.
function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  Code: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM pomolotl.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  Result := '';
end;

// Config file the app creates on first launch.
function ConfigPath(): String;
begin
  Result := ExpandConstant('{userappdata}\pomolotl\config.toml');
end;

// Only offered on a first install; after that the setting lives in the app.
function IsFirstInstall(): Boolean;
begin
  Result := not FileExists(ConfigPath());
end;

// "Check for new versions" unchecked: start with the update check off.
// Settings missing from the file use their defaults.
procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and IsFirstInstall() and not WizardIsTaskSelected('checkupdates') then
  begin
    ForceDirectories(ExtractFileDir(ConfigPath()));
    SaveStringToFile(ConfigPath(), '[behavior]'#13#10'check_updates = false'#13#10, False);
  end;
end;
