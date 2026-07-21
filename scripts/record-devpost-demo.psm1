#Requires -Version 7.4

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:ProfileName = 'Radia Build Week'
$script:CollectionName = 'Radia Build Week'
$script:MarkerSlot = 'radia_build_week_automation'
$script:MarkerValue = 'radia-video-v1'
$script:CollectionMarkerSlot = 'radia_build_week_collection_automation'
$script:FinalScenes = @('Renderer', 'AEP Proof', 'Math Proof', 'Buffer Modes', 'Evidence', 'Closing')
$script:AllScenes = @('Narration') + $script:FinalScenes
$script:ModeLabelInput = 'Radia Mode Label'
$script:WindowInput = 'Radia Window'
$script:NarrationInput = 'Radia Narration Track'
$script:MicInput = 'Radia Narration Mic'
$script:TeleprompterInput = 'Radia Teleprompter'
$script:WindowSelector = 'Radia - Jade Dragon Triad:Window Class:radia-demo.exe'
$script:WindowsGraphicsCaptureMethod = 2
$script:MinimumPreviewBytes = 16384

function New-RadiaWindowCaptureSettings {
    [CmdletBinding()]
    [OutputType([hashtable])]
    param()

    return @{
        window = $script:WindowSelector
        priority = 2
        cursor = $false
        client_area = $true
        method = $script:WindowsGraphicsCaptureMethod
    }
}

function Get-RadiaProjectRoot {
    [CmdletBinding()]
    [OutputType([string])]
    param()

    return [System.IO.Path]::GetFullPath((Join-Path -Path $PSScriptRoot -ChildPath '..'))
}

function Get-RadiaTimeline {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)]
        [string] $Path
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Get-RadiaTimeline: failed_input='Path' value='$Path' skipped='timeline load'"
    }
    $resolved = (Resolve-Path -LiteralPath $Path).Path
    $timeline = Get-Content -Raw -LiteralPath $resolved | ConvertFrom-Json -Depth 32
    Test-RadiaVideoTimeline -Timeline $timeline
    return $timeline
}

function Test-RadiaVideoTimeline {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory, ValueFromPipeline)]
        [pscustomobject] $Timeline
    )

    process {
        if ([int]$Timeline.schema_version -ne 1) {
            throw "Test-RadiaVideoTimeline: failed_input='schema_version' skipped='timeline validation'"
        }
        if ([int]$Timeline.canvas.width -ne 1920 -or [int]$Timeline.canvas.height -ne 1080) {
            throw "Test-RadiaVideoTimeline: failed_input='canvas' skipped='timeline validation'"
        }
        if ([int]$Timeline.canvas.fps_numerator -ne 30 -or [int]$Timeline.canvas.fps_denominator -ne 1) {
            throw "Test-RadiaVideoTimeline: failed_input='fps' skipped='timeline validation'"
        }
        $normalStop = [int]$Timeline.normal_stop_ms
        $hardStop = [int]$Timeline.hard_stop_ms
        if ($normalStop -lt 150000 -or $normalStop -gt 165000) {
            throw "Test-RadiaVideoTimeline: failed_input='normal_stop_ms' skipped='timeline validation'"
        }
        if ($hardStop -le $normalStop -or $hardStop -ge 179000) {
            throw "Test-RadiaVideoTimeline: failed_input='hard_stop_ms' skipped='timeline validation'"
        }

        $knownScenes = [System.Collections.Generic.HashSet[string]]::new(
            [string[]]$script:FinalScenes,
            [System.StringComparer]::Ordinal
        )
        $knownCommands = '^(reset|quit|mode (triangle|off|radia|gi|albedo|normal|emissive|depth|ao|sdf|primitive|steps|hit))$'
        $seen = [System.Collections.Generic.HashSet[int]]::new()
        $previous = -1
        foreach ($cue in @($Timeline.cues)) {
            $at = [int]$cue.at_ms
            if ($at -le $previous -or -not $seen.Add($at)) {
                throw "Test-RadiaVideoTimeline: failed_input='cues.at_ms' value='$at' skipped='timeline validation'"
            }
            if ($at -ge $normalStop) {
                throw "Test-RadiaVideoTimeline: failed_input='cues.at_ms' value='$at' skipped='timeline validation'"
            }
            if (-not $knownScenes.Contains([string]$cue.scene)) {
                throw "Test-RadiaVideoTimeline: failed_input='cues.scene' value='$($cue.scene)' skipped='timeline validation'"
            }
            foreach ($command in @($cue.radia_commands)) {
                if ([string]$command -notmatch $knownCommands) {
                    throw "Test-RadiaVideoTimeline: failed_input='radia_commands' value='$command' skipped='timeline validation'"
                }
            }
            $previous = $at
        }
        if ($previous -lt 150000) {
            throw "Test-RadiaVideoTimeline: failed_input='final cue' skipped='timeline validation'"
        }

        $sectionSeen = [System.Collections.Generic.HashSet[int]]::new()
        $sectionPrevious = -1
        foreach ($section in @($Timeline.narration_sections)) {
            $at = [int]$section.at_ms
            if ($at -le $sectionPrevious -or -not $sectionSeen.Add($at)) {
                throw "Test-RadiaVideoTimeline: failed_input='narration_sections.at_ms' value='$at' skipped='timeline validation'"
            }
            if ($at -ge $normalStop) {
                throw "Test-RadiaVideoTimeline: failed_input='narration_sections.at_ms' value='$at' skipped='timeline validation'"
            }
            $sectionPrevious = $at
        }
    }
}

function Get-FileSha256 {
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Mandatory)]
        [string] $Path
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "Get-FileSha256: failed_input='Path' value='$Path' skipped='hash'"
    }
    return (Get-FileHash -Algorithm SHA256 -LiteralPath $Path).Hash.ToLowerInvariant()
}

function Get-GitCommit {
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Mandatory)]
        [string] $ProjectRoot
    )

    $commit = & git -C $ProjectRoot rev-parse HEAD
    if ($LASTEXITCODE -ne 0) {
        throw "Get-GitCommit: git rev-parse failed with exit code $LASTEXITCODE"
    }
    return ([string]$commit).Trim()
}

function Get-MediaDurationMilliseconds {
    [CmdletBinding()]
    [OutputType([long])]
    param(
        [Parameter(Mandatory)]
        [string] $Path
    )

    if (-not $IsWindows) {
        throw "Get-MediaDurationMilliseconds: failed_input='platform' skipped='media duration'"
    }
    $resolved = (Resolve-Path -LiteralPath $Path).Path
    $shell = New-Object -ComObject Shell.Application
    try {
        $folder = $shell.Namespace((Split-Path -Parent $resolved))
        if ($null -eq $folder) {
            throw "Get-MediaDurationMilliseconds: failed_input='folder' value='$resolved' skipped='media duration'"
        }
        $item = $folder.ParseName((Split-Path -Leaf $resolved))
        if ($null -eq $item) {
            throw "Get-MediaDurationMilliseconds: failed_input='file' value='$resolved' skipped='media duration'"
        }
        $duration100Ns = $item.ExtendedProperty('System.Media.Duration')
        if ($null -eq $duration100Ns) {
            throw "Get-MediaDurationMilliseconds: failed_input='System.Media.Duration' value='$resolved' skipped='media duration'"
        }
        return [long]([double]$duration100Ns / 10000.0)
    }
    finally {
        if ($null -ne $shell) {
            [void][System.Runtime.InteropServices.Marshal]::FinalReleaseComObject($shell)
        }
    }
}

function ConvertTo-ObsAuthentication {
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Mandatory)]
        [string] $Password,
        [Parameter(Mandatory)]
        [string] $Salt,
        [Parameter(Mandatory)]
        [string] $Challenge
    )

    $encoding = [System.Text.Encoding]::UTF8
    $secretBytes = [System.Security.Cryptography.SHA256]::HashData($encoding.GetBytes($Password + $Salt))
    $secret = [Convert]::ToBase64String($secretBytes)
    $authBytes = [System.Security.Cryptography.SHA256]::HashData($encoding.GetBytes($secret + $Challenge))
    return [Convert]::ToBase64String($authBytes)
}

