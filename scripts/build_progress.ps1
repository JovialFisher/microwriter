<#
    scripts/build_progress.ps1

    Friendly first-build progress for the Windows launcher.

    Runs `cargo build --release` with all of its compiler output redirected to
    the log file, and draws a quiet progress line instead - spinner, real
    percentage, bar, elapsed time - so someone who has never seen a compiler
    is not buried in "Compiling ..." noise.

    The percentage is not a guess: cargo metadata reports how many crates this
    machine needs, and the script counts the "Compiling ..." lines cargo
    writes as it works.

    Exits with cargo's exit code, so the launcher can tell success from
    failure. Called by launch_microwriter.bat:

        powershell -NoProfile -ExecutionPolicy Bypass -File scripts\build_progress.ps1 -Cargo cargo -Log target\build.log

    Written for Windows PowerShell 5.1: no non-ASCII source characters (glyphs
    come from [char] codes, which also survive an ANSI code page) and no
    PowerShell 7 syntax.
#>
[CmdletBinding()]
param(
    [string]$Cargo = 'cargo',
    [string]$Log = 'target\build.log',
    [int]$BarWidth = 24,
    [int]$TickMilliseconds = 250
)

$ErrorActionPreference = 'Stop'

# Anchor everything to the repository root (this script lives in <root>\scripts)
# so the build and the log land in the same place whatever the caller's
# working directory was.
$root = Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath $root

function Resolve-LogPath {
    param([string]$Path)
    if ([System.IO.Path]::IsPathRooted($Path)) { return $Path }
    return (Join-Path $root $Path)
}

$stdoutLog = Resolve-LogPath -Path $Log

$logDirectory = Split-Path -Parent $stdoutLog
if ($logDirectory -and -not (Test-Path -LiteralPath $logDirectory)) {
    New-Item -ItemType Directory -Path $logDirectory -Force | Out-Null
}

# A glyph is only used when the console's encoding can carry it, otherwise it
# would show up as a question mark. A legacy code page (IBM437, the common
# case) has the block glyphs but not braille, so the spinner and the bar are
# chosen separately.
function Test-GlyphSupported {
    param([int]$CodePoint)
    try {
        return ([Console]::OutputEncoding.GetBytes([string][char]$CodePoint)[0] -ne 0x3F)
    }
    catch {
        return $false
    }
}

