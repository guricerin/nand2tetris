#requires -version 7
Set-StrictMode -Version Latest

Join-Path $PSScriptRoot "../../tools/TextComparer.bat" | Set-Variable -Name DIFF_TOOL -Option Constant

function run-cargo($arg) {
    $cmd = "cargo"
    $proc = Start-Process $cmd -WorkingDirectory $PSScriptRoot -ArgumentList $arg -Wait -PassThru -NoNewWindow
    $res = $proc.ExitCode -eq 0
    return $res;
}

function main() {
    run-cargo "test"

    $success = @()
    $fail = @()

    @(
        "../06-assembler/add/Add.asm"
        "../06-assembler/max/Max.asm"
        "../06-assembler/max/MaxL.asm"
        "../06-assembler/pong/Pong.asm"
        "../06-assembler/pong/PongL.asm"
        "../06-assembler/rect/Rect.asm"
        "../06-assembler/rect/RectL.asm"
    ) | ForEach-Object {
        $res = run-cargo "run --release $_"
        $path = Join-Path $PSScriptRoot $_
        $path = Convert-Path $path
        if ($res) {
            $success += "  [o] $path"
        }
        else {
            $fail += "  [x] $path"
        }
    }

    $success | ForEach-Object {
        Write-Host $_ -ForegroundColor Green
    }
    $fail | ForEach-Object {
        Write-Host $_ -ForegroundColor Red
    }
    Write-Host
    Write-Host "Info: please check .hack code with the CPUEmulator" -ForegroundColor Yellow
}

main
