# Go to script directory
Set-Location -Path $PSScriptRoot

# Create log directory for Windows
$logFolder = "$PSScriptRoot\log\windows"
if(-not(Test-Path -path $logFolder) -and (Test-Path $logFolder -IsValid)){
    New-Item -Path "$PSScriptRoot\log\windows" -ItemType Directory
}

# Run script to get game sales
$dateTime = (Get-Date -Format "yyyy_dd_MM_HH_mm")
cargo run -- --send-email| Out-String | Set-Content $logFolder\$dateTime"_email.log"