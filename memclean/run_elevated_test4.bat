@echo off
schtasks /create /tn "MemCleanTestRun" /tr "C:\Users\Acer\Downloads\ram clean\memclean\run_elevated_test3.bat" /sc once /st 00:00 /ru SYSTEM /f > nul 2>&1
schtasks /run /tn "MemCleanTestRun" /i 1 > nul 2>&1
schtasks /delete /tn "MemCleanTestRun" /f > nul 2>&1
