# Go to script directory
Set-Location -Path $PSScriptRoot

# Create log directory
$LogFolder = "$PSScriptRoot\..\logs"
if(-not(Test-Path -path $LogFolder) -and (Test-Path $LogFolder -IsValid)){
    New-Item -Path $LogFolder -ItemType Directory
}

# Create email subdirectory in logs\
$JobsSubFolder = "$LogFolder\jobs"
if(-not(Test-Path -path $JobsSubFolder) -and (Test-Path $JobsSubFolder -IsValid)){
    New-Item -Path $JobsSubFolder -ItemType Directory
}

# Check if target directory exists
$TargetFolder = "$PSScriptRoot\..\target\release"
if(-not(Test-Path -path $TargetFolder) -and (Test-Path $TargetFolder -IsValid)){
    cargo build --release
}

# Run script to get game sales
$DateTime = (Get-Date -Format "yyyy_MM_dd_HH_mm")
Set-Location -Path ..
.\target\release\gss-cli.exe --send-email | Out-String | Set-Content $JobsSubFolder\$DateTime"_email.html"