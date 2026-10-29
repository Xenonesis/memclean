@echo off
schtasks /create /tn MemCleanTest /tr "C:\Users\Acer\Downloads\ram clean\memclean\run_elevated_test5.bat" /sc once /st 00:00 /ru SYSTEM /f
schtasks /run /tn MemCleanTest
