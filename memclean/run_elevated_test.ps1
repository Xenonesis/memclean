$shell = New-Object -ComObject Shell.Application
$shell.ShellExecute('cmd.exe', '/c cd /d "C:\Users\Acer\Downloads\ram clean\memclean" && cargo test --test test_cleaner', '', 'runas', 1)
