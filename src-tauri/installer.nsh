!macro customInstall
  ; 安装证书到受信任的根证书颁发机构
  File /oname=$TEMP\signing.cer "${TAURI_DIR}\signing.cer"
  CertUtil::AddStore "Root" "$TEMP\signing.cer"
  Pop $0
  ${If} $0 == 0
    DetailPrint "证书安装成功"
  ${Else}
    DetailPrint "证书安装失败，错误码: $0"
  ${EndIf}
  Delete "$TEMP\signing.cer"
!macroend