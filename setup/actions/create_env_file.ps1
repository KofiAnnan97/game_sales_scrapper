# Read environment variables
$STEAM_API_KEY = $Env:STEAM_API_KEY
Set-Variable -Name "PROJECT_PATH" -Value $(python3 .\setup\actions\double_slash.py "$Env:PROJECT_PATH")
Set-Variable -Name "TEST_PATH" -Value $(python3 .\setup\actions\double_slash.py "$Env:PROJECT_PATH\\src\\tests")

# Create file contents
$envContent = @"
STEAM_API_KEY="$STEAM_API_KEY"
PROJECT_PATH="$PROJECT_PATH"
TEST_PATH="$TEST_PATH"
"@

# Write to .env file
Set-Content -Path ".env" -Value $envContent -Encoding UTF8