if (Test-GlyphSupported -CodePoint 0x280B) {
    # Braille spinner frames.
    $spinner = @(0x280B, 0x2819, 0x2839, 0x2838, 0x283C, 0x2834, 0x2826, 0x2827, 0x2807, 0x280F) |
        ForEach-Object { [string][char]$_ }
}
else {
    $spinner = @('|', '/', '-', '\')
}

if ((Test-GlyphSupported -CodePoint 0x2588) -and (Test-GlyphSupported -CodePoint 0x2591)) {
    $filledGlyph = [string][char]0x2588
    $emptyGlyph = [string][char]0x2591
}
else {
    $filledGlyph = '#'
    $emptyGlyph = '-'
}

# How many crates this machine is going to build, or 0 when it cannot be
# counted (the progress line then shows the running count instead of a
# percentage).
function Get-ExpectedCrates {
    try {
        $metadataArgs = @('metadata', '--format-version', '1')

        # Filtering to the local target keeps the count close to what cargo
        # actually compiles instead of counting every platform's dependencies.
        $rustc = $null
        $onPath = Get-Command rustc -ErrorAction SilentlyContinue
        if ($onPath) {
            $rustc = $onPath.Source
        }
        else {
            $besideCargo = Join-Path (Split-Path -Parent $Cargo) 'rustc.exe'
            if (Test-Path -LiteralPath $besideCargo) { $rustc = $besideCargo }
        }
        if ($rustc) {
            $hostTriple = (& $rustc -vV 2>$null | Select-String -Pattern '^host:\s*(\S+)')
            if ($hostTriple) {
                $metadataArgs += @('--filter-platform', $hostTriple.Matches[0].Groups[1].Value)
            }
        }

        $metadata = & $Cargo @metadataArgs 2>$null | Out-String
        return ([regex]::Matches($metadata, '"manifest_path"')).Count
    }
    catch {
        return 0
    }
}

# Crates cargo has finished so far, counted from its own progress lines.
function Get-CompiledCount {
    if (-not (Test-Path -LiteralPath $stdoutLog)) { return 0 }
    $found = Select-String -LiteralPath $stdoutLog -Pattern 'Compiling' -SimpleMatch -ErrorAction SilentlyContinue |
        Measure-Object
    return $found.Count
}

# One line of progress, e.g.
#   (spinner)  Building      42%  [#####---------------]  0:48
function Format-Progress {
    param(
        [int]$Done,
        [int]$Elapsed,
        [int]$Expected,
        [string]$Frame
    )

    $time = '{0}:{1:00}' -f [math]::Floor($Elapsed / 60), ($Elapsed % 60)

    if ($Done -le 0) {
        # Nothing compiled yet: cargo is starting up or fetching dependencies.
        $dots = '.' * (($Elapsed % 4) + 1)
        if ($Elapsed -lt 12) { $label = 'Getting started' }
        else { $label = 'Fetching what it needs' }
        return ('{0}  {1}{2}  {3}' -f $Frame, $label, $dots, $time)
    }

    if ($Expected -le 0) {
        return ('{0}  {1,-13} {2} pieces done  {3}' -f $Frame, 'Building', $Done, $time)
    }

    $percent = [math]::Min(99, [int][math]::Floor(100 * $Done / $Expected))
    # The last crate is microwriter itself, and its linking step takes a while,
    # so say what is happening rather than looking stuck at 99%.
    if ($Done -ge ($Expected - 1)) { $label = 'Finishing up' }
    else { $label = 'Building' }

    $filled = [int][math]::Floor($percent * $BarWidth / 100)
    $bar = ($filledGlyph * $filled) + ($emptyGlyph * ($BarWidth - $filled))
    return ('{0}  {1,-13} {2,2}%  [{3}]  {4}' -f $Frame, $label, $percent, $bar, $time)
}

$expected = Get-ExpectedCrates

# The build runs through a tiny generated cmd script: cmd then owns the
# redirection into the log, and - unlike Start-Process, which drops the exit
# code when it redirects streams - the exit code can be read back reliably.
# Without it a failed build would look like a successful one.
$helperPath = Join-Path $logDirectory 'build_progress.cmd'
$helper = @(
    '@echo off'
    ('"{0}" build --release > "{1}" 2>&1' -f $Cargo, $stdoutLog)
) -join "`r`n"
Set-Content -LiteralPath $helperPath -Value $helper -Encoding ASCII

$startInfo = New-Object System.Diagnostics.ProcessStartInfo
$startInfo.FileName = $env:ComSpec
$startInfo.Arguments = '/c "' + $helperPath + '"'
$startInfo.UseShellExecute = $false
$startInfo.CreateNoWindow = $true
$startInfo.WorkingDirectory = $root

try {
    $proc = [System.Diagnostics.Process]::Start($startInfo)
}
catch {
    Write-Host ''
    Write-Host "  Sorry - the build could not be started: $($_.Exception.Message)"
    exit 1
}

$interactive = -not [Console]::IsOutputRedirected
if ($interactive) {
    try { [Console]::CursorVisible = $false } catch { }
}

# Pad (and so clear) the progress line to the console width, so it never wraps
# onto a second row in a narrow window.
$lineWidth = 72
if ($interactive) {
    try {
        if ([Console]::WindowWidth -gt 8) { $lineWidth = [Console]::WindowWidth - 1 }
    }
    catch { }
}

$watch = [System.Diagnostics.Stopwatch]::StartNew()
$frame = 0
$milestone = -1

while (-not $proc.HasExited) {
    Start-Sleep -Milliseconds $TickMilliseconds

    $done = Get-CompiledCount
    $elapsed = [int]$watch.Elapsed.TotalSeconds
    $line = Format-Progress -Done $done -Elapsed $elapsed -Expected $expected -Frame $spinner[$frame % $spinner.Count]

    if ($interactive) {
        Write-Host -NoNewline ("`r" + $line.PadRight($lineWidth))
    }
    else {
        # A redirected run (a log, or a script) gets one plain line per
        # milestone instead of a line that rewrites itself.
        if ($done -gt 0 -and $expected -gt 0) { $step = [int]($done * 100 / $expected / 20) }
        else { $step = [int]($elapsed / 15) }
        if ($step -ne $milestone) {
            $milestone = $step
            Write-Host $line
        }
    }
    $frame++
}

$proc.WaitForExit()

if ($interactive) {
    Write-Host -NoNewline ("`r" + (' ' * $lineWidth) + "`r")
    try { [Console]::CursorVisible = $true } catch { }
}

exit $proc.ExitCode
