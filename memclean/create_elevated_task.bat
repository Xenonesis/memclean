@echo off
schtasks /create /tn MemCleanTestElev /tr "C:\Users\Acer\Downloads\ram clean\memclean\run_elevated_test6.bat" /sc once /st 00:00 /ru SYSTEM /f 2>&1
echo CREATE_RESULT=%ERRORLEVEL%
