#requires -version 7
Set-StrictMode -Version Latest

$PSScriptRoot | Set-Variable -Name SCRIPT_ROOT -Option Constant
Join-Path $PSScriptRoot "../../tools/CPUEmulator.bat" | Set-Variable -Name TEST_SCRIPT -Option Constant

function print-result([bool] $success, $msg) {
    if ($success) {
        Write-Host "  [o] success: ${msg}" -ForegroundColor Green
    }
    else {
        Write-Host "  [x] failed: ${msg}" -ForegroundColor Red
        exit 1
    }
}

function test ($path) {
    $target_dir = Join-Path $PSScriptRoot $path
    $name = Split-Path $target_dir -Leaf
    cargo run -- $target_dir
    print-result -success $? "cargo run"

    return cmd.exe /c $TEST_SCRIPT $path/$name.tst
}

function main() {
    Write-Host "cargo build"
    cargo build
    print-result -success $? "cargo build"

    $successes = @()
    $errors = @()

    $target = "../07-vm1-stack-arithmetic/StackArithmetic/SimpleAdd"
    $res = test -path $target
    if ($res) {
        $successes += $target
    }
    else {
        $errors += $target
    }

    Write-Host "successes:"
    $successes | ForEach-Object {
        Write-Host "  [o] ${_}" -ForegroundColor Green
    }
    Write-Host "errors:"
    $errors | ForEach-Object {
        Write-Host "  [x] ${_}" -ForegroundColor Red
    }
}

main
