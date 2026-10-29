@echo off
cd /d "C:\Users\Acer\Downloads\ram clean\memclean"
"C:\Users\Acer\Downloads\ram clean\memclean\target\debug\deps\test_cleaner-*" 2>nul
set CARGO_PATH=C:\Users\Acer\.cargo\bin\cargo.exe
if not exist "%CARGO_PATH%" set CARGO_PATH=C:\Users\Acer\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin\cargo.exe
"%CARGO_PATH%" test --test test_cleaner -- --nocapture > "C:\Users\Acer\Downloads\ram clean\memclean\elevated_test_output.txt" 2>&1
echo %ERRORLEVEL% > "C:\Users\Acer\Downloads\ram clean\memclean\elevated_test_exitcode.txt"
echo DONE > "C:\Users\Acer\Downloads\ram clean\memclean\elevated_test_done.txt"
