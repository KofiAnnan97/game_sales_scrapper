$trigger = New-ScheduledTaskTrigger `
    -Weekly -DaysOfWeek Tuesday,Thursday -At 1pm
$action = New-ScheduledTaskAction `
    -Execute "Powershell.exe" `
    -Argument "-NoProfile -ExecutionPolicy Bypass -File $PSScriptRoot\send_email.ps1" 
$description = "Runs a script to see if any games have reached a desired price. If yes, an email is sent."
$settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
Register-ScheduledTask -TaskName 'steam-sales-scrapper' `
    -Action $action `
    -Trigger $trigger `
    -Description $description `
    -Settings $settings