function Send-ObsMessage {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)]
        [System.Net.WebSockets.ClientWebSocket] $Socket,
        [Parameter(Mandatory)]
        [object] $Message
    )

    $json = $Message | ConvertTo-Json -Depth 32 -Compress
    $bytes = [System.Text.Encoding]::UTF8.GetBytes($json)
    $segment = [System.ArraySegment[byte]]::new($bytes)
    [void]$Socket.SendAsync(
        $segment,
        [System.Net.WebSockets.WebSocketMessageType]::Text,
        $true,
        [System.Threading.CancellationToken]::None
    ).GetAwaiter().GetResult()
}

function Receive-ObsMessage {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)]
        [System.Net.WebSockets.ClientWebSocket] $Socket,
        [int] $TimeoutSeconds = 10
    )

    $buffer = [byte[]]::new(16384)
    $stream = [System.IO.MemoryStream]::new()
    $timeout = [System.Threading.CancellationTokenSource]::new([TimeSpan]::FromSeconds($TimeoutSeconds))
    try {
        do {
            $segment = [System.ArraySegment[byte]]::new($buffer)
            $result = $Socket.ReceiveAsync($segment, $timeout.Token).GetAwaiter().GetResult()
            if ($result.MessageType -eq [System.Net.WebSockets.WebSocketMessageType]::Close) {
                throw "Receive-ObsMessage: OBS closed WebSocket code='$($Socket.CloseStatus)' description='$($Socket.CloseStatusDescription)'"
            }
            $stream.Write($buffer, 0, $result.Count)
        } while (-not $result.EndOfMessage)
        $json = [System.Text.Encoding]::UTF8.GetString($stream.ToArray())
        return $json | ConvertFrom-Json -Depth 32
    }
    finally {
        $timeout.Dispose()
        $stream.Dispose()
    }
}

