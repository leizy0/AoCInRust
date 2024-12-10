$root_dir = Get-Location
$cargo_dirs = Get-ChildItem -Recurse -Directory | Where-Object Name -Match ".*day[0-9_]+$" | ForEach-Object {$_.FullName}

foreach ($dir in $cargo_dirs) {
    Write-Host "Cleaning $dir"
    Set-Location $dir
    cargo clean
}

Set-Location $root_dir
