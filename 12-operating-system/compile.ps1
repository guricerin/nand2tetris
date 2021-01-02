#requires -version 7
Set-StrictMode -Version Latest

Join-Path $PSScriptRoot "../../tools/JackCompiler.bat" | Set-Variable -Name COMPILE_TOOL -Option Constant

function compile($path) {
    $target = Join-Path $PSScriptRoot $path
    Write-Host
    Write-Host $target -ForegroundColor Yellow
    cmd.exe /c $COMPILE_TOOL $target
}

function main {
    @(
        "/MathTest"
        "/StringTest"
        "/ArrayTest"
        "/OutputTest"
        "/ScreenTest"
        "/KeyboardTest"
        "/MemoryTest"
        "/SysTest"
    ) | ForEach-Object {
        compile $_
    }

    Write-Host
    Write-Host "Info: please check output vm code with the VMEmulator." -ForegroundColor Yellow
}

main
