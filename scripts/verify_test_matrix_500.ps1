$files = 1..10 | ForEach-Object { "tests/matrix_500_{0:D2}.rs" -f $_ }
$names = foreach ($file in $files) {
    [regex]::Matches((Get-Content $file -Raw), 'case!\((t\d{3}),\s*(\d+)\)') | ForEach-Object { $_.Groups[1].Value }
}
$expected = 1..500 | ForEach-Object { 't{0:D3}' -f $_ }
$matrixIds = Get-Content tests/TEST_MATRIX_500.md | ForEach-Object {
    if ($_ -match '^\|\s*([A-Z0-9]+-\d{3})\s*\|') { $Matches[1] }
}
if ($files.Count -ne 10) { throw 'Nombre de fichiers incorrect' }
foreach ($file in $files) {
    $count = [regex]::Matches((Get-Content $file -Raw), 'case!\(t\d{3},').Count
    if ($count -ne 50) { throw "$file contient $count cas au lieu de 50" }
}
if (($names | Sort-Object -Unique).Count -ne 500) { throw 'Les noms de tests ne sont pas uniques' }
if (Compare-Object $expected ($names | Sort-Object)) { throw 'La plage t001-t500 est incomplète' }
if (($matrixIds | Sort-Object -Unique).Count -ne 500) { throw 'La matrice ne contient pas 500 IDs uniques' }
Write-Host 'OK: 10 fichiers x 50 tests; noms uniques t001-t500; 500 IDs de matrice uniques.'
