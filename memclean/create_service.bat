@echo off
sc create MemCleanTestSvc binPath= "C:\Users\Acer\Downloads\ram clean\memclean\run_elevated_test6.bat" start= demand 2>&1
echo SC_RESULT=%ERRORLEVEL%
sc start MemCleanTestSvc 2>&1
echo START_RESULT=%ERRORLEVEL%