function Connect-ObsWebSocket {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)]
        [uri] $Uri,
        [Parameter(Mandatory)]
        [pscredential] $Credential
    )

    $socket = [System.Net.WebSockets.ClientWebSocket]::new()
    $timeout = [System.Threading.CancellationTokenSource]::new([TimeSpan]::FromSeconds(10))
    try {
        [void]$socket.ConnectAsync($Uri, $timeout.Token).GetAwaiter().GetResult()
    }
    catch {
        $socket.Dispose()
        throw "Connect-ObsWebSocket: failed_input='Uri' value='$Uri' skipped='OBS control'; $($_.Exception.Message)"
    }
    finally {
        $timeout.Dispose()
    }

    $hello = Receive-ObsMessage -Socket $socket
    if ([int]$hello.op -ne 0) {
        $socket.Dispose()
        throw "Connect-ObsWebSocket: expected OBS Hello opcode 0"
    }
    if ($null -eq $hello.d.authentication) {
        $socket.Dispose()
        throw "Connect-ObsWebSocket: OBS authentication is disabled; rotate the password and require authentication"
    }

    $plainPassword = $Credential.GetNetworkCredential().Password
    try {
        $authentication = ConvertTo-ObsAuthentication `
            -Password $plainPassword `
            -Salt ([string]$hello.d.authentication.salt) `
            -Challenge ([string]$hello.d.authentication.challenge)
    }
    finally {
        $plainPassword = $null
    }
    [void](Send-ObsMessage -Socket $socket -Message ([ordered]@{
        op = 1
        d = [ordered]@{
            rpcVersion = 1
            eventSubscriptions = 0
            authentication = $authentication
        }
    }))
    $identified = Receive-ObsMessage -Socket $socket
    if ([int]$identified.op -ne 2) {
        $socket.Dispose()
        throw "Connect-ObsWebSocket: OBS identification failed"
    }
    return [pscustomobject]@{
        PSTypeName = 'Radia.ObsSession'
        Socket = $socket
        RequestSequence = 0
        WebSocketVersion = [string]$hello.d.obsWebSocketVersion
        RpcVersion = [int]$identified.d.negotiatedRpcVersion
    }
}

function Disconnect-ObsWebSocket {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)]
        [pscustomobject] $Session
    )

    try {
        if ($Session.Socket.State -eq [System.Net.WebSockets.WebSocketState]::Open) {
            [void]$Session.Socket.CloseAsync(
                [System.Net.WebSockets.WebSocketCloseStatus]::NormalClosure,
                'Radia controller finished',
                [System.Threading.CancellationToken]::None
            ).GetAwaiter().GetResult()
        }
    }
    finally {
        $Session.Socket.Dispose()
    }
}

function Invoke-ObsRequest {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)]
        [pscustomobject] $Session,
        [Parameter(Mandatory)]
        [string] $RequestType,
        [object] $RequestData = $null
    )

    $Session.RequestSequence = [int]$Session.RequestSequence + 1
    $requestId = "radia-$($Session.RequestSequence)"
    $payload = [ordered]@{
        requestType = $RequestType
        requestId = $requestId
    }
    if ($null -ne $RequestData) {
        $payload.requestData = $RequestData
    }
    [void](Send-ObsMessage -Socket $Session.Socket -Message ([ordered]@{ op = 6; d = $payload }))
    while ($true) {
        $message = Receive-ObsMessage -Socket $Session.Socket
        if ([int]$message.op -ne 7) {
            continue
        }
        if ([string]$message.d.requestId -ne $requestId) {
            continue
        }
        if (-not [bool]$message.d.requestStatus.result) {
            $commentProperty = $message.d.requestStatus.PSObject.Properties['comment']
            $comment = if ($null -eq $commentProperty) { '' } else { [string]$commentProperty.Value }
            $code = [int]$message.d.requestStatus.code
            $exception = [System.InvalidOperationException]::new(
                "Invoke-ObsRequest: request='$RequestType' code='$code' cause='$comment'"
            )
            $exception.Data['ObsRequestType'] = $RequestType
            $exception.Data['ObsStatusCode'] = $code
            throw $exception
        }
        $responseProperty = $message.d.PSObject.Properties['responseData']
        if ($null -eq $responseProperty -or $null -eq $responseProperty.Value) {
            return [pscustomobject]@{}
        }
        return $responseProperty.Value
    }
}

function Start-ObsRecordingSafely {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session
    )

    $directoryResponse = Invoke-ObsRequest -Session $Session -RequestType 'GetRecordDirectory'
    $recordDirectory = [string]$directoryResponse.recordDirectory
    if (-not (Test-Path -LiteralPath $recordDirectory -PathType Container)) {
        throw "Start-ObsRecordingSafely: record directory is unavailable: '$recordDirectory'"
    }
    $existingPaths = @(
        Get-ChildItem -LiteralPath $recordDirectory -File -Filter '*.mkv' |
            ForEach-Object { $_.FullName }
    )
    $startedAtUtc = [datetime]::UtcNow
    [void](Invoke-ObsRequest -Session $Session -RequestType 'StartRecord')
    return [pscustomobject]@{
        PSTypeName = 'Radia.ObsRecordingContext'
        RecordDirectory = $recordDirectory
        ExistingPaths = [string[]]$existingPaths
        StartedAtUtc = $startedAtUtc
    }
}

function Stop-ObsRecordingSafely {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [pscustomobject] $Context
    )

    try {
        $stopped = Invoke-ObsRequest -Session $Session -RequestType 'StopRecord'
        if ([string]::IsNullOrWhiteSpace([string]$stopped.outputPath)) {
            throw 'Stop-ObsRecordingSafely: successful StopRecord omitted outputPath'
        }
        return [pscustomobject]@{
            OutputPath = [string]$stopped.outputPath
            Recovered = $false
        }
    }
    catch {
        $failure = $_
        $statusCode = $failure.Exception.Data['ObsStatusCode']
        if ($statusCode -ne 501) {
            throw
        }
        $status = Invoke-ObsRequest -Session $Session -RequestType 'GetRecordStatus'
        if ([bool]$status.outputActive) {
            throw $failure
        }
        $existing = [System.Collections.Generic.HashSet[string]]::new(
            [string[]]$Context.ExistingPaths,
            [System.StringComparer]::OrdinalIgnoreCase
        )
        $minimumWriteTime = ([datetime]$Context.StartedAtUtc).AddSeconds(-2)
        $candidates = @(
            Get-ChildItem -LiteralPath ([string]$Context.RecordDirectory) -File -Filter '*.mkv' |
                Where-Object {
                    -not $existing.Contains($_.FullName) -and
                    $_.LastWriteTimeUtc -ge $minimumWriteTime
                }
        )
        if ($candidates.Count -ne 1) {
            throw [System.InvalidOperationException]::new(
                "Stop-ObsRecordingSafely: StopRecord code 501, OBS inactive, " +
                "but new MKV candidates=$($candidates.Count); original='$($failure.Exception.Message)'"
            )
        }
        return [pscustomobject]@{
            OutputPath = $candidates[0].FullName
            Recovered = $true
        }
    }
}

function Get-ObsCredential {
    [CmdletBinding()]
    [OutputType([pscredential])]
    param(
        [pscredential] $Credential,
        [Parameter(Mandatory)] [string] $ProjectRoot
    )

    if ($null -ne $Credential) {
        return $Credential
    }
    $credentialPath = Join-Path -Path $ProjectRoot -ChildPath '.secrets\obs-websocket.credential.xml'
    if (Test-Path -LiteralPath $credentialPath -PathType Leaf) {
        $stored = Import-Clixml -LiteralPath $credentialPath
        if ($stored -isnot [pscredential]) {
            throw "Get-ObsCredential: stored file is not a PSCredential: '$credentialPath'"
        }
        return $stored
    }
    return Get-Credential -UserName 'obs' -Message 'OBS WebSocket password (kept in memory only)'
}

function Save-ObsCredential {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)] [string] $ProjectRoot
    )

    $secretDirectory = Join-Path -Path $ProjectRoot -ChildPath '.secrets'
    $credentialPath = Join-Path -Path $secretDirectory -ChildPath 'obs-websocket.credential.xml'
    [void](New-Item -ItemType Directory -Path $secretDirectory -Force)
    $credential = Get-Credential -UserName 'obs' -Message 'Save OBS WebSocket password for this Windows user and machine'
    $credential | Export-Clixml -LiteralPath $credentialPath -Depth 4 -Force
    return [pscustomobject]@{
        PSTypeName = 'Radia.ObsCredentialStore'
        Path = $credentialPath
        Protection = 'Windows DPAPI via PSCredential CLIXML'
        GitIgnored = $true
    }
}

function Resolve-ObsInputKind {
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Mandatory)] [string[]] $AvailableKinds,
        [Parameter(Mandatory)] [string] $BaseKind
    )

    if ($AvailableKinds -contains $BaseKind) {
        return $BaseKind
    }
    $escaped = [regex]::Escape($BaseKind)
    $versioned = @(
        $AvailableKinds |
            Where-Object { $_ -match "^${escaped}_v(?<version>[0-9]+)$" } |
            Sort-Object { [int]([regex]::Match($_, '_v([0-9]+)$').Groups[1].Value) } -Descending
    )
    if ($versioned.Count -eq 0) {
        throw "Resolve-ObsInputKind: OBS does not expose '$BaseKind'"
    }
    return $versioned[0]
}

function Set-ObsProfileParameter {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [string] $Category,
        [Parameter(Mandatory)] [string] $Name,
        [Parameter(Mandatory)] [string] $Value
    )

    [void](Invoke-ObsRequest -Session $Session -RequestType 'SetProfileParameter' -RequestData ([ordered]@{
        parameterCategory = $Category
        parameterName = $Name
        parameterValue = $Value
    }))
}

function Enter-RadiaObsContext {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session
    )

    $profiles = Invoke-ObsRequest -Session $Session -RequestType 'GetProfileList'
    $collections = Invoke-ObsRequest -Session $Session -RequestType 'GetSceneCollectionList'
    $profileExists = @($profiles.profiles) -contains $script:ProfileName
    if ($profileExists) {
        [void](Invoke-ObsRequest -Session $Session -RequestType 'SetCurrentProfile' -RequestData @{ profileName = $script:ProfileName })
        $marker = Invoke-ObsRequest -Session $Session -RequestType 'GetPersistentData' -RequestData @{
            realm = 'OBS_WEBSOCKET_DATA_REALM_PROFILE'
            slotName = $script:MarkerSlot
        }
        if ([string]$marker.slotValue -ne $script:MarkerValue) {
            if ([string]$profiles.currentProfileName -ne $script:ProfileName) {
                [void](Invoke-ObsRequest -Session $Session -RequestType 'SetCurrentProfile' -RequestData @{
                    profileName = [string]$profiles.currentProfileName
                })
            }
            throw "Enter-RadiaObsContext: reserved profile '$($script:ProfileName)' is not owned by Radia automation"
        }
    }
    else {
        [void](Invoke-ObsRequest -Session $Session -RequestType 'CreateProfile' -RequestData @{ profileName = $script:ProfileName })
        [void](Invoke-ObsRequest -Session $Session -RequestType 'SetPersistentData' -RequestData @{
            realm = 'OBS_WEBSOCKET_DATA_REALM_PROFILE'
            slotName = $script:MarkerSlot
            slotValue = $script:MarkerValue
        })
    }

    $collectionExists = @($collections.sceneCollections) -contains $script:CollectionName
    if ($collectionExists) {
        $collectionMarker = Invoke-ObsRequest -Session $Session -RequestType 'GetPersistentData' -RequestData @{
            realm = 'OBS_WEBSOCKET_DATA_REALM_GLOBAL'
            slotName = $script:CollectionMarkerSlot
        }
        if ([string]$collectionMarker.slotValue -ne $script:MarkerValue) {
            if ([string]$profiles.currentProfileName -ne $script:ProfileName) {
                [void](Invoke-ObsRequest -Session $Session -RequestType 'SetCurrentProfile' -RequestData @{
                    profileName = [string]$profiles.currentProfileName
                })
            }
            throw "Enter-RadiaObsContext: reserved collection '$($script:CollectionName)' is not owned by Radia automation"
        }
        [void](Invoke-ObsRequest -Session $Session -RequestType 'SetCurrentSceneCollection' -RequestData @{ sceneCollectionName = $script:CollectionName })
    }
    else {
        [void](Invoke-ObsRequest -Session $Session -RequestType 'CreateSceneCollection' -RequestData @{ sceneCollectionName = $script:CollectionName })
        [void](Invoke-ObsRequest -Session $Session -RequestType 'SetPersistentData' -RequestData @{
            realm = 'OBS_WEBSOCKET_DATA_REALM_GLOBAL'
            slotName = $script:CollectionMarkerSlot
            slotValue = $script:MarkerValue
        })
    }

    return [pscustomobject]@{
        PSTypeName = 'Radia.ObsContext'
        OriginalProfile = [string]$profiles.currentProfileName
        OriginalCollection = [string]$collections.currentSceneCollectionName
        ProfileCreated = -not $profileExists
        CollectionCreated = -not $collectionExists
    }
}

function Exit-RadiaObsContext {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [pscustomobject] $Context
    )

    if ([string]$Context.OriginalCollection -ne $script:CollectionName) {
        [void](Invoke-ObsRequest -Session $Session -RequestType 'SetCurrentSceneCollection' -RequestData @{
            sceneCollectionName = [string]$Context.OriginalCollection
        })
    }
    if ([string]$Context.OriginalProfile -ne $script:ProfileName) {
        [void](Invoke-ObsRequest -Session $Session -RequestType 'SetCurrentProfile' -RequestData @{
            profileName = [string]$Context.OriginalProfile
        })
    }
}

function Set-RadiaObsProfile {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [string] $OutputDirectory
    )

    [void](Invoke-ObsRequest -Session $Session -RequestType 'SetVideoSettings' -RequestData @{
        fpsNumerator = 30
        fpsDenominator = 1
        baseWidth = 1920
        baseHeight = 1080
        outputWidth = 1920
        outputHeight = 1080
    })
    foreach ($setting in @(
        @('Output', 'Mode', 'Simple'),
        @('SimpleOutput', 'FilePath', $OutputDirectory),
        @('SimpleOutput', 'RecFormat2', 'mkv'),
        @('SimpleOutput', 'RecEncoder', 'nvenc'),
        @('SimpleOutput', 'RecQuality', 'HQ'),
        @('SimpleOutput', 'NVENCPreset2', 'p5'),
        @('SimpleOutput', 'ABitrate', '192'),
        @('SimpleOutput', 'RecAudioEncoder', 'aac'),
        @('Video', 'ColorFormat', 'NV12'),
        @('Video', 'ColorSpace', '709'),
        @('Video', 'ColorRange', 'Partial'),
        @('Video', 'AutoRemux', 'true'),
        @('Audio', 'SampleRate', '48000'),
        @('Audio', 'ChannelSetup', 'Stereo')
    )) {
        Set-ObsProfileParameter -Session $Session -Category $setting[0] -Name $setting[1] -Value $setting[2]
    }
}

function Get-ObsInputMap {
    [CmdletBinding()]
    [OutputType([hashtable])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session
    )

    $map = @{}
    $response = Invoke-ObsRequest -Session $Session -RequestType 'GetInputList'
    foreach ($obsInput in @($response.inputs)) {
        $map[[string]$obsInput.inputName] = [string]$obsInput.inputKind
    }
    return $map
}

function Ensure-ObsScene {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [string] $Name
    )

    $response = Invoke-ObsRequest -Session $Session -RequestType 'GetSceneList'
    $names = @($response.scenes | ForEach-Object { [string]$_.sceneName })
    if ($names -contains $Name) {
        return
    }
    if ($Name -eq 'Renderer' -and $names -contains 'Scene') {
        [void](Invoke-ObsRequest -Session $Session -RequestType 'SetSceneName' -RequestData @{
            sceneName = 'Scene'
            newSceneName = 'Renderer'
        })
        return
    }
    [void](Invoke-ObsRequest -Session $Session -RequestType 'CreateScene' -RequestData @{ sceneName = $Name })
}

function Ensure-ObsInput {
    [CmdletBinding()]
    [OutputType([int])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [string] $Scene,
        [Parameter(Mandatory)] [string] $Name,
        [Parameter(Mandatory)] [string] $Kind,
        [Parameter(Mandatory)] [object] $Settings
    )

    $inputs = Get-ObsInputMap -Session $Session
    if ($inputs.ContainsKey($Name)) {
        if ([string]$inputs[$Name] -ne $Kind) {
            throw "Ensure-ObsInput: reserved input '$Name' has kind '$($inputs[$Name])', expected '$Kind'"
        }
        [void](Invoke-ObsRequest -Session $Session -RequestType 'SetInputSettings' -RequestData @{
            inputName = $Name
            inputSettings = $Settings
            overlay = $true
        })
        $item = Ensure-ObsSceneItem -Session $Session -Scene $Scene -Source $Name
        return $item
    }
    $created = Invoke-ObsRequest -Session $Session -RequestType 'CreateInput' -RequestData @{
        sceneName = $Scene
        inputName = $Name
        inputKind = $Kind
        inputSettings = $Settings
        sceneItemEnabled = $true
    }
    return [int]$created.sceneItemId
}

function Ensure-ObsSceneItem {
    [CmdletBinding()]
    [OutputType([int])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [string] $Scene,
        [Parameter(Mandatory)] [string] $Source
    )

    $items = Invoke-ObsRequest -Session $Session -RequestType 'GetSceneItemList' -RequestData @{ sceneName = $Scene }
    foreach ($item in @($items.sceneItems)) {
        if ([string]$item.sourceName -eq $Source) {
            return [int]$item.sceneItemId
        }
    }
    $created = Invoke-ObsRequest -Session $Session -RequestType 'CreateSceneItem' -RequestData @{
        sceneName = $Scene
        sourceName = $Source
        sceneItemEnabled = $true
    }
    return [int]$created.sceneItemId
}

function Set-ObsFullCanvasTransform {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [string] $Scene,
        [Parameter(Mandatory)] [int] $SceneItemId
    )

    [void](Invoke-ObsRequest -Session $Session -RequestType 'SetSceneItemTransform' -RequestData @{
        sceneName = $Scene
        sceneItemId = $SceneItemId
        sceneItemTransform = @{
            positionX = 0.0
            positionY = 0.0
            alignment = 5
            boundsType = 'OBS_BOUNDS_STRETCH'
            boundsAlignment = 5
            boundsWidth = 1920.0
            boundsHeight = 1080.0
        }
    })
}

function New-DeckUri {
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Mandatory)] [string] $DeckPath,
        [Parameter(Mandatory)] [string] $Slide,
        [Parameter(Mandatory)] [string] $Revision,
        [string] $Section = ''
    )

    $absolute = (Resolve-Path -LiteralPath $DeckPath).Path
    $uri = [System.Uri]::new($absolute).AbsoluteUri
    $query = "slide=$([uri]::EscapeDataString($Slide))&revision=$([uri]::EscapeDataString($Revision))"
    if ($Section.Length -gt 0) {
        $query += "&section=$([uri]::EscapeDataString($Section))"
    }
    return "$uri`?$query"
}

