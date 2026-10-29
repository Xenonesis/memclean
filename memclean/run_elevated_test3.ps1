param($batFile = "C:\Users\Acer\Downloads\ram clean\memclean\run_elevated_test3.bat")
$encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes("cmd.exe /c " + $batFile))
Start-Process -FilePath "powershell.exe" -ArgumentList "-NoProfile -ExecutionPolicy Bypass -EncodedCommand $encoded" -Verb RunAs -Wait
