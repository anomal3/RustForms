# Собирает дизайнер и плагин RustRover. Нужна только папка самой IDE: её JDK и библиотеки.
param(
    # Переносимая сборка для релиза: дизайнер вкладывается в плагин, путей этой машины нет.
    [switch]$Release,
    [string]$Ide = $(if ($env:RUSTROVER_HOME) { $env:RUSTROVER_HOME }
                     else { "$env:USERPROFILE\Downloads\RustRover-2026.3.1.1.win" })
)
$ErrorActionPreference = 'Stop'
$utf8 = New-Object Text.UTF8Encoding $false
$javac = "$Ide\jbr\bin\javac.exe"
if (-not (Test-Path $javac)) { throw "Не найдена RustRover в '$Ide'. Укажите: build.ps1 -Ide <папка>" }

$root = Split-Path (Split-Path $PSScriptRoot)
# Открытый дизайнер держит свой exe: заменить его нельзя, а переименовать можно.
$exe = "$root\target\release\rustforms-designer.exe"
Get-ChildItem "$root\target\release\rustforms-designer.*.old" -ErrorAction SilentlyContinue |
    Remove-Item -ErrorAction SilentlyContinue
if (Test-Path $exe) {
    Move-Item $exe "$root\target\release\rustforms-designer.$PID.old" -ErrorAction SilentlyContinue
}
Push-Location $root
if ($Release) {
    # Статическая CRT — чтобы не требовался распространяемый пакет Visual C++;
    # пути домашней папки вычищаются из сообщений внутри exe.
    $env:RUSTFLAGS = "-C target-feature=+crt-static --remap-path-prefix=$env:USERPROFILE=~ --remap-path-prefix=$root=rustforms"
    cargo build --release -p rustforms-designer --target-dir "$root\target\dist"
    $env:RUSTFLAGS = $null
} else {
    cargo build --release -p rustforms-designer
}
$failed = $LASTEXITCODE
Pop-Location
if ($failed) { throw 'Дизайнер не собрался' }

$build = "$PSScriptRoot\build"
if (Test-Path $build) { Remove-Item $build -Recurse -Force }
New-Item -ItemType Directory "$build\classes", "$build\dist\rustforms\lib" | Out-Null
Copy-Item "$PSScriptRoot\resources\*" "$build\classes" -Recurse
New-Item -ItemType Directory "$build\classes\icons" | Out-Null
Copy-Item "$root\crates\rustforms-designer\icons\*.png" "$build\classes\icons"
if (-not $Release) {
    [IO.File]::WriteAllText("$build\classes\designer.path", "$root\target\release\rustforms-designer.exe", $utf8)
}

# Список файлов и classpath передаются через файл: так не упереться в длину командной строки.
$jars = Get-ChildItem "$Ide\lib", "$Ide\modules" -Recurse -Filter *.jar
$classpath = ($jars.FullName -join ';').Replace('\', '/')
$arguments = @('--release', '21', '-encoding', 'utf-8', '-proc:none', '-Xlint:deprecation',
               '-classpath', "`"$classpath`"", '-d', "`"$($build.Replace('\','/'))/classes`"")
$arguments += Get-ChildItem "$PSScriptRoot\src" -Recurse -Filter *.java |
    ForEach-Object { '"' + $_.FullName.Replace('\', '/') + '"' }
[IO.File]::WriteAllLines("$build\javac.args", [string[]]$arguments, $utf8)
& $javac "@$build\javac.args"
if ($LASTEXITCODE) { throw 'javac завершился с ошибкой' }

# В JDK из поставки IDE нет jar.exe, а Compress-Archive пишет в архив обратные косые черты.
Add-Type -AssemblyName System.IO.Compression, System.IO.Compression.FileSystem
function New-Zip([string]$Folder, [string]$Target) {
    $base = (Resolve-Path $Folder).Path
    $zip = [IO.Compression.ZipFile]::Open($Target, 'Create')
    try {
        foreach ($file in Get-ChildItem $base -Recurse -File) {
            $name = $file.FullName.Substring($base.Length + 1).Replace('\', '/')
            [IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $file.FullName, $name) | Out-Null
        }
    } finally { $zip.Dispose() }
}
New-Zip "$build\classes" "$build\dist\rustforms\lib\rustforms.jar"
$name = 'rustforms-rustrover.zip'
if ($Release) {
    New-Item -ItemType Directory "$build\dist\rustforms\bin" | Out-Null
    Copy-Item "$root\target\dist\release\rustforms-designer.exe" "$build\dist\rustforms\bin"
    $version = ([xml](Get-Content "$PSScriptRoot\resources\META-INF\plugin.xml" -Raw -Encoding UTF8)).'idea-plugin'.version
    $name = "rustforms-rustrover-$version-windows-x64.zip"
}
New-Zip "$build\dist" "$build\$name"
Write-Host "Готово: $build\$name"