function Initialize-RadiaObsScenes {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [string] $ProjectRoot,
        [Parameter(Mandatory)] [string] $Revision
    )

    foreach ($scene in $script:AllScenes) {
        Ensure-ObsScene -Session $Session -Name $scene
    }
    $kinds = Invoke-ObsRequest -Session $Session -RequestType 'GetInputKindList' -RequestData @{ unversioned = $false }
    $kindNames = [string[]]@($kinds.inputKinds)
    $windowKind = Resolve-ObsInputKind -AvailableKinds $kindNames -BaseKind 'window_capture'
    $browserKind = Resolve-ObsInputKind -AvailableKinds $kindNames -BaseKind 'browser_source'
    $microphoneKind = Resolve-ObsInputKind -AvailableKinds $kindNames -BaseKind 'wasapi_input_capture'
    $textKind = Resolve-ObsInputKind -AvailableKinds $kindNames -BaseKind 'text_gdiplus'

    # Automatic selected BitBlt on the validated workstation and captured the
    # desktop region behind the Vulkan window. Force Windows Graphics Capture.
    $windowSettings = New-RadiaWindowCaptureSettings
    $windowItem = Ensure-ObsInput -Session $Session -Scene 'Renderer' -Name $script:WindowInput -Kind $windowKind -Settings $windowSettings
    Set-ObsFullCanvasTransform -Session $Session -Scene 'Renderer' -SceneItemId $windowItem
    $bufferWindowItem = Ensure-ObsSceneItem -Session $Session -Scene 'Buffer Modes' -Source $script:WindowInput
    Set-ObsFullCanvasTransform -Session $Session -Scene 'Buffer Modes' -SceneItemId $bufferWindowItem

    $labelSettings = @{
        text = 'RADIA'
        color = 16777215
        opacity = 100
        outline = $true
        outline_color = 0
        outline_opacity = 85
        outline_size = 3
        font = @{ face = 'Segoe UI'; size = 36; flags = 1; style = 'Bold' }
    }
    $labelItem = Ensure-ObsInput -Session $Session -Scene 'Renderer' -Name $script:ModeLabelInput -Kind $textKind -Settings $labelSettings
    [void](Invoke-ObsRequest -Session $Session -RequestType 'SetSceneItemTransform' -RequestData @{
        sceneName = 'Renderer'; sceneItemId = $labelItem; sceneItemTransform = @{ positionX = 42.0; positionY = 38.0; alignment = 5 }
    })
    $bufferLabelItem = Ensure-ObsSceneItem -Session $Session -Scene 'Buffer Modes' -Source $script:ModeLabelInput
    [void](Invoke-ObsRequest -Session $Session -RequestType 'SetSceneItemTransform' -RequestData @{
        sceneName = 'Buffer Modes'; sceneItemId = $bufferLabelItem; sceneItemTransform = @{ positionX = 42.0; positionY = 38.0; alignment = 5 }
    })

    $deckPath = Join-Path -Path $ProjectRoot -ChildPath 'docs\hackathon\video-deck.html'
    foreach ($card in @(
        @('AEP Proof', 'Radia AEP Card', 'aep'),
        @('Math Proof', 'Radia Math Card', 'math'),
        @('Evidence', 'Radia Evidence Card', 'evidence'),
        @('Closing', 'Radia Closing Card', 'close')
    )) {
        $settings = @{
            url = New-DeckUri -DeckPath $deckPath -Slide $card[2] -Revision $Revision
            width = 1920
            height = 1080
            reroute_audio = $false
        }
        $item = Ensure-ObsInput -Session $Session -Scene $card[0] -Name $card[1] -Kind $browserKind -Settings $settings
        Set-ObsFullCanvasTransform -Session $Session -Scene $card[0] -SceneItemId $item
    }

    $teleprompterSettings = @{
        url = New-DeckUri -DeckPath $deckPath -Slide 'teleprompter' -Revision $Revision -Section 'thesis'
        width = 1920
        height = 1080
        reroute_audio = $false
    }
    $teleprompterItem = Ensure-ObsInput -Session $Session -Scene 'Narration' -Name $script:TeleprompterInput -Kind $browserKind -Settings $teleprompterSettings
    Set-ObsFullCanvasTransform -Session $Session -Scene 'Narration' -SceneItemId $teleprompterItem
    $microphoneDeviceId = 'default'
    $inputs = Get-ObsInputMap -Session $Session
    if ($inputs.ContainsKey($script:MicInput)) {
        $microphoneSettings = Invoke-ObsRequest -Session $Session -RequestType 'GetInputSettings' -RequestData @{ inputName = $script:MicInput }
        $configuredDeviceId = [string]$microphoneSettings.inputSettings.device_id
        if (-not [string]::IsNullOrWhiteSpace($configuredDeviceId)) {
            $microphoneDeviceId = $configuredDeviceId
        }
    }
    [void](Ensure-ObsInput -Session $Session -Scene 'Narration' -Name $script:MicInput -Kind $microphoneKind -Settings @{ device_id = $microphoneDeviceId })
}

