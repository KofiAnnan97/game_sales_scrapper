# Go to script directory
Set-Location -Path $PSScriptRoot

# Create log directory
$LogFolder = "$PSScriptRoot\log"
if(-not(Test-Path -path $LogFolder) -and (Test-Path $LogFolder -IsValid)){
    New-Item -Path $LogFolder -ItemType Directory
}

# Run script to get game sales
$DateTime = (Get-Date -Format "yyyy_MM_dd_HH_mm")
cargo run -- --send-email| Out-String | Set-Content $LogFolder\$DateTime"_email.log"