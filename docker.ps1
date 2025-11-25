# Define the location of the .env file (change if needed)
$envFile = "./auth-service/.env"

# Check if the .env file exists
if (-not (Test-Path $envFile)) {
    Write-Host "Error: .env file not found!" -ForegroundColor Red
    exit 1
}

# Read each non-comment, non-empty line
Get-Content $envFile | ForEach-Object {
    $line = $_.Trim()
    if ($line -and -not ($line.StartsWith("#"))) {
        # Split the line into key and value
        $parts = $line -split "=", 2
        if ($parts.Count -eq 2) {
            $key = $parts[0].Trim()
            $value = $parts[1].Trim()

            # Add the echo of the env values created
            # The url value is composed by the previous env values so I need to interpolate it
            if ($key -eq "DATABASE_URL") {
                $pattern = '\$\{(\w+)\}'
                $interpolatedValue = [regex]::Replace(
                    $value,
                    $pattern,
                    { param($m)
                        $envName = $m.Groups[1].Value
                        [System.Environment]::GetEnvironmentVariable($envName, 'Process')
                    }
                )
                [System.Environment]::SetEnvironmentVariable($key, $interpolatedValue, "Process")
                Write-Host "Set environment variable: $key=$interpolatedValue"
            } else {
                
            # Export as environment variable
                [System.Environment]::SetEnvironmentVariable($key, $value, "Process")
                Write-Host "Set environment variable: $key=$value"
            }
        }
    }
}

# Run docker-compose with the loaded environment variables
docker compose build
docker-compose up