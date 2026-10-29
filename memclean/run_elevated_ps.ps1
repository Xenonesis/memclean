param([string]$cargoPath = "C:\Users\Acer\.cargo\bin\cargo.exe")

$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if ($isAdmin) {
    Set-Location "C:\Users\Acer\Downloads\ram clean\memclean"
    & $cargoPath test --test test_cleaner -- --nocapture > "C:\Users\Acer\Downloads\ram clean\memclean\elevated_test_output.txt" 2>&1
    echo $LASTEXITCODE > "C:\Users\Acer\Downloads\ram clean\memclean\elevated_test_exitcode.txt"
    echo "DONE" > "C:\Users\Acer\Downloads\ram clean\memclean\elevated_test_done.txt"
} else {
    $encoded = [System.Convert]::ToBase64String([System.Text.Encoding]::Unicode.GetBytes('powershell -NoProfile -ExecutionPolicy Bypass -File "C:\Users\Acer\Downloads\ram clean\memclean\run_elevated_ps.ps1"'))
    Start-Process powershell.exe -ArgumentList "-NoProfile -ExecutionPolicy Bypass -EncodedCommand $encoded" -Verb RunAs
}
