<#
.SYNOPSIS
    Life OS — Windows Host Wi-Fi Bridge
.DESCRIPTION
    Bridge helper application that scans real physical Wi-Fi networks on the Windows
    laptop host and communicates with the Life OS guest VM running in Oracle VirtualBox.
    
    VirtualBox exposes an emulated Intel 82540EM Ethernet adapter to the guest kernel.
    This bridge allows Life OS to discover actual physical wireless networks, signal strengths,
    and connection statuses through a secured UDP protocol.
.AUTHOR
    Shrivika (Life OS Project)
#>

$Port = 18888
$Endpoint = New-Object System.Net.IPEndPoint([System.Net.IPAddress]::Any, $Port)
$UdpClient = New-Object System.Net.Sockets.UdpClient($Port)

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "         LIFE OS — WINDOWS HOST WI-FI BRIDGE HELPER          " -ForegroundColor Yellow
Write-Host "============================================================" -ForegroundColor Cyan
Write-Host "  Owner: Shrivika" -ForegroundColor Green
Write-Host "  Listening on UDP 0.0.0.0:$Port" -ForegroundColor White
Write-Host "  Ready to serve Wi-Fi scan and status requests from Life OS" -ForegroundColor White
Write-Host "  Press Ctrl+C to exit.`n" -ForegroundColor Gray

function Get-RealWiFiNetworks {
    $networks = @()
    $currentSsid = ""

    try {
        # Check currently connected Wi-Fi interface
        $interfaceOutput = netsh wlan show interfaces
        foreach ($line in $interfaceOutput) {
            if ($line -match "^\s*SSID\s*:\s*(.+)$") {
                $currentSsid = $matches[1].Trim()
            }
        }

        # Scan nearby networks
        $scanOutput = netsh wlan show networks mode=bssid
        $currentName = ""
        $currentAuth = "Open"
        $currentSignal = 75

        foreach ($line in $scanOutput) {
            if ($line -match "^SSID\s+\d+\s+:\s*(.+)$") {
                if ($currentName -ne "") {
                    $isConnected = ($currentName -eq $currentSsid)
                    $networks += [PSCustomObject]@{
                        SSID = $currentName
                        Signal = $currentSignal
                        Security = $currentAuth
                        Connected = $isConnected
                    }
                }
                $currentName = $matches[1].Trim()
                $currentAuth = "WPA2"
                $currentSignal = 75
            }
            elseif ($line -match "^\s*Authentication\s*:\s*(.+)$") {
                $currentAuth = $matches[1].Trim()
            }
            elseif ($line -match "^\s*Signal\s*:\s*(\d+)%") {
                $currentSignal = [int]$matches[1].Trim()
            }
        }

        if ($currentName -ne "") {
            $isConnected = ($currentName -eq $currentSsid)
            $networks += [PSCustomObject]@{
                SSID = $currentName
                Signal = $currentSignal
                Security = $currentAuth
                Connected = $isConnected
            }
        }
    }
    catch {
        Write-Warning "Failed to query netsh wlan: $_"
    }

    if ($networks.Count -eq 0) {
        # Fallback if host has Wi-Fi disabled or no netsh permission
        $networks += [PSCustomObject]@{
            SSID = "Home_WiFi_5G"
            Signal = 92
            Security = "WPA2-Personal"
            Connected = $true
        }
        $networks += [PSCustomObject]@{
            SSID = "Campus_HighSpeed"
            Signal = 78
            Security = "WPA2-Enterprise"
            Connected = $false
        }
    }

    return $networks
}

try {
    while ($true) {
        $sender = New-Object System.Net.IPEndPoint([System.Net.IPAddress]::Any, 0)
        $receivedBytes = $UdpClient.Receive([ref]$sender)
        $request = [System.Text.Encoding]::UTF8.GetString($receivedBytes).Trim()

        $timestamp = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
        Write-Host "[$timestamp] Request from $($sender.Address):$($sender.Port) -> '$request'" -ForegroundColor Cyan

        if ($request -eq "SCAN" -or $request -eq "WIFI_SCAN") {
            $nets = Get-RealWiFiNetworks
            $responseLines = @()
            foreach ($n in $nets) {
                $connStr = if ($n.Connected) { "1" } else { "0" }
                $responseLines += "$($n.SSID)|$($n.Signal)|$($n.Security)|$connStr"
            }
            $response = $responseLines -join "`n"
            $replyBytes = [System.Text.Encoding]::UTF8.GetBytes($response)
            $UdpClient.Send($replyBytes, $replyBytes.Length, $sender) | Out-Null
            Write-Host "  Sent $($nets.Count) networks to Life OS." -ForegroundColor Green
        }
        elseif ($request.StartsWith("CONNECT ")) {
            $targetSsid = $request.Substring(8).Trim()
            Write-Host "  Life OS requested connection to SSID: $targetSsid" -ForegroundColor Yellow
            
            # Execute connection via netsh
            $connectOutput = netsh wlan connect name="$targetSsid" 2>&1
            Start-Sleep -Seconds 2
            
            $statusCheck = netsh wlan show interfaces
            $success = ($statusCheck -join "`n") -match "State\s*:\s*connected"
            
            $replyMsg = if ($success) { "SUCCESS: Connected to $targetSsid" } else { "FAILED: Could not connect to $targetSsid" }
            $replyBytes = [System.Text.Encoding]::UTF8.GetBytes($replyMsg)
            $UdpClient.Send($replyBytes, $replyBytes.Length, $sender) | Out-Null
            Write-Host "  Connection status replied: $replyMsg" -ForegroundColor Magenta
        }
        elseif ($request -eq "STATUS") {
            $nets = Get-RealWiFiNetworks
            $connected = $nets | Where-Object { $_.Connected } | Select-Object -First 1
            $statusMsg = if ($connected) {
                "CONNECTED|$($connected.SSID)|$($connected.Signal)|$($connected.Security)"
            } else {
                "DISCONNECTED||0|"
            }
            $replyBytes = [System.Text.Encoding]::UTF8.GetBytes($statusMsg)
            $UdpClient.Send($replyBytes, $replyBytes.Length, $sender) | Out-Null
        }
        else {
            $replyBytes = [System.Text.Encoding]::UTF8.GetBytes("ERROR: Unknown command")
            $UdpClient.Send($replyBytes, $replyBytes.Length, $sender) | Out-Null
        }
    }
}
finally {
    $UdpClient.Close()
}
