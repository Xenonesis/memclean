$process = ([wmiclass] "win32_process").Create("C:\Users\Acer\Downloads\ram clean\memclean\run_elevated_test6.bat")
echo "Process created with PID: $($process.ProcessId)"
echo "Return value: $($process.ReturnValue)"
