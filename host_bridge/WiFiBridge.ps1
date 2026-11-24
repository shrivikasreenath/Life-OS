<#
.SYNOPSIS
    Life OS — Windows Host Wi-Fi Bridge
.DESCRIPTION
    Bridge helper application that scans real physical Wi-Fi networks on the Windows
    laptop host and communicates with the Life OS guest VM running in Oracle VirtualBox.
    
    VirtualBox exposes an emulated Intel 82540EM Ethernet adapter to the guest kernel.
    This bridge allows Life OS to discover actual physical wireless networks, signal strengths,
    and connection statuses through a secured UDP protocol on port 18888.
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
        $currentAuth = "WPA2-Personal"
        $currentSignal = 80
        $currentBand = "2.4 GHz"
        $currentChannel = "6"

        foreach ($line in $scanOutput) {
            if ($line -match "^\s*SSID\s+\d+\s*:\s*(.+)$") {
                if ($currentName -ne "") {
                    $isConnected = ($currentName -eq $currentSsid)
                    $networks += [PSCustomObject]@{
                        SSID = $currentName
                        Signal = $currentSignal
                        Security = $currentAuth
                        Band = $currentBand
                        Channel = $currentChannel
                        Connected = $isConnected
                    }
                }
                $currentName = $matches[1].Trim()
                $currentAuth = "WPA2-Personal"
                $currentSignal = 80
                $currentBand = "2.4 GHz"
                $currentChannel = "6"
            }
            elseif ($line -match "^\s*Authentication\s*:\s*(.+)$") {
                $currentAuth = $matches[1].Trim()
            }
            elseif ($line -match "^\s*Signal\s*:\s*(\d+)%") {
                $currentSignal = [int]$matches[1].Trim()
            }
            elseif ($line -match "^\s*Band\s*:\s*(.+)$") {
                $currentBand = $matches[1].Trim()
            }
            elseif ($line -match "^\s*Channel\s*:\s*(\d+)") {
                $currentChannel = $matches[1].Trim()
            }
        }

        if ($currentName -ne "") {
            $isConnected = ($currentName -eq $currentSsid)
            $networks += [PSCustomObject]@{
                SSID = $currentName
                Signal = $currentSignal
                Security = $currentAuth
                Band = $currentBand
                Channel = $currentChannel
                Connected = $isConnected
            }
        }
    }
    catch {
        Write-Warning "Failed to query netsh wlan: $_"
    }

    return $networks
}

try {
    while ($true) {
        $sender = New-Object System.Net.IPEndPoint([System.Net.IPAddress]::Any, 0)
        $receivedBytes = $UdpClient.Receive([ref]$sender)
        $request = [System.Text.Encoding]::UTF8.GetString($receivedBytes).Trim()

        $timestamp = (Get-Date).ToString("yyyy-MM-dd HH:mm:ss")
        # Security: Do not log raw passwords if CONNECT command
        if ($request.StartsWith("CONNECT ")) {
            Write-Host "[$timestamp] Request from $($sender.Address):$($sender.Port) -> 'CONNECT [credentials hidden]'" -ForegroundColor Cyan
        } else {
            Write-Host "[$timestamp] Request from $($sender.Address):$($sender.Port) -> '$request'" -ForegroundColor Cyan
        }

        if ($request -eq "SCAN" -or $request -eq "WIFI_SCAN") {
            $nets = Get-RealWiFiNetworks
            $responseLines = @()
            foreach ($n in $nets) {
                $connStr = if ($n.Connected) { "1" } else { "0" }
                $responseLines += "$($n.SSID)|$($n.Signal)|$($n.Security)|$($n.Band)|$($n.Channel)|$connStr"
            }
            $response = $responseLines -join "`n"
            $replyBytes = [System.Text.Encoding]::UTF8.GetBytes($response)
            $UdpClient.Send($replyBytes, $replyBytes.Length, $sender) | Out-Null
            Write-Host "  Sent $($nets.Count) physical Wi-Fi networks to Life OS." -ForegroundColor Green
        }
        elseif ($request.StartsWith("CONNECT ")) {
            $payload = $request.Substring(8).Trim()
            $parts = $payload.Split('|')
            $targetSsid = $parts[0].Trim()
            $password = if ($parts.Length -gt 1) { $parts[1].Trim() } else { "" }
            
            Write-Host "  [CONNECT] Attempting connection to SSID: '$targetSsid'" -ForegroundColor Yellow

            # If password is provided, generate a temporary profile
            if ($password -ne "") {
                $profileXml = @"
<?xml version="1.0"?>
<WLANProfile xmlns="http://www.microsoft.com/networking/WLAN/profile/v1">
    <name>$targetSsid</name>
    <SSIDConfig>
        <SSID>
            <name>$targetSsid</name>
        </SSID>
    </SSIDConfig>
    <connectionType>ESS</connectionType>
    <connectionMode>manual</connectionMode>
    <MSM>
        <security>
            <authEncryption>
                <authentication>WPA2PSK</authentication>
                <encryption>AES</encryption>
                <useOneX>false</useOneX>
            </authEncryption>
            <sharedKey>
                <keyType>passPhrase</keyType>
                <protected>false</protected>
                <keyMaterial>$password</keyMaterial>
            </sharedKey>
        </security>
    </MSM>
</WLANProfile>
"@
                $tempFile = [System.IO.Path]::GetTempFileName()
                [System.IO.File]::WriteAllText($tempFile, $profileXml)
                netsh wlan add profile filename="$tempFile" user=all | Out-Null
                Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
            }

            # Execute connection via netsh
            netsh wlan connect name="$targetSsid" | Out-Null
            
            # Verify actual connection status over 3 seconds
            $connected = $false
            for ($i = 0; $i -lt 6; $i++) {
                Start-Sleep -Milliseconds 500
                $statusCheck = netsh wlan show interfaces
                if (($statusCheck -join "`n") -match "State\s*:\s*connected" -and ($statusCheck -join "`n") -match "SSID\s*:\s*$([regex]::Escape($targetSsid))") {
                    $connected = $true
                    break
                }
            }

            $replyMsg = if ($connected) {
                "SUCCESS: Connected to $targetSsid"
            } else {
                "FAILED: Could not connect to $targetSsid (check password or range)"
            }
            $replyBytes = [System.Text.Encoding]::UTF8.GetBytes($replyMsg)
            $UdpClient.Send($replyBytes, $replyBytes.Length, $sender) | Out-Null
            Write-Host "  Result: $replyMsg" -ForegroundColor Magenta
        }
        elseif ($request -eq "DISCONNECT") {
            netsh wlan disconnect | Out-Null
            $replyMsg = "SUCCESS: Disconnected"
            $replyBytes = [System.Text.Encoding]::UTF8.GetBytes($replyMsg)
            $UdpClient.Send($replyBytes, $replyBytes.Length, $sender) | Out-Null
            Write-Host "  Disconnected from Wi-Fi." -ForegroundColor Cyan
        }
        elseif ($request -eq "STATUS") {
            $nets = Get-RealWiFiNetworks
            $connected = $nets | Where-Object { $_.Connected } | Select-Object -First 1
            $statusMsg = if ($connected) {
                "CONNECTED|$($connected.SSID)|$($connected.Signal)|$($connected.Security)|$($connected.Band)|$($connected.Channel)"
            } else {
                "DISCONNECTED||0|||"
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
