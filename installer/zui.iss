; Compile with Inno Setup 7. Cargo is not invoked by this script.
#ifndef RepoRoot
  #define RepoRoot SourcePath + ".."
#endif
#ifndef MyAppVersion
  #define MyAppVersion "0.1.1"
#endif
#ifndef MyAppExe
  #if FileExists(RepoRoot + "\target\x86_64-pc-windows-msvc\release\zUI.exe")
    #define MyAppExe RepoRoot + "\target\x86_64-pc-windows-msvc\release\zUI.exe"
  #else
    #define MyAppExe RepoRoot + "\target\release\zUI.exe"
  #endif
#endif

[Setup]
AppId=zUI
AppName=zUI
AppVersion={#MyAppVersion}
AppVerName=zUI {#MyAppVersion} Lyra
AppPublisher=maybepritz
DefaultDirName={localappdata}\zUI
DefaultGroupName=zUI
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
WizardStyle=modern dark hidebevels includetitlebar
WizardBackColor=$202020
DisableWelcomePage=no
DisableDirPage=yes
DisableProgramGroupPage=yes
DisableReadyPage=yes
DisableFinishedPage=yes
SetupIconFile={#RepoRoot}\crates\app\assets\icon.ico
UninstallDisplayIcon={app}\zUI.exe
WizardSmallImageFile={#RepoRoot}\installer\assets\app-logo.png
Compression=lzma2
SolidCompression=yes
OutputDir={#RepoRoot}\dist
OutputBaseFilename=zUI-{#MyAppVersion}-Lyra-Setup
CloseApplications=yes
RestartApplications=no
UninstallDisplayName=zUI

[Languages]
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[LangOptions]
DialogFontName=Segoe UI
DialogFontSize=9

[Files]
Source: "{#MyAppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#RepoRoot}\LICENSE"; DestDir: "{app}"
Source: "{#RepoRoot}\installer\assets\app-logo.png"; Flags: dontcopy

[Icons]
Name: "{group}\zUI"; Filename: "{app}\zUI.exe"; WorkingDir: "{app}"
Name: "{userdesktop}\zUI"; Filename: "{app}\zUI.exe"; WorkingDir: "{app}"; Check: CreateDesktopShortcut

[Run]
Filename: "{app}\zUI.exe"; WorkingDir: "{app}"; Flags: nowait skipifsilent; Check: LaunchAfterInstall

[Code]
const
  BackgroundColor = $202020;
  SurfaceColor = $292929;
  BorderColor = $373737;
  TextColor = $DADADA;
  MutedColor = $A4A4A4;

var
  DesktopCheck, LaunchCheck: TNewCheckBox;
  InstallButton: TBitmapButton;
  WelcomeBackground: TBitmapImage;

function TextLabel(Parent: TWinControl; X, Y, W, H, Size: Integer;
  Color: TColor; Text: String; Bold: Boolean): TNewStaticText;
begin
  Result := TNewStaticText.Create(WizardForm);
  Result.Parent := Parent;
  Result.StyleElements := [];
  Result.SetBounds(ScaleX(X), ScaleY(Y), ScaleX(W), ScaleY(H));
  Result.AutoSize := False;
  Result.WordWrap := True;
  Result.Font.Name := 'Segoe UI';
  Result.Font.Size := Size;
  Result.Font.Color := Color;
  Result.ParentBackground := False;
  Result.Color := BackgroundColor;
  if Bold then Result.Font.Style := [fsBold];
  Result.Caption := Text;
end;

procedure RoundedSurface(Image: TBitmapImage; X, Y, W, H: Integer);
begin
  Image.Bitmap.Canvas.Brush.Color := SurfaceColor;
  Image.Bitmap.Canvas.Pen.Color := BorderColor;
  Image.Bitmap.Canvas.RoundRect(ScaleX(X), ScaleY(Y), ScaleX(X + W),
    ScaleY(Y + H), ScaleX(18), ScaleY(18));
end;

function CreateDesktopShortcut: Boolean;
begin
  Result := DesktopCheck.Checked;
end;

function LaunchAfterInstall: Boolean;
begin
  Result := LaunchCheck.Checked;
end;

procedure UpdateInstallButton(Sender: TObject);
var
  Caption: String;
  X, Y: Integer;
begin
  if LaunchCheck.Checked then
    Caption := 'Установить и запустить'
  else
    Caption := 'Установить';
  WizardForm.NextButton.Caption := Caption;
  InstallButton.Caption := Caption;
  InstallButton.Bitmap.Width := InstallButton.Width;
  InstallButton.Bitmap.Height := InstallButton.Height;
  InstallButton.Bitmap.Canvas.Brush.Color := BackgroundColor;
  InstallButton.Bitmap.Canvas.Pen.Color := BackgroundColor;
  InstallButton.Bitmap.Canvas.Rectangle(0, 0, InstallButton.Width, InstallButton.Height);
  InstallButton.Bitmap.Canvas.Brush.Color := $DEDEDE;
  InstallButton.Bitmap.Canvas.Pen.Color := $DEDEDE;
  InstallButton.Bitmap.Canvas.RoundRect(0, 0, InstallButton.Width, InstallButton.Height,
    ScaleX(16), ScaleY(16));
  InstallButton.Bitmap.Canvas.Font.Name := 'Segoe UI';
  InstallButton.Bitmap.Canvas.Font.Size := 10;
  InstallButton.Bitmap.Canvas.Font.Style := [fsBold];
  InstallButton.Bitmap.Canvas.Font.Color := $242424;
  X := (InstallButton.Width - InstallButton.Bitmap.Canvas.TextWidth(Caption)) div 2;
  Y := (InstallButton.Height - InstallButton.Bitmap.Canvas.TextHeight(Caption)) div 2;
  InstallButton.Bitmap.Canvas.TextOut(X, Y, Caption);
  InstallButton.Invalidate;
end;

procedure InstallClick(Sender: TObject);
begin
  { Delegate navigation and installation to Inno's original action. }
  if (WizardForm.CurPageID = wpWelcome) and WizardForm.NextButton.Enabled then
    WizardForm.NextButton.OnClick(WizardForm.NextButton);
end;

procedure WizardKeyDown(Sender: TObject; var Key: Word; Shift: TShiftState);
begin
  if (WizardForm.CurPageID = wpWelcome) and (Key = 13)
    and (WizardForm.ActiveControl <> WizardForm.CancelButton) then begin
    Key := 0;
    InstallClick(Sender);
  end;
end;

procedure InitializeWizard;
var
  Image: TBitmapImage;
  LabelControl: TNewStaticText;
begin
  WizardForm.Caption := 'Установка zUI';
  WizardForm.StyleElements := [seBorder];
  WizardForm.Color := BackgroundColor;
  WizardForm.Font.Name := 'Segoe UI';
  WizardForm.Font.Size := 9;
  WizardForm.Font.Color := TextColor;
  WizardForm.ClientWidth := ScaleX(560);
  WizardForm.ClientHeight := ScaleY(480);
  WizardForm.OuterNotebook.SetBounds(0, 0, WizardForm.ClientWidth, ScaleY(408));
  WizardForm.WelcomePage.StyleElements := [];
  WizardForm.WelcomePage.Color := BackgroundColor;
  WizardForm.InnerPage.StyleElements := [];
  WizardForm.InnerPage.Color := BackgroundColor;
  WizardForm.InstallingPage.StyleElements := [];
  WizardForm.InstallingPage.Color := BackgroundColor;
  WizardForm.MainPanel.StyleElements := [];
  WizardForm.MainPanel.Color := BackgroundColor;
  WizardForm.Bevel.Visible := False;
  WizardForm.Bevel1.Visible := False;
  WizardForm.WelcomeLabel1.Visible := False;
  WizardForm.WelcomeLabel2.Visible := False;
  WizardForm.WizardBitmapImage.Visible := False;

  WelcomeBackground := TBitmapImage.Create(WizardForm);
  WelcomeBackground.Parent := WizardForm.WelcomePage;
  WelcomeBackground.SetBounds(0, 0, ScaleX(560), ScaleY(408));
  WelcomeBackground.Bitmap.Width := WelcomeBackground.Width;
  WelcomeBackground.Bitmap.Height := WelcomeBackground.Height;
  WelcomeBackground.Bitmap.Canvas.Brush.Color := BackgroundColor;
  WelcomeBackground.Bitmap.Canvas.Pen.Color := BackgroundColor;
  WelcomeBackground.Bitmap.Canvas.Rectangle(0, 0, WelcomeBackground.Width, WelcomeBackground.Height);
  RoundedSurface(WelcomeBackground, 28, 174, 504, 68);
  RoundedSurface(WelcomeBackground, 28, 256, 504, 100);
  WelcomeBackground.Bitmap.Canvas.Pen.Color := BorderColor;
  WelcomeBackground.Bitmap.Canvas.MoveTo(ScaleX(44), ScaleY(306));
  WelcomeBackground.Bitmap.Canvas.LineTo(ScaleX(516), ScaleY(306));

  ExtractTemporaryFile('app-logo.png');
  Image := TBitmapImage.Create(WizardForm);
  Image.Parent := WizardForm.WelcomePage;
  Image.SetBounds(ScaleX(28), ScaleY(24), ScaleX(56), ScaleY(56));
  Image.Stretch := True;
  Image.PngImage.LoadFromFile(ExpandConstant('{tmp}\app-logo.png'));

  LabelControl := TextLabel(WizardForm.WelcomePage, 104, 22, 340, 36, 23, TextColor, 'zUI', True);
  LabelControl := TextLabel(WizardForm.WelcomePage, 106, 64, 340, 20, 9, MutedColor,
    'Lyra  /  {#MyAppVersion}', False);
  LabelControl := TextLabel(WizardForm.WelcomePage, 28, 110, 504, 26, 13, TextColor,
    'Всё готово к установке', True);
  LabelControl := TextLabel(WizardForm.WelcomePage, 28, 140, 504, 24, 9, MutedColor,
    'Приложение будет доступно в меню «Пуск».', False);
  LabelControl := TextLabel(WizardForm.WelcomePage, 44, 186, 472, 20, 9, MutedColor,
    'Папка установки', False);
  LabelControl.Color := SurfaceColor;
  LabelControl := TextLabel(WizardForm.WelcomePage, 44, 211, 472, 24, 8, TextColor,
    WizardForm.DirEdit.Text, False);
  LabelControl.Color := SurfaceColor;

  DesktopCheck := TNewCheckBox.Create(WizardForm);
  DesktopCheck.Parent := WizardForm.WelcomePage;
  DesktopCheck.SetBounds(ScaleX(44), ScaleY(269), ScaleX(472), ScaleY(24));
  DesktopCheck.StyleElements := [seClient, seBorder];
  DesktopCheck.Font.Name := 'Segoe UI';
  DesktopCheck.Font.Size := 9;
  DesktopCheck.Font.Color := TextColor;
  DesktopCheck.Color := SurfaceColor;
  DesktopCheck.ParentBackground := False;
  DesktopCheck.Caption := 'Создать ярлык на рабочем столе';
  DesktopCheck.Checked := False;

  LaunchCheck := TNewCheckBox.Create(WizardForm);
  LaunchCheck.Parent := WizardForm.WelcomePage;
  LaunchCheck.SetBounds(ScaleX(44), ScaleY(318), ScaleX(472), ScaleY(24));
  LaunchCheck.StyleElements := [seClient, seBorder];
  LaunchCheck.Font.Name := 'Segoe UI';
  LaunchCheck.Font.Size := 9;
  LaunchCheck.Font.Color := TextColor;
  LaunchCheck.Color := SurfaceColor;
  LaunchCheck.ParentBackground := False;
  LaunchCheck.Caption := 'Запустить zUI после установки';
  LaunchCheck.Checked := True;
  LaunchCheck.OnClick := @UpdateInstallButton;

  LabelControl := TextLabel(WizardForm.WelcomePage, 28, 374, 504, 24, 8, MutedColor,
    'Zapret и Telegram подключаются внутри zUI.', False);

  WizardForm.CancelButton.SetBounds(ScaleX(28), ScaleY(426), ScaleX(88), ScaleY(38));
  WizardForm.CancelButton.Caption := 'Отмена';
  WizardForm.NextButton.SetBounds(ScaleX(300), ScaleY(422), ScaleX(232), ScaleY(42));
  InstallButton := TBitmapButton.Create(WizardForm);
  InstallButton.Parent := WizardForm;
  InstallButton.StyleElements := [];
  InstallButton.SetBounds(ScaleX(300), ScaleY(422), ScaleX(232), ScaleY(42));
  InstallButton.AutoSize := False;
  InstallButton.BackColor := BackgroundColor;
  InstallButton.TabStop := True;
  InstallButton.Cursor := crHandPoint;
  InstallButton.OnClick := @InstallClick;
  WizardForm.KeyPreview := True;
  WizardForm.OnKeyDown := @WizardKeyDown;
  WizardForm.FilenameLabel.Visible := False;
end;

procedure CurPageChanged(CurPageID: Integer);
begin
  WizardForm.BackButton.Visible := False;
  InstallButton.Visible := (CurPageID = wpWelcome);
  if CurPageID = wpWelcome then begin
    WizardForm.NextButton.Visible := False;
    UpdateInstallButton(nil);
    WizardForm.ActiveControl := InstallButton;
  end;
end;

function ShouldSkipPage(PageID: Integer): Boolean;
begin
  Result := (PageID = wpSelectTasks);
end;
