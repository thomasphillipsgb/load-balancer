# PowerShell script to launch both load_balancer and worker binaries
# This script builds the project and runs both services in separate Windows

param(
    [switch]$BuildFirst = $true,
    [string]$LoadBalancerPort = "1337",
    [string]$WorkerPort = "3000",
    [int]$NumberOfWorkers = 2
)

# Set the project root
$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$debugDir = Join-Path $projectRoot "target\debug"

Write-Host "Load Balancer Service Launcher" -ForegroundColor Cyan
Write-Host "==============================`n" -ForegroundColor Cyan

# Build the project if requested
if ($BuildFirst) {
    Write-Host "Building project..." -ForegroundColor Yellow
    Push-Location $projectRoot
    cargo build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Build failed!" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    Pop-Location
    Write-Host "Build completed successfully`n" -ForegroundColor Green
}

# Verify binaries exist
$loadBalancerBinary = Join-Path $debugDir "load-balancer.exe"
$workerBinary = Join-Path $debugDir "worker.exe"

if (-not (Test-Path $loadBalancerBinary)) {
    Write-Host "Load balancer binary not found at: $loadBalancerBinary" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $workerBinary)) {
    Write-Host "Worker binary not found at: $workerBinary" -ForegroundColor Red
    exit 1
}

Write-Host "Launching services..." -ForegroundColor Yellow
Write-Host "Load Balancer: $loadBalancerBinary (Port: $LoadBalancerPort)" -ForegroundColor Cyan
for ($i = 0; $i -lt $NumberOfWorkers; $i++) {
    $workerPortNum = [int]$WorkerPort + $i
    Write-Host "Worker $($i + 1): $workerBinary (Port: $workerPortNum)" -ForegroundColor Cyan
}
Write-Host ""

# Launch load_balancer in a new PowerShell window
$loadBalancerProcess = Start-Process powershell -ArgumentList "-NoExit", "-Command", "&'$loadBalancerBinary'" -PassThru -WindowStyle Normal
Write-Host "Load Balancer started (PID: $($loadBalancerProcess.Id))" -ForegroundColor Green

# Small delay to avoid startup conflicts
Start-Sleep -Milliseconds 500

# Launch multiple worker instances in new PowerShell windows
$workerProcesses = @()
for ($i = 0; $i -lt $NumberOfWorkers; $i++) {
    $workerPortNum = [int]$WorkerPort + $i
    $workerProcess = Start-Process powershell -ArgumentList "-NoExit", "-Command", "&'$workerBinary' '$workerPortNum'" -PassThru -WindowStyle Normal
    $workerProcesses += $workerProcess
    Write-Host "Worker $($i + 1) started (PID: $($workerProcess.Id), Port: $workerPortNum)" -ForegroundColor Green
    Start-Sleep -Milliseconds 300
}

Write-Host ""
Write-Host "Services running in separate windows." -ForegroundColor Green
Write-Host "Close the individual windows to stop each service." -ForegroundColor Gray
Write-Host ""
Write-Host "Process Info:" -ForegroundColor Cyan
Write-Host "  Load Balancer PID: $($loadBalancerProcess.Id)" -ForegroundColor Gray
for ($i = 0; $i -lt $workerProcesses.Count; $i++) {
    $workerPortNum = [int]$WorkerPort + $i
    Write-Host "  Worker $($i + 1) PID: $($workerProcesses[$i].Id) (Port: $workerPortNum)" -ForegroundColor Gray
}

# Wait for all processes
$loadBalancerProcess | Wait-Process
$workerProcesses | Wait-Process
Write-Host ""
Write-Host "All services have stopped." -ForegroundColor Yellow
