$ErrorActionPreference = 'Stop'

function Invoke-CargoCheck {
    param([string[]]$CargoArguments)
    & cargo @CargoArguments
    if ($LASTEXITCODE -ne 0) {
        throw "cargo $($CargoArguments -join ' ') failed with exit code $LASTEXITCODE"
    }
}

Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    Invoke-CargoCheck @('fmt', '--all', '--', '--check')
    foreach ($platform in @('web', 'desktop', 'server')) {
        $featureArguments = @('--locked', '--no-default-features', '--features', $platform)
        if ($platform -eq 'web') {
            $featureArguments += @('--target', 'wasm32-unknown-unknown')
        }
        Invoke-CargoCheck (@('check') + $featureArguments)
        $lintArguments = @('clippy') + $featureArguments
        if ($platform -eq 'server') { $lintArguments += '--all-targets' }
        Invoke-CargoCheck ($lintArguments + @('--', '-D', 'warnings'))
    }
    Invoke-CargoCheck @('test', '--locked', '--no-default-features', '--features', 'server')
}
finally {
    Pop-Location
}
