# Read environment variables
$STEAM_API_KEY = $Env:STEAM_API_KEY
$PROJECT_PATH = $Env:PROJECT_PATH

# Create file contents
$envContent = @"STEAM_API_KEY="$STEAM_API_KEY"
PROJECT_PATH="$PROJECT_PATH"
TEST_PATH="$PROJECT_PATH/src/tests""@

# Write to .env file
Set-Content -Path ".env" -Value $envContent -Encoding UTF8