function Save-ObsSourcePreview {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [string] $SourceName,
        [Parameter(Mandatory)] [string] $Path
    )

    $capture = Invoke-ObsRequest -Session $Session -RequestType 'GetSourceScreenshot' -RequestData @{
        sourceName = $SourceName
        imageFormat = 'png'
        imageWidth = 960
        imageHeight = 540
        imageCompressionQuality = -1
    }
    $imageData = [string]$capture.imageData
    if ($imageData -notmatch '^data:image/png;base64,(?<payload>.+)$') {
        throw "Save-ObsSourcePreview: source='$SourceName' returned invalid PNG data"
    }
    $bytes = [Convert]::FromBase64String($Matches['payload'])
    $signature = [Convert]::ToHexString($bytes[0..7])
    if ($signature -ne '89504E470D0A1A0A') {
        throw "Save-ObsSourcePreview: source='$SourceName' returned invalid PNG signature"
    }
    $directory = Split-Path -Parent $Path
    [System.IO.Directory]::CreateDirectory($directory) | Out-Null
    [System.IO.File]::WriteAllBytes($Path, $bytes)
    if ($bytes.Length -lt $script:MinimumPreviewBytes) {
        throw "Save-ObsSourcePreview: source='$SourceName' produced a likely blank preview bytes='$($bytes.Length)' path='$Path'"
    }
    return [pscustomobject]@{
        Source = $SourceName
        Path = $Path
        Bytes = $bytes.Length
        Sha256 = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    }
}

function Assert-RadiaObsVisualSources {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [string] $OutputDirectory
    )

    $collections = Invoke-ObsRequest -Session $Session -RequestType 'GetSceneCollectionList'
    if ([string]$collections.currentSceneCollectionName -ne $script:CollectionName) {
        throw "Assert-RadiaObsVisualSources: active collection is '$($collections.currentSceneCollectionName)', expected '$($script:CollectionName)'"
    }

    $forbiddenKinds = [System.Collections.Generic.HashSet[string]]::new(
        [string[]]@('display_capture', 'monitor_capture', 'screen_capture', 'duplicator_capture', 'game_capture'),
        [System.StringComparer]::OrdinalIgnoreCase
    )
    $inputs = Invoke-ObsRequest -Session $Session -RequestType 'GetInputList'
    foreach ($inputItem in @($inputs.inputs)) {
        $kind = if ($null -ne $inputItem.PSObject.Properties['unversionedInputKind']) {
            [string]$inputItem.unversionedInputKind
        }
        else {
            [string]$inputItem.inputKind
        }
        if ($forbiddenKinds.Contains($kind)) {
            throw "Assert-RadiaObsVisualSources: forbidden input kind='$kind' name='$($inputItem.inputName)'"
        }
    }

    $window = Invoke-ObsRequest -Session $Session -RequestType 'GetInputSettings' -RequestData @{
        inputName = $script:WindowInput
    }
    if ([string]$window.inputSettings.window -ne $script:WindowSelector) {
        throw "Assert-RadiaObsVisualSources: Radia window selector does not match the presentation contract"
    }
    if ([int]$window.inputSettings.method -ne $script:WindowsGraphicsCaptureMethod) {
        throw "Assert-RadiaObsVisualSources: Radia capture method is not Windows Graphics Capture"
    }

    $availableWindows = Invoke-ObsRequest -Session $Session -RequestType 'GetInputPropertiesListPropertyItems' -RequestData @{
        inputName = $script:WindowInput
        propertyName = 'window'
    }
    $match = @(
        $availableWindows.propertyItems |
            Where-Object { [string]$_.itemValue -eq $script:WindowSelector -and [bool]$_.itemEnabled }
    )
    if ($match.Count -ne 1) {
        throw 'Assert-RadiaObsVisualSources: launched Radia presentation window is not available to OBS'
    }

    $stamp = [datetime]::UtcNow.ToString('yyyyMMddTHHmmssZ')
    $previewDirectory = Join-Path -Path $OutputDirectory -ChildPath "preflight-$stamp"
    $targets = @(
        @('Renderer', $script:WindowInput, 'renderer.png'),
        @('AEP Proof', 'Radia AEP Card', 'aep.png'),
        @('Math Proof', 'Radia Math Card', 'math.png'),
        @('Evidence', 'Radia Evidence Card', 'evidence.png'),
        @('Closing', 'Radia Closing Card', 'closing.png')
    )
    $previews = [System.Collections.Generic.List[object]]::new()
    foreach ($target in $targets) {
        [void](Invoke-ObsRequest -Session $Session -RequestType 'SetCurrentProgramScene' -RequestData @{
            sceneName = $target[0]
        })
        Start-Sleep -Milliseconds 1000
        $path = Join-Path -Path $previewDirectory -ChildPath $target[2]
        $previews.Add((Save-ObsSourcePreview -Session $Session -SourceName $target[1] -Path $path))
    }
    [void](Invoke-ObsRequest -Session $Session -RequestType 'SetCurrentProgramScene' -RequestData @{
        sceneName = 'Renderer'
    })
    return [pscustomobject]@{
        PSTypeName = 'Radia.ObsVisualPreflight'
        Directory = $previewDirectory
        Previews = $previews
    }
}

function Set-ObsInputMuteIfPresent {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [AllowNull()] [string] $InputName,
        [Parameter(Mandatory)] [bool] $Muted
    )

    if ([string]::IsNullOrWhiteSpace($InputName)) {
        return
    }
    $inputs = Get-ObsInputMap -Session $Session
    if (-not $inputs.ContainsKey($InputName)) {
        return
    }
    [void](Invoke-ObsRequest -Session $Session -RequestType 'SetInputMute' -RequestData @{
        inputName = $InputName
        inputMuted = $Muted
    })
}

function Set-ObsFinalAudioState {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session
    )

    $special = Invoke-ObsRequest -Session $Session -RequestType 'GetSpecialInputs'
    foreach ($propertyName in 'desktop1', 'desktop2', 'mic1', 'mic2', 'mic3', 'mic4') {
        $property = $special.PSObject.Properties[$propertyName]
        if ($null -ne $property) {
            Set-ObsInputMuteIfPresent -Session $Session -InputName ([string]$property.Value) -Muted $true
        }
    }
    Set-ObsInputMuteIfPresent -Session $Session -InputName $script:MicInput -Muted $true
    Set-ObsInputMuteIfPresent -Session $Session -InputName $script:NarrationInput -Muted $false
}

function Set-ObsNarrationAudioState {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session
    )

    Set-ObsFinalAudioState -Session $Session
    Set-ObsInputMuteIfPresent -Session $Session -InputName $script:MicInput -Muted $false
}

