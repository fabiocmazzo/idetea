; Script do Inno Setup 6 para gerar o instalador Windows da IDE TEA (IDE_TEA-<versão>-x86_64-setup.exe).
; Executado no GitHub Actions via: iscc /DAppVersion=<versão> packaging/windows/idetea.iss

#ifndef AppVersion
  #define AppVersion "0.5.0"
#endif

[Setup]
AppId={{D4B8E5C2-9A14-4F3B-8C7E-1D2E3F4A5B6C}
AppName=IDE TEA
AppVersion={#AppVersion}
AppVerName=IDE TEA {#AppVersion}
AppPublisher=Fabio Covolo Mazzo
AppPublisherURL=https://idetea.belabs.cloud
AppSupportURL=https://github.com/fabiocmazzo/idetea
AppUpdatesURL=https://github.com/fabiocmazzo/idetea/releases
DefaultDirName={autopf}\IDE TEA
DefaultGroupName=IDE TEA
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
LicenseFile=..\..\LICENSE
SetupIconFile=..\..\assets\idetea.ico
UninstallDisplayIcon={app}\idetea.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesEnvironment=yes
OutputDir=..\..\dist
OutputBaseFilename=IDE_TEA-{#AppVersion}-x86_64-setup

[Languages]
Name: "ptbr"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "es"; MessagesFile: "compiler:Languages\Spanish.isl"
Name: "fr"; MessagesFile: "compiler:Languages\French.isl"
Name: "de"; MessagesFile: "compiler:Languages\German.isl"
Name: "it"; MessagesFile: "compiler:Languages\Italian.isl"
Name: "ja"; MessagesFile: "compiler:Languages\Japanese.isl"
Name: "zh"; MessagesFile: "compiler:Languages\ChineseSimplified.isl"

[CustomMessages]
ptbr.AddToPath=Adicionar ao PATH (permite rodar "idetea ." no terminal)
en.AddToPath=Add to PATH (allows running "idetea ." from the terminal)
es.AddToPath=Añadir al PATH (permite ejecutar "idetea ." en la terminal)
fr.AddToPath=Ajouter au PATH (permet d'exécuter "idetea ." dans le terminal)
de.AddToPath=Zum PATH hinzufügen (damit "idetea ." im Terminal läuft)
it.AddToPath=Aggiungi al PATH (permette di eseguire "idetea ." nel terminale)
ja.AddToPath=PATH に追加する（ターミナルで "idetea ." を実行できます）
zh.AddToPath=添加到 PATH（即可在终端中运行 "idetea ."）
ptbr.ContextMenu=Adicionar "Abrir com IDE TEA" ao menu de contexto do Windows Explorer
en.ContextMenu=Add "Open with IDE TEA" to the Windows Explorer context menu
es.ContextMenu=Añadir "Abrir con IDE TEA" al menú contextual del Explorador de Windows
fr.ContextMenu=Ajouter « Ouvrir avec IDE TEA » au menu contextuel de l'Explorateur Windows
de.ContextMenu="Mit IDE TEA öffnen" zum Kontextmenü des Windows-Explorers hinzufügen
it.ContextMenu=Aggiungi "Apri con IDE TEA" al menu contestuale di Esplora file
ja.ContextMenu=エクスプローラーのコンテキスト メニューに「IDE TEA で開く」を追加する
zh.ContextMenu=在 Windows 资源管理器的右键菜单中添加“用 IDE TEA 打开”
ptbr.OpenWithIdeTea=Abrir com IDE TEA
en.OpenWithIdeTea=Open with IDE TEA
es.OpenWithIdeTea=Abrir con IDE TEA
fr.OpenWithIdeTea=Ouvrir avec IDE TEA
de.OpenWithIdeTea=Mit IDE TEA öffnen
it.OpenWithIdeTea=Apri con IDE TEA
ja.OpenWithIdeTea=IDE TEA で開く
zh.OpenWithIdeTea=用 IDE TEA 打开

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "contextmenu"; Description: "{cm:ContextMenu}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "addtopath"; Description: "{cm:AddToPath}"; GroupDescription: "{cm:AdditionalIcons}"

[Files]
Source: "..\..\dist\idetea-{#AppVersion}-x86_64-windows\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\IDE TEA"; Filename: "{app}\idetea.exe"; IconFilename: "{app}\idetea.ico"; Flags: runminimized
Name: "{autodesktop}\IDE TEA"; Filename: "{app}\idetea.exe"; IconFilename: "{app}\idetea.ico"; Tasks: desktopicon; Flags: runminimized

[Registry]
; Abrir arquivo com IDE TEA
Root: HKA; Subkey: "Software\Classes\*\shell\IDETEA"; ValueType: expandsz; ValueName: ""; ValueData: "{cm:OpenWithIdeTea}"; Tasks: contextmenu; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\*\shell\IDETEA"; ValueType: expandsz; ValueName: "Icon"; ValueData: "{app}\idetea.exe"; Tasks: contextmenu
Root: HKA; Subkey: "Software\Classes\*\shell\IDETEA\command"; ValueType: expandsz; ValueName: ""; ValueData: """{app}\idetea.exe"" ""%1"""; Tasks: contextmenu

; Abrir pasta com IDE TEA
Root: HKA; Subkey: "Software\Classes\Directory\shell\IDETEA"; ValueType: expandsz; ValueName: ""; ValueData: "{cm:OpenWithIdeTea}"; Tasks: contextmenu; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Directory\shell\IDETEA"; ValueType: expandsz; ValueName: "Icon"; ValueData: "{app}\idetea.exe"; Tasks: contextmenu
Root: HKA; Subkey: "Software\Classes\Directory\shell\IDETEA\command"; ValueType: expandsz; ValueName: ""; ValueData: """{app}\idetea.exe"" ""%1"""; Tasks: contextmenu

; Abrir fundo da pasta atual com IDE TEA
Root: HKA; Subkey: "Software\Classes\Directory\Background\shell\IDETEA"; ValueType: expandsz; ValueName: ""; ValueData: "{cm:OpenWithIdeTea}"; Tasks: contextmenu; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Directory\Background\shell\IDETEA"; ValueType: expandsz; ValueName: "Icon"; ValueData: "{app}\idetea.exe"; Tasks: contextmenu
Root: HKA; Subkey: "Software\Classes\Directory\Background\shell\IDETEA\command"; ValueType: expandsz; ValueName: ""; ValueData: """{app}\idetea.exe"" ""%V"""; Tasks: contextmenu

[Run]
Filename: "{app}\idetea.exe"; Description: "{cm:LaunchProgram,IDE TEA}"; Flags: nowait postinstall skipifsilent runminimized

[Code]
procedure AddAppToPath();
var
   EnvKey: string;
  Paths: string;
  AppDir: string;
begin
  AppDir := ExpandConstant('{app}');
  if IsAdminInstallMode() then
    EnvKey := 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment'
  else
    EnvKey := 'Environment';

  if not RegQueryStringValue(HKA, EnvKey, 'Path', Paths) then
    Paths := '';

  if Pos(';' + Uppercase(AppDir) + ';', ';' + Uppercase(Paths) + ';') = 0 then
  begin
    if (Length(Paths) > 0) and (Paths[Length(Paths)] <> ';') then
      Paths := Paths + ';';
    Paths := Paths + AppDir;
    RegWriteExpandStringValue(HKA, EnvKey, 'Path', Paths);
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if (CurStep = ssPostInstall) and WizardIsTaskSelected('addtopath') then
    AddAppToPath();
end;
