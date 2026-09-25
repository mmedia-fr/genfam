; Installeur Windows de GenFam (Inno Setup 6).
;
; Compilation : ISCC.exe /DAppVersion=8.0.1 build\genfam.iss
; Prérequis   : dist\GenFam\ contient GenFam.exe et les bibliothèques Qt (windeployqt).
; Sortie      : dist\GenFam-<version>-setup.exe

#ifndef AppVersion
  #define AppVersion "8.0.1"
#endif
#define AppName        "GenFam"
#define AppLongName    "GenFam — Généalogie"
#define AppPublisher   "Claude Boisseau / M-Media"
#define AppExe         "GenFam.exe"
#define ProgId         "MMedia.GenFam.1"

[Setup]
AppId={{2498CDDB-65BE-4B56-88BC-4178D0C38947}
AppName={#AppLongName}
AppVersion={#AppVersion}
AppVerName={#AppLongName} {#AppVersion}
AppPublisher={#AppPublisher}
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppLongName} {#AppVersion}
OutputDir=..\dist
OutputBaseFilename={#AppName}-{#AppVersion}-setup
SetupIconFile=..\assets\genfam.ico
; Licence présentée à l'installation (exigence morale du copyleft : l'utilisateur
; doit savoir sous quels termes il reçoit le programme).
LicenseFile=..\LICENSE
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
; Installation dans le profil sans élévation ; l'assistant propose l'installation
; pour tous les utilisateurs si l'on dispose des droits d'administration.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=commandline dialog
ArchitecturesInstallIn64BitMode=x64compatible
ArchitecturesAllowed=x64compatible
DisableProgramGroupPage=yes
ShowLanguageDialog=no

[Languages]
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "Créer un raccourci sur le &Bureau"; GroupDescription: "Raccourcis :"
Name: "associer"; Description: "Ouvrir les fichiers .ged avec {#AppName}"; GroupDescription: "Intégration à Windows :"

[Files]
Source: "..\dist\GenFam\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "..\README.md";     DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE";       DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppLongName}"; Filename: "{app}\{#AppExe}"
Name: "{autodesktop}\{#AppLongName}";  Filename: "{app}\{#AppExe}"; Tasks: desktopicon

[Registry]
Root: HKA; Subkey: "Software\Classes\{#ProgId}"; ValueType: string; ValueName: ""; ValueData: "Fichier généalogique GEDCOM"; Flags: uninsdeletekey; Tasks: associer
Root: HKA; Subkey: "Software\Classes\{#ProgId}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\{#AppExe},0"; Tasks: associer
Root: HKA; Subkey: "Software\Classes\{#ProgId}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; Tasks: associer
Root: HKA; Subkey: "Software\Classes\.ged\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue; Tasks: associer
Root: HKA; Subkey: "Software\Classes\Applications\{#AppExe}"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "{#AppName}"; Flags: uninsdeletekey; Tasks: associer
Root: HKA; Subkey: "Software\Classes\Applications\{#AppExe}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""; Tasks: associer

[Run]
Filename: "{app}\{#AppExe}"; Description: "Lancer {#AppName}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
Type: dirifempty; Name: "{app}"
