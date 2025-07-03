$file = $args[0]

if (-not $file) {
    Write-Host "Usage: rdl.ps1 <file>"
    exit 1
}

if (-not (Test-Path $file)) {
    Write-Host "File not found: $file"
    exit 1
}

$lines = New-Object System.Collections.Generic.List[System.Object]
$streamReader = New-Object System.IO.StreamReader($file)
while ($null -ne ($line = $streamReader.ReadLine())) {
    if ($line -match '^\s*#' || $lines.Contains($line)) {
        continue
    }

    $lines.Add($line)
}
$streamReader.Close()

foreach ($line in $lines.ToArray()) {
    $line = $line.Trim()
    if ($line.Length -ne 0) {
        Add-Content -Path "$file.new" -Value $line -Encoding UTF8
    }
}

# Replace the original file with the new one
Remove-Item -Path $file -Force
Move-Item -Path "$file.new" -Destination $file -Force