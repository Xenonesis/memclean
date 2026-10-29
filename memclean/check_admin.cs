using System;
using System.Security.Principal;
class Checker {
    static void Main() {
        WindowsPrincipal p = new WindowsPrincipal(WindowsIdentity.GetCurrent());
        Console.WriteLine("IsAdmin:" + p.IsInRole(WindowsBuiltInRole.Administrator));
    }
}
