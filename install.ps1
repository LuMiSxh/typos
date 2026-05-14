$ErrorActionPreference = "Stop"

$Repo = "LuMiSxh/typos"
$InstallDir = if ($env:TYPOS_INSTALL_DIR) { $env:TYPOS_INSTALL_DIR } else { "$env:USERPROFILE\.local\bin" }
$Target = "x86_64-pc-windows-msvc"

function Main {
    $latest = Get-LatestTag
    if (-not $latest) { Write-Error "could not determine latest release"; exit 1 }

    $url = "https://github.com/$Repo/releases/download/$latest/typos-$Target.zip"
    Write-Host "downloading typos $latest for $Target..."

    $tmp = New-TemporaryFile | ForEach-Object {
        Remove-Item $_
        New-Item -ItemType Directory -Path "$($_.FullName)_dir"
    }

    try {
        $zipPath = Join-Path $tmp.FullName "typos.zip"
        Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing
        Expand-Archive -Path $zipPath -DestinationPath $tmp.FullName -Force

        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }

        Copy-Item (Join-Path $tmp.FullName "typos.exe") (Join-Path $InstallDir "typos.exe") -Force
        Write-Host "installed typos to $InstallDir\typos.exe"

        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        if ($userPath -notlike "*$InstallDir*") {
            Write-Host "add to your PATH:"
            Write-Host "  [Environment]::SetEnvironmentVariable('Path', `"$InstallDir;`$env:Path`", 'User')"
        }
    } finally {
        Remove-Item $tmp.FullName -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Get-LatestTag {
    try {
        $r = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -UseBasicParsing
        return $r.tag_name
    } catch { return $null }
}

Main
