$shell = New-Object -ComObject Shell.Application
$shell.ShellExecute('cmd.exe', '/c C:\Users/Acer/Downloads/ram clean/memclean/run_elevated_test2.bat', '', 'runas', 1)
Start-Sleep -Seconds 10
