# Read environment variables
$STEAM_API_KEY = $Env:STEAM_API_KEY
rustc scripts\actions\double_slash.rs
Set-Variable -Name "PROJECT_PATH" -Value $(.\double_slash.exe "$Env:PROJECT_PATH")
Set-Variable -Name "TEST_PATH" -Value $(.\double_slash.exe "$Env:PROJECT_PATH\\crates\\gss-tests")

# Create file contents
$envContent = @"
STEAM_API_KEY="$STEAM_API_KEY"
PROJECT_PATH="$PROJECT_PATH"
TEST_PATH="$TEST_PATH"
RECIPIENT_EMAIL=""
SMTP_HOST=""
SMTP_PORT=0
SMTP_EMAIL=""
SMTP_USERNAME=""
SMTP_PWD=""
"@

# Write to .env file
Set-Content -Path ".env" -Value $envContent -Encoding UTF8