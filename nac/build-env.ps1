$workspace = Split-Path $PSScriptRoot -Parent
$env:CARGO_HOME = "$PSScriptRoot/.build/cargo"
$env:CARGO_TARGET_DIR = "$PSScriptRoot/target"
$env:TEMP = "$PSScriptRoot/.build/tmp"
$env:TMP = $env:TEMP
$env:LIBCLANG_PATH = "$PSScriptRoot/.build/libclang/libclang-18.1.1.data/platlib/clang/native"
$vs = 'C:/Program Files (x86)/Microsoft Visual Studio/2022/BuildTools'
$env:PATH = "$vs/Common7/IDE/CommonExtensions/Microsoft/CMake/CMake/bin;$vs/Common7/IDE/CommonExtensions/Microsoft/CMake/Ninja;$env:PATH"
$env:CMAKE_POLICY_VERSION_MINIMUM = '3.5'
$env:WINMESSAGES_NAC_BIN = "$workspace/vendor/nacbin/IMDAppleServices"

$env:OPENSSL_SRC_PERL = "$PSScriptRoot/.build/strawberry/perl/bin/perl.exe"
$env:PROTOC = "$PSScriptRoot/.build/protoc/bin/protoc.exe"
