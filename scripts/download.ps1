$env:RUST_LOG = "lolicon=debug"

while ($true) {
    $process = Start-Process -FilePath "./target/release/lolicon.exe" -PassThru -NoNewWindow
    $process.WaitForExit()

    if ($process.ExitCode -ne 0) {
        Write-Host "scrap failed!"
        exit 1
    }
    else {
        Write-Host "----- scrap end! -----"
        Start-Sleep -Seconds 5
    }
}