function Ensure-ObsNarrationTrack {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [string] $NarrationPath
    )

    $settings = @{
        is_local_file = $true
        local_file = (Resolve-Path -LiteralPath $NarrationPath).Path
        looping = $false
        restart_on_activate = $false
        clear_on_media_end = $false
    }
    $inputs = Get-ObsInputMap -Session $Session
    if ($inputs.ContainsKey($script:NarrationInput)) {
        if ([string]$inputs[$script:NarrationInput] -ne 'ffmpeg_source') {
            throw "Ensure-ObsNarrationTrack: reserved input '$($script:NarrationInput)' has wrong kind"
        }
        [void](Invoke-ObsRequest -Session $Session -RequestType 'SetInputSettings' -RequestData @{
            inputName = $script:NarrationInput; inputSettings = $settings; overlay = $true
        })
    }
    else {
        [void](Ensure-ObsInput -Session $Session -Scene 'Renderer' -Name $script:NarrationInput -Kind 'ffmpeg_source' -Settings $settings)
    }
    foreach ($scene in $script:FinalScenes) {
        [void](Ensure-ObsSceneItem -Session $Session -Scene $scene -Source $script:NarrationInput)
    }
}

function Start-RadiaPresentation {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)] [string] $ProjectRoot
    )

    & cargo build --release -p radia-demo
    if ($LASTEXITCODE -ne 0) {
        throw "Start-RadiaPresentation: cargo build failed with exit code $LASTEXITCODE"
    }
    $executable = Join-Path -Path $ProjectRoot -ChildPath 'target\release\radia-demo.exe'
    if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
        throw "Start-RadiaPresentation: failed_input='executable' value='$executable' skipped='presentation'"
    }
    $start = [System.Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $executable
    $start.WorkingDirectory = $ProjectRoot
    $start.UseShellExecute = $false
    $start.RedirectStandardInput = $true
    $start.RedirectStandardOutput = $true
    $start.RedirectStandardError = $true
    $start.CreateNoWindow = $false
    [void]$start.ArgumentList.Add('present')
    [void]$start.ArgumentList.Add('--control-stdin')
    $process = [System.Diagnostics.Process]::new()
    $process.StartInfo = $start
    if (-not $process.Start()) {
        $process.Dispose()
        throw "Start-RadiaPresentation: process start returned false"
    }
    $stderrTask = $process.StandardError.ReadToEndAsync()
    $readyTask = $process.StandardOutput.ReadLineAsync()
    $ready = $readyTask.WaitAsync([TimeSpan]::FromSeconds(45)).GetAwaiter().GetResult()
    if ([string]::IsNullOrWhiteSpace($ready) -or $ready -notmatch '^RADIA_READY adapter=(.+) backend=Vulkan width=1280 height=720$') {
        if (-not $process.HasExited) {
            $process.Kill($true)
        }
        $process.Dispose()
        throw "Start-RadiaPresentation: invalid ready line '$ready'"
    }
    return [pscustomobject]@{
        PSTypeName = 'Radia.PresentationProcess'
        Process = $process
        StandardInput = $process.StandardInput
        StandardOutput = $process.StandardOutput
        StandardErrorTask = $stderrTask
        ReadyLine = $ready
        AdapterName = [string]$Matches[1]
    }
}

function Send-RadiaPresentationCommand {
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Presentation,
        [Parameter(Mandatory)] [string] $Command,
        [int] $TimeoutSeconds = 3
    )

    $Presentation.StandardInput.WriteLine($Command)
    $Presentation.StandardInput.Flush()
    $lineTask = $Presentation.StandardOutput.ReadLineAsync()
    $line = $lineTask.WaitAsync([TimeSpan]::FromSeconds($TimeoutSeconds)).GetAwaiter().GetResult()
    if ([string]::IsNullOrWhiteSpace($line)) {
        throw "Send-RadiaPresentationCommand: no acknowledgement for '$Command'"
    }
    if ($line -like 'RADIA_CONTROL_ERROR*') {
        throw "Send-RadiaPresentationCommand: command='$Command' response='$line'"
    }
    if ($line -notlike 'RADIA_ACK*') {
        throw "Send-RadiaPresentationCommand: unexpected response '$line'"
    }
    return $line
}

function Stop-RadiaPresentation {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Presentation
    )

    try {
        if (-not $Presentation.Process.HasExited) {
            [void](Send-RadiaPresentationCommand -Presentation $Presentation -Command 'quit')
            if (-not $Presentation.Process.WaitForExit(5000)) {
                $Presentation.Process.Kill($true)
            }
        }
    }
    catch {
        if (-not $Presentation.Process.HasExited) {
            $Presentation.Process.Kill($true)
        }
        Write-Warning "Stop-RadiaPresentation: forced spawned Radia process shutdown; $($_.Exception.Message)"
    }
    finally {
        $Presentation.Process.Dispose()
    }
}

function Wait-RadiaCue {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [System.Diagnostics.Stopwatch] $Stopwatch,
        [Parameter(Mandatory)] [int] $TargetMilliseconds,
        [Parameter(Mandatory)] [int] $HardStopMilliseconds
    )

    while ($Stopwatch.ElapsedMilliseconds -lt $TargetMilliseconds) {
        if ($Stopwatch.ElapsedMilliseconds -ge $HardStopMilliseconds) {
            throw "Wait-RadiaCue: hard stop reached at $($Stopwatch.ElapsedMilliseconds) ms"
        }
        $remaining = $TargetMilliseconds - [int]$Stopwatch.ElapsedMilliseconds
        Start-Sleep -Milliseconds ([Math]::Min(50, [Math]::Max(1, $remaining)))
    }
}

function Invoke-RadiaCue {
    [CmdletBinding()]
    [OutputType([void])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [pscustomobject] $Presentation,
        [Parameter(Mandatory)] [pscustomobject] $Cue
    )

    [void](Invoke-ObsRequest -Session $Session -RequestType 'SetCurrentProgramScene' -RequestData @{ sceneName = [string]$Cue.scene })
    [void](Invoke-ObsRequest -Session $Session -RequestType 'SetInputSettings' -RequestData @{
        inputName = $script:ModeLabelInput
        inputSettings = @{ text = [string]$Cue.label }
        overlay = $true
    })
    foreach ($command in @($Cue.radia_commands)) {
        [void](Send-RadiaPresentationCommand -Presentation $Presentation -Command ([string]$command))
    }
}

function Invoke-RadiaVisualTimeline {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [pscustomobject] $Presentation,
        [Parameter(Mandatory)] [pscustomobject] $Timeline,
        [Parameter(Mandatory)] [bool] $Record
    )

    $cues = @($Timeline.cues)
    Invoke-RadiaCue -Session $Session -Presentation $Presentation -Cue $cues[0]
    $recordingContext = $null
    if ($Record) {
        $recordingContext = Start-ObsRecordingSafely -Session $Session
    }
    [void](Invoke-ObsRequest -Session $Session -RequestType 'TriggerMediaInputAction' -RequestData @{
        inputName = $script:NarrationInput
        mediaAction = 'OBS_WEBSOCKET_MEDIA_INPUT_ACTION_RESTART'
    })
    $clock = [System.Diagnostics.Stopwatch]::StartNew()
    $recordingPath = $null
    try {
        foreach ($cue in $cues | Select-Object -Skip 1) {
            Wait-RadiaCue -Stopwatch $clock -TargetMilliseconds ([int]$cue.at_ms) -HardStopMilliseconds ([int]$Timeline.hard_stop_ms)
            Invoke-RadiaCue -Session $Session -Presentation $Presentation -Cue $cue
        }
        Wait-RadiaCue -Stopwatch $clock -TargetMilliseconds ([int]$Timeline.normal_stop_ms) -HardStopMilliseconds ([int]$Timeline.hard_stop_ms)
        if ($Record) {
            $stopped = Stop-ObsRecordingSafely -Session $Session -Context $recordingContext
            $recordingPath = [string]$stopped.OutputPath
        }
        $stats = Invoke-ObsRequest -Session $Session -RequestType 'GetStats'
        return [pscustomobject]@{
            PSTypeName = 'Radia.RecordingResult'
            Recorded = $Record
            ElapsedMilliseconds = [long]$clock.ElapsedMilliseconds
            RecordingPath = $recordingPath
            Stats = $stats
        }
    }
    catch {
        if ($Record) {
            $status = Invoke-ObsRequest -Session $Session -RequestType 'GetRecordStatus'
            if ([bool]$status.outputActive) {
                [void](Invoke-ObsRequest -Session $Session -RequestType 'StopRecord')
            }
        }
        throw
    }
    finally {
        $clock.Stop()
    }
}

