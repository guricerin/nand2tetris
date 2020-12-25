#requires -version 7
Set-StrictMode -Version Latest

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
    Start-Process "cargo" -WorkingDirectory $PSScriptRoot -ArgumentList "run --release -- $target_dir" -Wait -NoNewWindow

    return cmd.exe /c $TEST_SCRIPT $target_dir/$name.tst
}

function run-cargo($arg) {
    $cmd = "cargo"
    # Invoke-Expressionは非同期実行しかできず戻り値を取得できないので、これをつかう
    $proc = Start-Process $cmd -WorkingDirectory $PSScriptRoot -ArgumentList $arg -Wait -PassThru -NoNewWindow
    $res = $proc.ExitCode -eq 0
    print-result -success $res "$cmd $arg"
}

function main() {
    # cargo test
    run-cargo "test"

    # # cargo build
    # run-cargo "build --release"

    $successes = @()
    $fails = @()

    @(
        "../07-vm1-stack-arithmetic/StackArithmetic/SimpleAdd"
        "../07-vm1-stack-arithmetic/StackArithmetic/StackTest"
        "../07-vm1-stack-arithmetic/MemoryAccess/BasicTest"
        "../07-vm1-stack-arithmetic/MemoryAccess/PointerTest"
        "../07-vm1-stack-arithmetic/MemoryAccess/StaticTest"
    ) | ForEach-Object {
        $res = test -path $_
        if ($res) {
            $successes += $_
        }
        else {
            $fails += $_
        }
    }

    Write-Host "test result:" -ForegroundColor Yellow
    $successes | ForEach-Object {
        Write-Host "  [o] ${_}" -ForegroundColor Green
    }
    $fails | ForEach-Object {
        Write-Host "  [x] ${_}" -ForegroundColor Red
    }
}

main
