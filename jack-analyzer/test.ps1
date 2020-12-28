#requires -version 7
Set-StrictMode -Version Latest

Join-Path $PSScriptRoot "../../tools/TextComparer.bat" | Set-Variable -Name DIFF_TOOL -Option Constant

function print-result([bool] $success, $msg) {
    if ($success) {
        Write-Host "  [o] success: ${msg}" -ForegroundColor Green
    }
    else {
        Write-Host "  [x] failed: ${msg}" -ForegroundColor Red
        exit 1
    }
}

function run-cargo($arg) {
    $cmd = "cargo"
    $proc = Start-Process $cmd -WorkingDirectory $PSScriptRoot -ArgumentList $arg -Wait -PassThru -NoNewWindow
    $res = $proc.ExitCode -eq 0
    print-result -success $res "$cmd $arg"
}

function lex_xml_test($path) {
    # remake output dir
    $output_dir = Join-Path $PSScriptRoot "output"
    Remove-Item $output_dir -Recurse -Force
    mkdir $output_dir

    # cargo run
    $target_dir = Join-Path $PSScriptRoot $path
    $name = Split-Path $target_dir -Leaf
    Start-Process "cargo" -WorkingDirectory $PSScriptRoot -ArgumentList "run --release -- $target_dir -o $output_dir xml" -Wait -NoNewWindow

    $success = @()
    $fail = @()

    Get-ChildItem $target_dir -Include ".jack" | ForEach-Object {
        $target = $_.FullName
        $target = $target.Replace(".jack", "T.xml")
        $cmd = "cmd.exe"
        $arg = "/c  $DIFF_TOOL $target_dir/$target $output_dir/$target"
        $proc = Start-Process $cmd -WorkingDirectory $PSScriptRoot -ArgumentList $arg -Wait -PassThru -NoNewWindow
        $res = $proc.ExitCode -eq 0
        if ($res) {
            $success += $target
        }
        else {
            $fail += $target
        }
    }

    $success | ForEach-Object {
        Write-Host "  [o] $_" -ForegroundColor Green
    }
    $fail | ForEach-Object {
        Write-Host "  [x] $_" -ForegroundColor Red
    }
}

function main() {
    run-cargo("test")

    @(
        "../10-compiler1-syntax-analysis/ArrayTest"
        "../10-compiler1-syntax-analysis/ExpressionLessSquare"
        "../10-compiler1-syntax-analysis/Square"
    ) | ForEach-Object {
        lex_xml_test $_
    }
}

main
