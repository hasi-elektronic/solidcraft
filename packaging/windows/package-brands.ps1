<#
.SYNOPSIS
  Build the independent HasiSolid and MSSolid Windows x64 packages.
.DESCRIPTION
  A clean checkout is required for release output. -Validation accepts a dirty
  tree and writes candidates below target/brand-validation, not dist/brands.
  Build, signing and post-install verification are separate release gates.
#>
param(
    [ValidateSet('HasiSolid', 'MSSolid', 'Both')] [string] $Brand = 'Both',
    [switch] $Validation,
    [string] $InnoCompiler
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$Version = '0.2.4'
$Target = 'x86_64-pc-windows-msvc'
$Source = (& git -C $Root rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0 -or $Source -notmatch '^[0-9a-f]{40}$') { throw 'Cannot identify source commit.' }
$Dirty = @(& git -C $Root status --porcelain --untracked-files=normal)
if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect the source tree.' }
if ($Dirty.Count -gt 0 -and -not $Validation) { throw 'Release packaging requires a clean source tree.' }
if (-not $Validation -and $Brand -ne 'Both') { throw 'Release packaging must produce both brands in one batch.' }

function Get-SourceFingerprint {
    $head = (& git -C $Root rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0 -or $head -ne $Source) { throw 'Source HEAD changed during packaging.' }
    $diff = @(& git -C $Root diff --no-ext-diff --binary HEAD --)
    if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect tracked source changes.' }
    $extra = @(& git -C $Root -c core.quotepath=false ls-files --others --exclude-standard)
    if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect untracked source files.' }
    $entries = foreach ($path in ($extra | Sort-Object)) {
        $full = Join-Path $Root $path
        if (-not (Test-Path -LiteralPath $full -PathType Leaf)) { throw "Cannot fingerprint untracked file: $path" }
        $item = Get-Item -LiteralPath $full -Force
        if ($item.LinkType) { throw "Untracked symlink cannot be included in a source snapshot: $path" }
        "$path`t$((Get-FileHash -LiteralPath $full -Algorithm SHA256).Hash)"
    }
    $text = @($head) + $diff + @($entries)
    $bytes = [Text.Encoding]::UTF8.GetBytes(($text -join "`n"))
    $hasher = [Security.Cryptography.SHA256]::Create()
    try { return [BitConverter]::ToString($hasher.ComputeHash($bytes)).Replace('-', '').ToLowerInvariant() }
    finally { $hasher.Dispose() }
}
$Fingerprint = Get-SourceFingerprint
function Assert-SourceUnchanged {
    if ((Get-SourceFingerprint) -ne $Fingerprint) { throw 'Source contents changed during packaging; the batch is rejected.' }
    if ($SnapshotCreated) {
        $snapshotHead = (& git -C $BuildRoot rev-parse HEAD).Trim()
        if ($LASTEXITCODE -ne 0 -or $snapshotHead -ne $Source) { throw 'Isolated source checkout HEAD changed.' }
        $snapshotChanges = @(& git -C $BuildRoot status --porcelain --untracked-files=normal)
        if ($LASTEXITCODE -ne 0 -or $snapshotChanges.Count -ne 0) { throw 'Isolated source checkout changed during packaging.' }
    }
}

$cargo = (Get-Command cargo.exe -ErrorAction SilentlyContinue).Source
if (-not $cargo) { $cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe' }
if (-not (Test-Path -LiteralPath $cargo -PathType Leaf)) { throw 'Cargo is not available.' }
if (-not $InnoCompiler) {
    $candidates = @(
        (Join-Path $env:LOCALAPPDATA 'Programs\Inno Setup 6\ISCC.exe'),
        (Join-Path ${env:ProgramFiles(x86)} 'Inno Setup 6\ISCC.exe')
    )
    $InnoCompiler = $candidates | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1
}
if (-not $InnoCompiler -or -not (Test-Path -LiteralPath $InnoCompiler -PathType Leaf)) { throw 'Inno Setup 6 compiler not found.' }

$RunId = [guid]::NewGuid().ToString('N')
$StageRoot = Join-Path $Root "target\brand-stage\$RunId"
$OutputDirectory = if ($Validation) { Join-Path $Root "target\brand-validation\$RunId" } else { Join-Path $StageRoot 'output' }
$FinalDirectory = Join-Path $Root "dist\brands\$Version"
if (-not $Validation -and (Test-Path -LiteralPath $FinalDirectory)) { throw "Release directory already exists: $FinalDirectory" }
$BuildRoot = $Root
$SnapshotCreated = $false
$env:CARGO_TARGET_DIR = Join-Path $Root "target\brand-build\$RunId"
$env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = '-C target-feature=+crt-static'
New-Item -ItemType Directory -Force -Path $OutputDirectory, $StageRoot | Out-Null
$Products = if ($Brand -eq 'Both') { @('HasiSolid', 'MSSolid') } else { @($Brand) }
$records = [System.Collections.Generic.List[string]]::new()
$signatures = [System.Collections.Generic.List[string]]::new()

function Assert-Pe([string] $Path, [int] $ExpectedSubsystem) {
    $stream = [IO.File]::OpenRead($Path)
    try {
        $reader = [IO.BinaryReader]::new($stream)
        if ($stream.Length -lt 256) { throw "PE file too small: $Path" }
        $stream.Position = 0x3c
        $pe = $reader.ReadInt32()
        if ($pe -lt 0 -or $pe + 0x60 -gt $stream.Length) { throw "Invalid PE header: $Path" }
        $stream.Position = $pe
        if ($reader.ReadUInt32() -ne 0x00004550) { throw "Invalid PE signature: $Path" }
        if ($reader.ReadUInt16() -ne 0x8664) { throw "Expected an x64 executable: $Path" }
        $stream.Position = $pe + 0x5c
        if ($reader.ReadUInt16() -ne $ExpectedSubsystem) { throw "Unexpected PE subsystem: $Path" }
    } finally {
        $stream.Dispose()
    }
}

if (-not $Validation) {
    $SnapshotParent = Join-Path $Root 'target\brand-sources'
    New-Item -ItemType Directory -Force -Path $SnapshotParent | Out-Null
    $BuildRoot = Join-Path $SnapshotParent $RunId
    & git -C $Root worktree add --detach $BuildRoot $Source
    if ($LASTEXITCODE -ne 0) { throw 'Cannot create isolated source checkout.' }
    $SnapshotCreated = $true
}
Push-Location $BuildRoot
try {
    foreach ($Product in $Products) {
        Assert-SourceUnchanged
        $Feature = if ($Product -eq 'HasiSolid') { 'brand-hasisolid' } else { 'brand-mssolid' }
        $Publisher = if ($Product -eq 'HasiSolid') { 'Hasi Elektronic' } else { 'Manfred Sickinger GmbH' }
        $Setup = Join-Path $OutputDirectory "$Product-$Version-Setup-x64.exe"
        $Zip = Join-Path $OutputDirectory "$Product-$Version-Portable-x64.zip"
        if ((Test-Path -LiteralPath $Setup) -or (Test-Path -LiteralPath $Zip)) { throw "Output already exists for $Product $Version." }

        & $cargo build --locked --release --target $Target -p solidcraft -p solidcraft-cli --features "solidcraft/$Feature,solidcraft-cli/$Feature"
        if ($LASTEXITCODE -ne 0) { throw "Cargo build failed for $Product." }
        Assert-SourceUnchanged
        $Bin = Join-Path $env:CARGO_TARGET_DIR "$Target\release"
        $Stage = Join-Path $StageRoot $Product
        New-Item -ItemType Directory -Force -Path $Stage | Out-Null
        $Gui = Join-Path $Stage "$Product.exe"
        $Cli = Join-Path $Stage "$Product-cli.exe"
        Copy-Item -LiteralPath (Join-Path $Bin 'solidcraft.exe') -Destination $Gui
        Copy-Item -LiteralPath (Join-Path $Bin 'solidcraft-cli.exe') -Destination $Cli
        Assert-Pe $Gui 2
        Assert-Pe $Cli 3

        $Info = [Diagnostics.FileVersionInfo]::GetVersionInfo($Gui)
        if ($Info.ProductName -ne $Product -or $Info.CompanyName -ne $Publisher -or $Info.ProductVersion -ne $Version) {
            throw "GUI metadata does not match $Product $Version ($Publisher)."
        }
        $CliVersion = (& $Cli --version | Out-String).Trim()
        if ($LASTEXITCODE -ne 0 -or $CliVersion -ne "$($Product.ToLowerInvariant())-cli $Version") {
            throw "CLI version check failed for $Product : $CliVersion"
        }
        foreach ($File in @('LICENSE-MIT', 'LICENSE-APACHE', 'NOTICE', 'THIRD-PARTY-LICENSES.txt')) {
            Copy-Item -LiteralPath (Join-Path $BuildRoot $File) -Destination $Stage
        }
        Copy-Item -LiteralPath (Join-Path $BuildRoot 'assets\brands\README.md') -Destination (Join-Path $Stage 'BRAND-LICENSE.md')
        $Readme = "$Product $Version`r`nSolidCraft 0.30.1, upstream 8e2b0cd60d14b4152b12b8f1fdb2bcfdcd8c52f4`r`nSource: $Source`r`nSnapshot: $Fingerprint`r`nValidation: $Validation`r`n"
        [IO.File]::WriteAllText((Join-Path $Stage 'README.txt'), $Readme, [Text.UTF8Encoding]::new($false))
        Compress-Archive -Path (Join-Path $Stage '*') -DestinationPath $Zip -CompressionLevel Optimal
        & $InnoCompiler "/DProduct=$Product" "/DVersion=$Version" "/DStage=$Stage" "/DOutputDirectory=$OutputDirectory" (Join-Path $BuildRoot 'packaging\windows\brands.iss')
        if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $Setup -PathType Leaf)) { throw "Inno Setup failed for $Product." }
        foreach ($file in @($Setup, $Zip)) {
            $hash = (Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash.ToLowerInvariant()
            $records.Add("$hash  $([IO.Path]::GetFileName($file))")
        }
        foreach ($file in @($Gui, $Cli, $Setup)) {
            $signature = Get-AuthenticodeSignature -LiteralPath $file
            $signatures.Add("$Product/$([IO.Path]::GetFileName($file)): $($signature.Status)")
        }
        Assert-SourceUnchanged
    }
    Assert-SourceUnchanged
    [IO.File]::WriteAllLines((Join-Path $OutputDirectory 'SHA256SUMS.txt'), $records, [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllLines((Join-Path $OutputDirectory 'SIGNATURES.txt'), $signatures, [Text.UTF8Encoding]::new($false))
    [IO.File]::WriteAllText((Join-Path $OutputDirectory 'SOURCE.txt'), "commit=$Source`nsnapshot_sha256=$Fingerprint`nvalidation=$Validation`ndirty=$($Dirty.Count -gt 0)`n", [Text.UTF8Encoding]::new($false))
    Assert-SourceUnchanged
    if (-not $Validation) {
        if (Test-Path -LiteralPath $FinalDirectory) { throw "Release directory already exists: $FinalDirectory" }
        $allowedStage = [IO.Path]::GetFullPath((Join-Path $Root 'target\brand-stage')).TrimEnd('\') + '\'
        $allowedOutput = [IO.Path]::GetFullPath((Join-Path $Root 'dist\brands')).TrimEnd('\') + '\'
        $resolvedOutput = [IO.Path]::GetFullPath($OutputDirectory)
        $resolvedFinal = [IO.Path]::GetFullPath($FinalDirectory)
        if (-not $resolvedOutput.StartsWith($allowedStage, [StringComparison]::OrdinalIgnoreCase) -or
            -not $resolvedFinal.StartsWith($allowedOutput, [StringComparison]::OrdinalIgnoreCase)) {
            throw 'Refusing to move a directory outside the intended staging and release paths.'
        }
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $FinalDirectory) | Out-Null
        [IO.Directory]::Move($resolvedOutput, $resolvedFinal)
        $OutputDirectory = $FinalDirectory
    }
    Write-Output "Packages staged at $OutputDirectory; installation, signing and release verification remain separate."
} finally {
    Pop-Location
    if ($SnapshotCreated) {
        $resolvedParent = [IO.Path]::GetFullPath($SnapshotParent).TrimEnd('\') + '\'
        $resolvedSource = [IO.Path]::GetFullPath($BuildRoot)
        if (-not $resolvedSource.StartsWith($resolvedParent, [StringComparison]::OrdinalIgnoreCase)) {
            throw 'Refusing to remove a checkout outside the intended source staging directory.'
        }
        & git -C $Root worktree remove -- $BuildRoot
        if ($LASTEXITCODE -ne 0) { Write-Warning "Could not remove isolated checkout: $BuildRoot" }
    }
}
