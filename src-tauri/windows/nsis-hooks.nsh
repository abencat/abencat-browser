!macro NSIS_HOOK_PREUNINSTALL
  IfFileExists "$INSTDIR\controller-data\*.*" 0 done_controller_data_cleanup
  MessageBox MB_ICONQUESTION|MB_YESNO "是否同时删除 controller-data 数据目录？$\r$\n$\r$\n这会删除所有浏览器环境、项目、脚本配置和本机浏览器用户数据。$\r$\n选择“否”将保留这些数据，方便以后重新安装继续使用。$\r$\n$\r$\n$INSTDIR\controller-data" IDNO done_controller_data_cleanup
  RMDir /r "$INSTDIR\controller-data"
done_controller_data_cleanup:
!macroend
