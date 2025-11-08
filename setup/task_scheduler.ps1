param(
    [Parameter(Mandatory=$true)]
    [string]$Cmd
)

$taskName = 'game-sales-scrapper'

function task-exists($taskName) {
    $exists = Get-ScheduledTask | Where-Object {$_.TaskName -like $taskName} | Out-String
    if($exists -ne "") {
        return $true
    }
    else {
        return $false
    }
}

function create-task($taskName) {
    $taskExists = task-exists $taskName
    if($taskExists){
        Write-Output "Task '$taskName' already exists"
    }
    else{
        $trigger = New-ScheduledTaskTrigger `
            -Weekly -DaysOfWeek Tuesday,Thursday -At 1pm
        $action = New-ScheduledTaskAction `
            -Execute "Powershell.exe" `
            -Argument "-NoProfile -ExecutionPolicy Bypass -File $PSScriptRoot\send_email.ps1"
        $description = "Runs a script to see if any games have reached a desired price. If yes, an email is sent."
        $settings = New-ScheduledTaskSettingsSet -AllowStartIfOnBatteries -DontStopIfGoingOnBatteries -StartWhenAvailable
        Register-ScheduledTask -TaskName $taskName `
            -Action $action `
            -Trigger $trigger `
            -Description $description `
            -Settings $settings

        Write-Output "`nTask '$taskName' successfully created."
    }
}

function toggle-task-status($taskName, $enable) {
    $taskExists = task-exists $taskName
    Write-Output $taskExists
    if($taskExists){
        if($enable){
            Enable-ScheduledTask -TaskName $taskName
            Write-Output "`nTask '$taskName' is enabled."
        }
        else{
            Disable-ScheduledTask -TaskName $taskName
            Write-Output "`nTask '$taskName' is disabled."
        }
    }
    else {
        Write-Output "Task '$taskName' does not exist. Cannot be enabled/disabled."
    }
}

function delete-task($taskName) {
    $taskExists = task-exists $taskName
    if($taskExists){
        Unregister-ScheduledTask -TaskName $taskName -Confirm:$false
        Write-Output "Task '$taskName' successfully deleted."
    }
    else {
        Write-Output "Task '$taskName' does not exist."
    }
}

# Decision Logic
if($Cmd -eq "create"){
    create-task $taskName
}
elseif($Cmd -eq "enable"){
    toggle-task-status $taskName $true
}
elseif($Cmd -eq "disable"){
    toggle-task-status $taskName $false
}
elseif($Cmd -eq "delete"){
    delete-task $taskName
}
else {
    Write-Output "Unrecognized -Cmd: '$Cmd'.`nValid -Cmd inputs include [create, enable, disable, delete]."
}