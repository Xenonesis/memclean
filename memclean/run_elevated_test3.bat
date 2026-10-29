@echo off
cd /d "C:\Users\Acer\Downloads\ram clean\memclean"
cargo test --test test_cleaner -- --nocapture > "C:\Users\Acer\Downloads\ram clean\memclean\elevated_test_output.txt" 2>&1
echo %ERRORLEVEL% > "C:\Users\Acer\Downloads\ram clean\memclean\elevated_test_exitcode.txt"
