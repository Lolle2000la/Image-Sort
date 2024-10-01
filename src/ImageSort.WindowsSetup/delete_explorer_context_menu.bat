@echo off
setlocal enabledelayedexpansion

for /f "tokens=*" %%G in ('reg query HKEY_USERS') do (
    set "userSID=%%G"
    if not "!userSID!"=="HKEY_USERS" (
        reg delete "!userSID!\Software\Classes\Directory\shell\ImageSort" /f
        reg delete "!userSID!\Software\Classes\Drive\shell\ImageSort" /f
        reg delete "!userSID!\Software\Classes\Folder\shell\ImageSort" /f
    )
)

endlocal
