# Read environment variables
$STEAM_API_KEY = $Env:STEAM_API_KEY
$PROJECT_PATH = $Env:PROJECT_PATH

# Create file contents
$envContent = @"
STEAM_API_KEY="$STEAM_API_KEY"
PROJECT_PATH="$PROJECT_PATH"
"@

# Write to .env file
Set-Content -Path ".env" -Value $envContent -Encoding UTF8