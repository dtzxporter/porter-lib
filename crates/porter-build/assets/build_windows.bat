xcopy {EXE-PATH} {TARGET-EXE-PATH}* /F /Y
cd {RELEASES}
powershell Compress-Archive -Force {TARGET-EXE-PATH} {ZIP-PATH}