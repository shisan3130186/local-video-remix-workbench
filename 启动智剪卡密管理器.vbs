Option Explicit
Dim shell, fso, rootDir, launcher, command
Set shell = CreateObject("WScript.Shell")
Set fso = CreateObject("Scripting.FileSystemObject")
rootDir = fso.GetParentFolderName(WScript.ScriptFullName)
launcher = fso.BuildPath(rootDir, "services\membership-api\admin-manager\launch-manager.vbs")
command = "wscript.exe """ & launcher & """"
shell.Run command, 0, False