function Invoke-RadiaNarrationCapture {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)] [pscustomobject] $Session,
        [Parameter(Mandatory)] [pscustomobject] $Timeline,
        [Parameter(Mandatory)] [string] $DeckPath,
        [Parameter(Mandatory)] [string] $Revision
    )

    [void](Invoke-ObsRequest -Session $Session -RequestType 'SetCurrentProgramScene' -RequestData @{ sceneName = 'Narration' })
    Set-ObsNarrationAudioState -Session $Session
    for ($seconds = 5; $seconds -ge 1; $seconds--) {
        Write-Progress -Activity 'Narration starts' -Status "$seconds" -PercentComplete ((5 - $seconds) * 20)
        Start-Sleep -Seconds 1
    }
    Write-Progress -Activity 'Narration starts' -Completed
    $recordingContext = Start-ObsRecordingSafely -Session $Session
    $clock = [System.Diagnostics.Stopwatch]::StartNew()
    $sections = @($Timeline.narration_sections)
    $nextSection = 0
    try {
        while ($clock.ElapsedMilliseconds -lt [int]$Timeline.hard_stop_ms) {
            if ($nextSection -lt $sections.Count -and $clock.ElapsedMilliseconds -ge [int]$sections[$nextSection].at_ms) {
                $sectionName = [string]$sections[$nextSection].section
                $url = New-DeckUri -DeckPath $DeckPath -Slide 'teleprompter' -Revision $Revision -Section $sectionName
                [void](Invoke-ObsRequest -Session $Session -RequestType 'SetInputSettings' -RequestData @{
                    inputName = $script:TeleprompterInput
                    inputSettings = @{ url = $url }
                    overlay = $true
                })
                $nextSection++
            }
            $percent = [Math]::Min(100, [int](100.0 * $clock.ElapsedMilliseconds / [int]$Timeline.hard_stop_ms))
            Write-Progress -Activity 'Recording narration' -Status 'Press Enter to stop early' -PercentComplete $percent
            if ([Console]::KeyAvailable -and [Console]::ReadKey($true).Key -eq [ConsoleKey]::Enter) {
                break
            }
            Start-Sleep -Milliseconds 50
        }
        $stopped = Stop-ObsRecordingSafely -Session $Session -Context $recordingContext
        return [pscustomobject]@{
            PSTypeName = 'Radia.NarrationResult'
            RecordingPath = [string]$stopped.OutputPath
            DurationMilliseconds = [long]$clock.ElapsedMilliseconds
        }
    }
    finally {
        $clock.Stop()
        Write-Progress -Activity 'Recording narration' -Completed
        Set-ObsInputMuteIfPresent -Session $Session -InputName $script:MicInput -Muted $true
    }
}

function Wait-ForRemuxedMp4 {
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Mandatory)] [string] $MkvPath,
        [int] $TimeoutSeconds = 45
    )

    $mp4 = [System.IO.Path]::ChangeExtension($MkvPath, '.mp4')
    $clock = [System.Diagnostics.Stopwatch]::StartNew()
    try {
        while (-not (Test-Path -LiteralPath $mp4 -PathType Leaf)) {
            if ($clock.Elapsed.TotalSeconds -ge $TimeoutSeconds) {
                throw "Wait-ForRemuxedMp4: timed out; recoverable MKV remains at '$MkvPath'"
            }
            Start-Sleep -Milliseconds 250
        }
        return (Resolve-Path -LiteralPath $mp4).Path
    }
    finally {
        $clock.Stop()
    }
}

function Write-RadiaRecordingManifest {
    [CmdletBinding()]
    [OutputType([string])]
    param(
        [Parameter(Mandatory)] [string] $OutputDirectory,
        [Parameter(Mandatory)] [string] $ProjectRoot,
        [Parameter(Mandatory)] [string] $TimelinePath,
        [Parameter(Mandatory)] [string] $NarrationPath,
        [Parameter(Mandatory)] [string] $MkvPath,
        [Parameter(Mandatory)] [string] $Mp4Path,
        [Parameter(Mandatory)] [long] $DurationMilliseconds,
        [Parameter(Mandatory)] [string] $AdapterName,
        [Parameter(Mandatory)] [string] $ObsWebSocketVersion,
        [Parameter(Mandatory)] [pscustomobject] $Stats
    )

    $manifest = [ordered]@{
        schema_version = 1
        purpose = 'presentation'
        authority = 'Existing raw GPU captures and manifests remain numeric authority.'
        created_utc = [DateTimeOffset]::UtcNow.ToString('O')
        git_commit = Get-GitCommit -ProjectRoot $ProjectRoot
        obs = [ordered]@{
            studio_version = '30.2.3'
            websocket_version = $ObsWebSocketVersion
            profile = $script:ProfileName
            collection = $script:CollectionName
            canvas = '1920x1080'
            fps = '30/1'
            color = 'Rec.709 SDR, NV12 partial'
            recording = 'NVENC H.264 MKV with automatic MP4 remux'
            audio = 'AAC stereo, 48 kHz, 192 kbps'
            render_total_frames = $Stats.renderTotalFrames
            render_skipped_frames = $Stats.renderSkippedFrames
            output_total_frames = $Stats.outputTotalFrames
            output_skipped_frames = $Stats.outputSkippedFrames
        }
        radia = [ordered]@{
            adapter = $AdapterName
            backend = 'Vulkan'
            command = 'target\release\radia-demo.exe present --control-stdin'
        }
        timeline = [ordered]@{
            path = 'docs/hackathon/video-timeline.json'
            sha256 = Get-FileSha256 -Path $TimelinePath
        }
        narration = [ordered]@{
            path = $NarrationPath
            sha256 = Get-FileSha256 -Path $NarrationPath
        }
        recording = [ordered]@{
            duration_ms = $DurationMilliseconds
            mkv_path = $MkvPath
            mkv_sha256 = Get-FileSha256 -Path $MkvPath
            mkv_bytes = (Get-Item -LiteralPath $MkvPath).Length
            mp4_path = $Mp4Path
            mp4_sha256 = Get-FileSha256 -Path $Mp4Path
            mp4_bytes = (Get-Item -LiteralPath $Mp4Path).Length
        }
        reproduction = "pwsh -NoProfile -File scripts/record-devpost-demo.ps1 -Action Record -NarrationPath <take>"
    }
    $stamp = [DateTimeOffset]::UtcNow.ToString('yyyyMMddTHHmmssZ')
    $manifestDirectory = Join-Path -Path $OutputDirectory -ChildPath $stamp
    [void](New-Item -ItemType Directory -Path $manifestDirectory -Force)
    $manifestPath = Join-Path -Path $manifestDirectory -ChildPath 'recording-manifest.json'
    $manifest | ConvertTo-Json -Depth 16 | Set-Content -LiteralPath $manifestPath -Encoding utf8NoBOM
    return $manifestPath
}

function Test-RadiaLocalInputs {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)] [string] $ProjectRoot,
        [Parameter(Mandatory)] [string] $TimelinePath,
        [AllowEmptyString()] [string] $NarrationPath
    )

    $timeline = Get-RadiaTimeline -Path $TimelinePath
    $deckPath = Join-Path -Path $ProjectRoot -ChildPath 'docs\hackathon\video-deck.html'
    if (-not (Test-Path -LiteralPath $deckPath -PathType Leaf)) {
        throw "Test-RadiaLocalInputs: failed_input='deck' value='$deckPath' skipped='dry run'"
    }
    $duration = $null
    if (-not [string]::IsNullOrWhiteSpace($NarrationPath)) {
        if (-not (Test-Path -LiteralPath $NarrationPath -PathType Leaf)) {
            throw "Test-RadiaLocalInputs: failed_input='NarrationPath' value='$NarrationPath' skipped='dry run'"
        }
        $duration = Get-MediaDurationMilliseconds -Path $NarrationPath
        if ($duration -gt [long]$timeline.hard_stop_ms) {
            throw "Test-RadiaLocalInputs: narration duration '$duration' exceeds hard stop '$($timeline.hard_stop_ms)'"
        }
    }
    return [pscustomobject]@{
        PSTypeName = 'Radia.DryRunResult'
        TimelinePath = (Resolve-Path -LiteralPath $TimelinePath).Path
        TimelineSha256 = Get-FileSha256 -Path $TimelinePath
        DeckPath = (Resolve-Path -LiteralPath $deckPath).Path
        NarrationPath = $NarrationPath
        NarrationDurationMilliseconds = $duration
        NormalStopMilliseconds = [int]$timeline.normal_stop_ms
        HardStopMilliseconds = [int]$timeline.hard_stop_ms
        ObsStarted = $false
    }
}

