@echo off
cd /d "C:\Users\Acer\Downloads\ram clean\memclean"
cargo test --test test_cleaner 2>&1
exit /b %ERRORLEVEL%
