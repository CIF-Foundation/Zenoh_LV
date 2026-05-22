; LabVIEW Zenoh Windows installer

; Installs zenoh_lv_wrapper.dll to a fixed location under Program Files.



!include "MUI2.nsh"



!define PRODUCT_NAME "LabVIEW Zenoh"

!define PRODUCT_VERSION "0.1.0"

!define PRODUCT_PUBLISHER "Dome Automation"

!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\LV-ZENOH"

!define INSTALL_DIR "$PROGRAMFILES64\CIF_Foundation\Libraries"

!define UNINSTALLER_NAME "lv-zenoh-Uninstall.exe"



Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"

OutFile "lv-zenoh-${PRODUCT_VERSION}.exe"

InstallDir "${INSTALL_DIR}"

RequestExecutionLevel admin

ShowInstDetails show

ShowUnInstDetails show



!insertmacro MUI_PAGE_WELCOME

!insertmacro MUI_PAGE_INSTFILES

!define MUI_FINISHPAGE_TEXT "Setup has successfully installed ${PRODUCT_NAME}.$\r$\n$\r$\nIf you are developing applications with Zenoh in LabVIEW you should also install the Zenoh package using VIPM."

!insertmacro MUI_PAGE_FINISH



!insertmacro MUI_UNPAGE_CONFIRM

!insertmacro MUI_UNPAGE_INSTFILES



!insertmacro MUI_LANGUAGE "English"



Section "Install"

  SetOutPath "${INSTALL_DIR}"

  File "resource\zenoh_lv_wrapper.dll"



  WriteUninstaller "${INSTALL_DIR}\${UNINSTALLER_NAME}"



  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_NAME}"

  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "UninstallString" "$\"${INSTALL_DIR}\${UNINSTALLER_NAME}$\""

  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"

  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"

  WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "InstallLocation" "${INSTALL_DIR}"

  WriteRegDWORD HKLM "${PRODUCT_UNINST_KEY}" "NoModify" 1

  WriteRegDWORD HKLM "${PRODUCT_UNINST_KEY}" "NoRepair" 1

SectionEnd



Section "Uninstall"

  Delete "${INSTALL_DIR}\zenoh_lv_wrapper.dll"

  Delete "${INSTALL_DIR}\${UNINSTALLER_NAME}"

  DeleteRegKey HKLM "${PRODUCT_UNINST_KEY}"

SectionEnd