function Invoke-RadiaDemoVideo {
    [CmdletBinding()]
    [OutputType([pscustomobject])]
    param(
        [Parameter(Mandatory)]
        [ValidateSet('SaveCredential', 'Setup', 'Narration', 'DryRun', 'Rehearse', 'Record', 'Validate')]
        [string] $Action,
        [string] $NarrationPath = '',
        [string] $VideoPath = '',
        [string] $OutputDirectory = '',
        [uri] $ObsUri = 'ws://127.0.0.1:4455',
        [pscredential] $ObsCredential
    )

    if (-not $IsWindows) {
        throw "Invoke-RadiaDemoVideo: failed_input='platform' skipped='OBS automation'"
    }
    $projectRoot = Get-RadiaProjectRoot
    if ($Action -eq 'SaveCredential') {
        return Save-ObsCredential -ProjectRoot $projectRoot
    }
    $timelinePath = Join-Path -Path $projectRoot -ChildPath 'docs\hackathon\video-timeline.json'
    $deckPath = Join-Path -Path $projectRoot -ChildPath 'docs\hackathon\video-deck.html'
    if ([string]::IsNullOrWhiteSpace($OutputDirectory)) {
        $OutputDirectory = Join-Path -Path $projectRoot -ChildPath 'Temp\build-week-video'
    }
    [void](New-Item -ItemType Directory -Path $OutputDirectory -Force)
    $timeline = Get-RadiaTimeline -Path $timelinePath

    if ($Action -eq 'DryRun') {
        return Test-RadiaLocalInputs -ProjectRoot $projectRoot -TimelinePath $timelinePath -NarrationPath $NarrationPath
    }
    if ($Action -eq 'Validate') {
        if ([string]::IsNullOrWhiteSpace($VideoPath)) {
            $latest = Get-ChildItem -LiteralPath $OutputDirectory -Filter '*.mp4' -File | Sort-Object LastWriteTimeUtc -Descending | Select-Object -First 1
            if ($null -eq $latest) {
                throw "Invoke-RadiaDemoVideo: failed_input='VideoPath' skipped='validation'"
            }
            $VideoPath = $latest.FullName
        }
        $duration = Get-MediaDurationMilliseconds -Path $VideoPath
        if ($duration -lt 150000 -or $duration -gt [long]$timeline.hard_stop_ms) {
            throw "Invoke-RadiaDemoVideo: failed_input='video duration' value='$duration' skipped='validation'"
        }
        return [pscustomobject]@{
            PSTypeName = 'Radia.VideoValidation'
            VideoPath = (Resolve-Path -LiteralPath $VideoPath).Path
            Sha256 = Get-FileSha256 -Path $VideoPath
            DurationMilliseconds = $duration
            DurationValid = $true
            TimelineValid = $true
        }
    }

    if ($Action -in @('Rehearse', 'Record') -and [string]::IsNullOrWhiteSpace($NarrationPath)) {
        throw "Invoke-RadiaDemoVideo: failed_input='NarrationPath' skipped='$Action'"
    }
    if ($Action -in @('Rehearse', 'Record')) {
        [void](Test-RadiaLocalInputs -ProjectRoot $projectRoot -TimelinePath $timelinePath -NarrationPath $NarrationPath)
    }

    $credential = Get-ObsCredential -Credential $ObsCredential -ProjectRoot $projectRoot
    $sessionOutput = @(Connect-ObsWebSocket -Uri $ObsUri -Credential $credential)
    $sessionCandidates = @(
        $sessionOutput | Where-Object { $null -ne $_.PSObject.Properties['Socket'] }
    )
    if ($sessionCandidates.Count -ne 1) {
        $types = @($sessionOutput | ForEach-Object { $_.GetType().FullName }) -join ', '
        throw "Invoke-RadiaDemoVideo: OBS connection returned invalid session output types='$types'"
    }
    $session = $sessionCandidates[0]
    $context = $null
    $presentation = $null
    try {
        $context = Enter-RadiaObsContext -Session $session
        Set-RadiaObsProfile -Session $session -OutputDirectory $OutputDirectory
        $revision = Get-GitCommit -ProjectRoot $projectRoot
        if ($Action -eq 'Setup') {
            $presentation = Start-RadiaPresentation -ProjectRoot $projectRoot
            Initialize-RadiaObsScenes -Session $session -ProjectRoot $projectRoot -Revision $revision
            $preflight = Assert-RadiaObsVisualSources -Session $session -OutputDirectory $OutputDirectory
            return [pscustomobject]@{
                PSTypeName = 'Radia.ObsSetupResult'
                Profile = $script:ProfileName
                Collection = $script:CollectionName
                Canvas = '1920x1080'
                Fps = 30
                WindowReady = $true
                Adapter = $presentation.AdapterName
                PreviewDirectory = $preflight.Directory
            }
        }
        if ($Action -eq 'Narration') {
            Initialize-RadiaObsScenes -Session $session -ProjectRoot $projectRoot -Revision $revision
            return Invoke-RadiaNarrationCapture -Session $session -Timeline $timeline -DeckPath $deckPath -Revision $revision
        }

        $presentation = Start-RadiaPresentation -ProjectRoot $projectRoot
        Initialize-RadiaObsScenes -Session $session -ProjectRoot $projectRoot -Revision $revision
        Ensure-ObsNarrationTrack -Session $session -NarrationPath $NarrationPath
        Set-ObsFinalAudioState -Session $session
        $preflight = Assert-RadiaObsVisualSources -Session $session -OutputDirectory $OutputDirectory
        $result = Invoke-RadiaVisualTimeline -Session $session -Presentation $presentation -Timeline $timeline -Record ($Action -eq 'Record')
        $result | Add-Member -NotePropertyName PreviewDirectory -NotePropertyValue $preflight.Directory
        if ($Action -eq 'Rehearse') {
            return $result
        }
        $mp4Path = Wait-ForRemuxedMp4 -MkvPath $result.RecordingPath
        $manifestPath = Write-RadiaRecordingManifest `
            -OutputDirectory $OutputDirectory `
            -ProjectRoot $projectRoot `
            -TimelinePath $timelinePath `
            -NarrationPath $NarrationPath `
            -MkvPath $result.RecordingPath `
            -Mp4Path $mp4Path `
            -DurationMilliseconds $result.ElapsedMilliseconds `
            -AdapterName $presentation.AdapterName `
            -ObsWebSocketVersion $session.WebSocketVersion `
            -Stats $result.Stats
        return [pscustomobject]@{
            PSTypeName = 'Radia.VideoRecording'
            MkvPath = $result.RecordingPath
            Mp4Path = $mp4Path
            ManifestPath = $manifestPath
            DurationMilliseconds = $result.ElapsedMilliseconds
        }
    }
    finally {
        if ($null -ne $presentation) {
            Stop-RadiaPresentation -Presentation $presentation
        }
        if ($null -ne $context) {
            try {
                Exit-RadiaObsContext -Session $session -Context $context
            }
            catch {
                Write-Warning "Invoke-RadiaDemoVideo: OBS context restore failed; $($_.Exception.Message)"
            }
        }
        if ($null -ne $session -and $null -ne $session.PSObject.Properties['Socket']) {
            Disconnect-ObsWebSocket -Session $session
        }
    }
}

Export-ModuleMember -Function Invoke-RadiaDemoVideo, Test-RadiaVideoTimeline
