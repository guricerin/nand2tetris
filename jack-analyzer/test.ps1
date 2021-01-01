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

function lex-xml-test($path) {
    # remake output dir
    $output_dir = Join-Path $PSScriptRoot "output/lex"
    if (Test-Path $output_dir) {
        Remove-Item $output_dir -Recurse -Force
    }
    mkdir $output_dir

    # cargo run
    $target_dir = Join-Path $PSScriptRoot $path
    Start-Process "cargo" -WorkingDirectory $PSScriptRoot -ArgumentList "run --release -- $target_dir -o $output_dir txml" -Wait

    Get-ChildItem $target_dir -File -Recurse -Include *.jack | ForEach-Object {
        $target = $_.FullName
        $target = $target.Replace(".jack", "T.xml")

        $output = Split-Path $target -Leaf
        $output = Join-Path $output_dir $output

        $res = diff-test $target $output
        if ($res) {
            $script:success += "  [o] lex_xml_test: $target"
        }
        else {
            $script:fail += "  [x] lex_xml_test: $target"
        }
    }
}

function parse-xml-test($path) {
    # remake output dir
    $output_dir = Join-Path $PSScriptRoot "output/parse"
    if (Test-Path $output_dir) {
        Remove-Item $output_dir -Recurse -Force
    }
    mkdir $output_dir

    # cargo run
    $target_dir = Join-Path $PSScriptRoot $path
    Start-Process "cargo" -WorkingDirectory $PSScriptRoot -ArgumentList "run --release -- $target_dir -o $output_dir axml" -Wait

    Get-ChildItem $target_dir -File -Recurse -Include *.jack | ForEach-Object {
        $target = $_.FullName
        $target = $target.Replace(".jack", ".xml")

        $output = Split-Path $target -Leaf
        $output = Join-Path $output_dir $output

        $res = diff-test $target $output
        if ($res) {
            $script:success += "  [o] parse_xml_test: $target"
        }
        else {
            $script:fail += "  [x] parse_xml_test: $target"
        }
    }
}

function compile-test($path) {
    $leaf = Split-Path $path -Leaf
    $output_dir = Join-Path $PSScriptRoot "output/$leaf"
    if (!(Test-Path $output_dir)) {
        mkdir $output_dir
    }
    $jack_os_path = Join-Path $PSScriptRoot "../../tools/OS"
    Get-ChildItem $jack_os_path -File | ForEach-Object {
        $file = Split-Path $_ -Leaf
        Copy-Item $_ $output_dir/$file
    }

    $target_dir = Join-Path $PSScriptRoot $path
    $target_dir = Convert-Path $target_dir
    $proc = Start-Process "cargo" -WorkingDirectory $PSScriptRoot -ArgumentList "run --release -- $target_dir -o $output_dir c" -Wait -PassThru
    $res = $proc.ExitCode -eq 0
    if ($res) {
        $script:success += "  [o] compile_test: $target_dir"
    }
    else {
        $script:fail += "  [x] compile_test: $target_dir"
    }
}

function diff-test($target, $output) {
    # batファイルはこうやって実行させないと戻り値を取得できない
    return cmd.exe /c $DIFF_TOOL $target $output
}

function main() {
    run-cargo("test")

    $output_dir = Join-Path $PSScriptRoot "output"
    if (Test-Path $output_dir) {
        Remove-Item $output_dir -Recurse -Force
    }
    mkdir $output_dir

    $script:success = @()
    $script:fail = @()

    @(
        "../10-compiler1-syntax-analysis/ExpressionLessSquare"
        "../10-compiler1-syntax-analysis/ArrayTest"
        "../10-compiler1-syntax-analysis/Square"
    ) | ForEach-Object {
        lex-xml-test $_
        parse-xml-test $_
        compile-test $_
    }

    @(
        "../11-compiler2-code-generation/Seven"
        "../11-compiler2-code-generation/ConvertToBin"
        "../11-compiler2-code-generation/Square"
        "../11-compiler2-code-generation/Average"
        "../11-compiler2-code-generation/Pong"
        "../11-compiler2-code-generation/ComplexArrays"
    ) | ForEach-Object {
        compile-test $_
    }

    Write-Host
    $script:success | ForEach-Object {
        Write-Host "$_" -ForegroundColor Green
    }
    $script:fail | ForEach-Object {
        Write-Host "$_" -ForegroundColor Red
    }
    Write-Host "Info: please check output vm code with the VMEmulator." -ForegroundColor Yellow
}

main
