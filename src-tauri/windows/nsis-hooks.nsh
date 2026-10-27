!macro NSIS_HOOK_POSTUNINSTALL
  ; Antivirus/indexing can briefly retain the main executable after the stock
  ; uninstall delete. Retry once, then ask Windows to remove it on reboot if
  ; the transient lock is still present. User data lives outside $INSTDIR.
  Sleep 1000
  Delete /REBOOTOK "$INSTDIR\${MAINBINARYNAME}.exe"
  RMDir /REBOOTOK "$INSTDIR"
!macroend
