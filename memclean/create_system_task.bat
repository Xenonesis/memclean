@echo off
REM Create scheduled task to run as LOCAL SERVICE with highest privileges
schtasks /create /tn MemCleanTestSvc /tr "C:\Users\Acer\Downloads\ram clean\memclean\run_elevated_test6.bat" /sc once /st 00:00 /ru "NT AUTHORITY\SYSTEM" /f 2>&1
echo %ERRORLEVEL%
