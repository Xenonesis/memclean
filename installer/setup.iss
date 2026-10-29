[Setup]
AppId={{9F57C02D-2A9F-4D2A-B981-D189873E4A10}}
AppName=MemClean
AppVersion=1.0.0
AppPublisher=MemClean
DefaultDirName={autopf}\MemClean
DefaultGroupName=MemClean
AllowNoIcons=yes
OutputDir=..\dist
OutputBaseFilename=MemClean_Setup
Compression=lzma2/ultra64
SolidCompression=yes
PrivilegesRequired=admin
CloseApplications=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "autostart"; Description: "Run MemClean automatically on Windows startup (Silent Elevation)"; GroupDescription: "Startup:"

[Files]
Source: "..\memclean\target\release\memclean.exe"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\MemClean"; Filename: "{app}\memclean.exe"
Name: "{group}\Uninstall MemClean"; Filename: "{uninstallexe}"
Name: "{autodesktop}\MemClean"; Filename: "{app}\memclean.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\memclean.exe"; Parameters: "--autostart"; Tasks: autostart; Flags: runhidden
Filename: "{app}\memclean.exe"; Description: "{cm:LaunchProgram,MemClean}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{app}\memclean.exe"; Parameters: "--no-autostart"; Flags: runhidden
