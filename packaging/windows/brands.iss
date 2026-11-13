; Per-user installations, with distinct upgrade identities and no default-app takeover.
#ifndef Product
  #error Product must be HasiSolid or MSSolid
#endif
#if Product == "HasiSolid"
  #define ProductGuid "52bf2150-987c-4913-8d40-f1cb3f421921"
  #define Publisher "Hasi Elektronic"
  #define BrandFolder "hasisolid"
  #define ModelId "de.hasi-elektronic.hasisolid"
#elif Product == "MSSolid"
  #define ProductGuid "88b23342-b960-4386-b251-9db3dd992496"
  #define Publisher "Manfred Sickinger GmbH"
  #define BrandFolder "mssolid"
  #define ModelId "de.manfred-sickinger.mssolid"
#else
  #error Unknown Product
#endif

[Setup]
AppId={{{#ProductGuid}}
AppName={#Product}
AppVersion={#Version}
AppPublisher={#Publisher}
VersionInfoProductName={#Product}
VersionInfoDescription={#Product} Setup
VersionInfoVersion={#Version}.0
DefaultDirName={localappdata}\Programs\{#Product}
DisableDirPage=yes
UsePreviousAppDir=no
DefaultGroupName={#Product}
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0.17763
OutputDir={#OutputDirectory}
OutputBaseFilename={#Product}-{#Version}-Setup-x64
SetupIconFile=..\..\assets\brands\{#BrandFolder}\app.ico
UninstallDisplayIcon={app}\{#Product}.exe
UninstallDisplayName={#Product}
WizardStyle=modern
Compression=lzma2
SolidCompression=yes
DisableProgramGroupPage=yes
CloseApplications=yes
CloseApplicationsFilter={#Product}.exe
RestartApplications=no
ChangesAssociations=yes
LicenseFile={#Stage}\LICENSE-MIT

[Languages]
Name: "german"; MessagesFile: "compiler:Languages\German.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[CustomMessages]
german.DowngradeBlocked=Eine neuere oder nicht überprüfbare {#Product}-Version ist bereits installiert. Die Installation wurde zum Schutz dieser Version abgebrochen.
english.DowngradeBlocked=A newer or unverifiable {#Product} version is already installed. Setup stopped to protect that installation.
german.DirectoryBlocked=Der vorgesehene {#Product}-Installationsordner ist belegt oder entspricht nicht der vorhandenen Installation. Die Installation wurde zum Schutz der vorhandenen Dateien abgebrochen.
english.DirectoryBlocked=The intended {#Product} installation folder is occupied or differs from the existing installation. Setup stopped to protect existing files.

[Registry]
; OpenWith entries do not change UserChoice, extension defaults or preview handlers.
Root: HKCU; Subkey: "Software\Classes\{#Product}.SolidCraft"; ValueType: string; ValueData: "{#Product} 3D CAD"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\{#Product}.SolidCraft\DefaultIcon"; ValueType: string; ValueData: """{app}\{#Product}.exe"",0"
Root: HKCU; Subkey: "Software\Classes\{#Product}.SolidCraft\shell\open\command"; ValueType: string; ValueData: """{app}\{#Product}.exe"" ""%1"""
Root: HKCU; Subkey: "Software\Classes\.sc\OpenWithProgids"; ValueType: string; ValueName: "{#Product}.SolidCraft"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.solidcraft\OpenWithProgids"; ValueType: string; ValueName: "{#Product}.SolidCraft"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.solvecraft\OpenWithProgids"; ValueType: string; ValueName: "{#Product}.SolidCraft"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.scdwg\OpenWithProgids"; ValueType: string; ValueName: "{#Product}.SolidCraft"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.step\OpenWithProgids"; ValueType: string; ValueName: "{#Product}.SolidCraft"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\.stp\OpenWithProgids"; ValueType: string; ValueName: "{#Product}.SolidCraft"; ValueData: ""; Flags: uninsdeletevalue
Root: HKCU; Subkey: "Software\{#ModelId}\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "{#Product}"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\{#ModelId}\Capabilities"; ValueType: string; ValueName: "ApplicationIcon"; ValueData: """{app}\{#Product}.exe"",0"
Root: HKCU; Subkey: "Software\{#ModelId}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".sc"; ValueData: "{#Product}.SolidCraft"
Root: HKCU; Subkey: "Software\{#ModelId}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".solidcraft"; ValueData: "{#Product}.SolidCraft"
Root: HKCU; Subkey: "Software\{#ModelId}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".solvecraft"; ValueData: "{#Product}.SolidCraft"
Root: HKCU; Subkey: "Software\{#ModelId}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".scdwg"; ValueData: "{#Product}.SolidCraft"
Root: HKCU; Subkey: "Software\{#ModelId}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".step"; ValueData: "{#Product}.SolidCraft"
Root: HKCU; Subkey: "Software\{#ModelId}\Capabilities\FileAssociations"; ValueType: string; ValueName: ".stp"; ValueData: "{#Product}.SolidCraft"
Root: HKCU; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "{#Product}"; ValueData: "Software\{#ModelId}\Capabilities"; Flags: uninsdeletevalue

[Files]
Source: "{#Stage}\{#Product}.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\{#Product}-cli.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\LICENSE-MIT"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\LICENSE-APACHE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\NOTICE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\THIRD-PARTY-LICENSES.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\BRAND-LICENSE.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#Stage}\README.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#Product}"; Filename: "{app}\{#Product}.exe"; AppUserModelID: "{#ModelId}"
Name: "{autodesktop}\{#Product}"; Filename: "{app}\{#Product}.exe"; Tasks: desktopicon; AppUserModelID: "{#ModelId}"

[Run]
Filename: "{app}\{#Product}.exe"; Description: "{cm:LaunchProgram,{#Product}}"; Flags: nowait postinstall skipifsilent

[Code]
function SafeDirectoryPath(const Directory: String): Boolean;
var
  Current, Parent, Root, OptionalParent: String;
  Entry: TFindRec;
begin
  Result := False;
  Current := RemoveBackslashUnlessRoot(ExpandFileName(Directory));
  OptionalParent := RemoveBackslashUnlessRoot(ExtractFileDir(Current));
  Root := AddBackslash(ExtractFileDrive(Current));
  while CompareText(Current, Root) <> 0 do begin
    if FindFirst(Current, Entry) then begin
      try
        if ((Entry.Attributes and FILE_ATTRIBUTE_REPARSE_POINT) <> 0) or
           ((Entry.Attributes and FILE_ATTRIBUTE_DIRECTORY) = 0) then Exit;
      finally
        FindClose(Entry);
      end;
    end else if (CompareText(Current, Directory) <> 0) and (CompareText(Current, OptionalParent) <> 0) then Exit;
    Parent := RemoveBackslashUnlessRoot(ExtractFileDir(Current));
    if (Parent = '') or (CompareText(Parent, Current) = 0) then Exit;
    Current := Parent;
  end;
  Result := True;
end;

function SafeToInstallAt(const Directory: String; const DesiredVersion: Int64): Boolean;
var
  ExistingVersion: Int64;
  Exe: String;
begin
  Result := True;
  if Directory = '' then Exit;
  Exe := AddBackslash(Directory) + '{#Product}.exe';
  if not FileExists(Exe) then Exit;
  if not GetPackedVersion(Exe, ExistingVersion) then begin
    Result := False;
    Exit;
  end;
  Result := ComparePackedVersion(ExistingVersion, DesiredVersion) <= 0;
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
var
  DesiredVersion: Int64;
  PreviousDirectory: String;
  TargetDirectory: String;
  ExpectedDirectory: String;
  UninstallKey: String;
begin
  Result := '';
  TargetDirectory := RemoveBackslashUnlessRoot(ExpandFileName(ExpandConstant('{app}')));
  ExpectedDirectory := RemoveBackslashUnlessRoot(ExpandFileName(ExpandConstant('{localappdata}\Programs\{#Product}')));
  if CompareText(TargetDirectory, ExpectedDirectory) <> 0 then begin
    Result := CustomMessage('DirectoryBlocked');
    Exit;
  end;
  if not SafeDirectoryPath(TargetDirectory) then begin
    Result := CustomMessage('DirectoryBlocked');
    Exit;
  end;
  if not StrToVersion('{#Version}.0', DesiredVersion) then begin
    Result := CustomMessage('DowngradeBlocked');
    Exit;
  end;
  UninstallKey := 'Software\Microsoft\Windows\CurrentVersion\Uninstall\' + '{' + '{#ProductGuid}' + '}_is1';
  PreviousDirectory := '';
  if RegKeyExists(HKCU, UninstallKey) then begin
    if not RegQueryStringValue(HKCU, UninstallKey, 'InstallLocation', PreviousDirectory) or (Trim(PreviousDirectory) = '') then
      RegQueryStringValue(HKCU, UninstallKey, 'Inno Setup: App Path', PreviousDirectory);
    PreviousDirectory := RemoveQuotes(Trim(PreviousDirectory));
    if (PreviousDirectory = '') or
       (CompareText(RemoveBackslashUnlessRoot(ExpandFileName(PreviousDirectory)), TargetDirectory) <> 0) or
       not FileExists(AddBackslash(PreviousDirectory) + '{#Product}.exe') then begin
      Result := CustomMessage('DowngradeBlocked');
      Exit;
    end;
  end else if DirExists(TargetDirectory) or FileExists(TargetDirectory) then begin
    Result := CustomMessage('DirectoryBlocked');
    Exit;
  end;
  if not SafeToInstallAt(PreviousDirectory, DesiredVersion) or
     not SafeToInstallAt(ExpandConstant('{app}'), DesiredVersion) then
    Result := CustomMessage('DowngradeBlocked');
end;
