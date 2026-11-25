# Собирает дизайнер и расширение Visual Studio. Нужна VS с компонентом разработки расширений.
$ErrorActionPreference = 'Stop'
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$msbuild = & $vswhere -all -latest -prerelease -requires Microsoft.VisualStudio.Workload.VisualStudioExtension `
    -find 'MSBuild\Current\Bin\MSBuild.exe' | Select-Object -First 1
if (-not $msbuild) { throw 'Не найдена Visual Studio с компонентом разработки расширений' }

$root = Split-Path (Split-Path $PSScriptRoot)
# Открытый дизайнер держит свой exe: заменить его нельзя, а переименовать можно.
$exe = "$root\target\release\rustforms-designer.exe"
Get-ChildItem "$root\target\release\rustforms-designer.*.old" -ErrorAction SilentlyContinue |
    Remove-Item -ErrorAction SilentlyContinue
if (Test-Path $exe) {
    Move-Item $exe "$root\target\release\rustforms-designer.$PID.old" -ErrorAction SilentlyContinue
}
Push-Location $root
cargo build --release -p rustforms-designer
$failed = $LASTEXITCODE
Pop-Location
if ($failed) { throw 'Дизайнер не собрался' }

& $msbuild "$PSScriptRoot\RustForms.VisualStudio.csproj" -restore -nologo -v:minimal `
    -p:Configuration=Release "-p:DesignerPath=$exe"
if ($LASTEXITCODE) { throw 'Расширение не собралось' }
Write-Host "Готово: $PSScriptRoot\bin\Release\net472\RustForms.VisualStudio.vsix"
