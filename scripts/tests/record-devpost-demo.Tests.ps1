#Requires -Version 7.4

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$projectRoot = [System.IO.Path]::GetFullPath((Join-Path -Path $PSScriptRoot -ChildPath '..\..'))
$modulePath = Join-Path -Path $projectRoot -ChildPath 'scripts\record-devpost-demo.psm1'
Import-Module -Name $modulePath -Force

Describe 'Radia Build Week video contract' {
    It 'accepts the committed timeline' {
        $path = Join-Path -Path $projectRoot -ChildPath 'docs\hackathon\video-timeline.json'
        $timeline = Get-Content -Raw -LiteralPath $path | ConvertFrom-Json -Depth 32
        { $timeline | Test-RadiaVideoTimeline } | Should Not Throw
    }

    It 'rejects duplicate cue timestamps' {
        $path = Join-Path -Path $projectRoot -ChildPath 'docs\hackathon\video-timeline.json'
        $timeline = Get-Content -Raw -LiteralPath $path | ConvertFrom-Json -Depth 32
        $timeline.cues[1].at_ms = $timeline.cues[0].at_ms
        $thrown = $false
        try { Test-RadiaVideoTimeline -Timeline $timeline } catch { $thrown = $true }
        $thrown | Should Be $true
    }

    It 'rejects timelines at or beyond three minutes' {
        $path = Join-Path -Path $projectRoot -ChildPath 'docs\hackathon\video-timeline.json'
        $timeline = Get-Content -Raw -LiteralPath $path | ConvertFrom-Json -Depth 32
        $timeline.hard_stop_ms = 179000
        $thrown = $false
        try { Test-RadiaVideoTimeline -Timeline $timeline } catch { $thrown = $true }
        $thrown | Should Be $true
    }

    It 'keeps the public deck free of private paths and credentials' {
        $path = Join-Path -Path $projectRoot -ChildPath 'docs\hackathon\video-deck.html'
        $deck = Get-Content -Raw -LiteralPath $path
        $deck | Should Not Match 'C:\\Legaia'
        $deck | Should Not Match 'C:\\Users'
        $deck | Should Not Match 'server_password'
        $deck | Should Not Match 'Display Capture'
    }

    It 'keeps the script, manifest, and selected voice contract aligned' {
        $manifestPath = Join-Path -Path $projectRoot -ChildPath 'docs\hackathon\tts-narration.json'
        $scriptPath = Join-Path -Path $projectRoot -ChildPath 'docs\hackathon\demo-script.md'
        $selectionPath = Join-Path -Path $projectRoot -ChildPath 'docs\hackathon\voice-selection.json'
        $manifest = Get-Content -Raw -LiteralPath $manifestPath | ConvertFrom-Json -Depth 32
        $script = Get-Content -Raw -LiteralPath $scriptPath
        $selection = Get-Content -Raw -LiteralPath $selectionPath | ConvertFrom-Json -Depth 32
        $draft = [regex]::Match(
            $script,
            '(?ms)^## Full read-aloud draft\s+(?<body>.*?)^## 0:00-0:15 - Thesis'
        ).Groups['body'].Value
        $quoted = [regex]::Matches(
            $draft,
            '(?ms)^### [^\r\n]+\r?\n\r?\n"(?<text>.*?)"\r?$'
        )
        $quoted.Count | Should Be $manifest.sections.Count
        $manifest.sections.Count | Should Be $selection.script.sections

        $totalWords = 0
        for ($index = 0; $index -lt $manifest.sections.Count; $index++) {
            $draftText = ([regex]::Replace($quoted[$index].Groups['text'].Value, '\s+', ' ')).Trim()
            $manifestText = ([regex]::Replace($manifest.sections[$index].text, '\s+', ' ')).Trim()
            $draftText | Should Be $manifestText
            $wordCount = [regex]::Matches(
                $manifestText,
                "\b[\p{L}\p{N}]+(?:[-']?[\p{L}\p{N}]+)*\b"
            ).Count
            $totalWords += $wordCount
            $slotMs = $manifest.sections[$index].endMs - $manifest.sections[$index].startMs
            ([int]$selection.script.approvedSectionDurationsMs[$index] -le $slotMs) |
                Should Be $true
        }

        $totalWords | Should Be $selection.script.words
        $manifest.durationMs | Should Be $selection.script.timelineDurationMs
        $manifest.engine.seed | Should Be $selection.selection.settings.seed
        $manifest.engine.exaggeration | Should Be $selection.selection.settings.exaggeration
        $manifest.engine.cfgWeight | Should Be $selection.selection.settings.cfgWeight
        $manifest.engine.temperature | Should Be $selection.selection.settings.temperature
        $selection.governance.fullNarrationApproved | Should Be $true
        $selection.governance.obsRecordingApproved | Should Be $true
    }
}

InModuleScope record-devpost-demo {
    Describe 'OBS input kind resolution' {
        It 'prefers exact supported kinds' {
            Resolve-ObsInputKind -AvailableKinds @('window_capture', 'window_capture_v2') -BaseKind 'window_capture' |
                Should Be 'window_capture'
        }

        It 'selects the newest versioned kind when OBS omits the base kind' {
            Resolve-ObsInputKind -AvailableKinds @('text_gdiplus_v1', 'text_gdiplus_v2') -BaseKind 'text_gdiplus' |
                Should Be 'text_gdiplus_v2'
        }
    }

    Describe 'OBS visual capture contract' {
        It 'forces Windows Graphics Capture for the Vulkan renderer' {
            $settings = New-RadiaWindowCaptureSettings
            $settings.window | Should Be 'Radia - Jade Dragon Triad:Window Class:radia-demo.exe'
            $settings.method | Should Be 2
            $settings.cursor | Should Be $false
            $settings.client_area | Should Be $true
        }

        It 'refuses an automatic or BitBlt renderer capture before screenshots' {
            Mock Invoke-ObsRequest { [pscustomobject]@{ currentSceneCollectionName = 'Radia Build Week' } } `
                -ParameterFilter { $RequestType -eq 'GetSceneCollectionList' }
            Mock Invoke-ObsRequest { [pscustomobject]@{ inputs = @() } } `
                -ParameterFilter { $RequestType -eq 'GetInputList' }
            Mock Invoke-ObsRequest {
                [pscustomobject]@{
                    inputSettings = [pscustomobject]@{
                        window = 'Radia - Jade Dragon Triad:Window Class:radia-demo.exe'
                        method = 0
                    }
                }
            } -ParameterFilter { $RequestType -eq 'GetInputSettings' }

            $message = $null
            try {
                Assert-RadiaObsVisualSources -Session ([pscustomobject]@{}) -OutputDirectory $TestDrive
            }
            catch {
                $message = $_.Exception.Message
            }
            $message | Should Match 'capture method is not Windows Graphics Capture'
            Assert-MockCalled Invoke-ObsRequest -Times 0 -Exactly `
                -ParameterFilter { $RequestType -eq 'GetSourceScreenshot' }
        }

        It 'refuses a display capture input in the dedicated collection' {
            Mock Invoke-ObsRequest { [pscustomobject]@{ currentSceneCollectionName = 'Radia Build Week' } } `
                -ParameterFilter { $RequestType -eq 'GetSceneCollectionList' }
            Mock Invoke-ObsRequest {
                [pscustomobject]@{
                    inputs = @(
                        [pscustomobject]@{
                            inputName = 'Desktop'
                            inputKind = 'monitor_capture'
                            unversionedInputKind = 'monitor_capture'
                        }
                    )
                }
            } -ParameterFilter { $RequestType -eq 'GetInputList' }

            $message = $null
            try {
                Assert-RadiaObsVisualSources -Session ([pscustomobject]@{}) -OutputDirectory $TestDrive
            }
            catch {
                $message = $_.Exception.Message
            }
            $message | Should Match "forbidden input kind='monitor_capture'"
        }
    }

    Describe 'OBS recording stop recovery' {
        It 'returns the path from a normal StopRecord response' {
            Mock Invoke-ObsRequest { [pscustomobject]@{ outputPath = 'C:\recordings\take.mkv' } } `
                -ParameterFilter { $RequestType -eq 'StopRecord' }
            $context = [pscustomobject]@{
                RecordDirectory = $TestDrive
                ExistingPaths = @()
                StartedAtUtc = [datetime]::UtcNow
            }
            $result = Stop-ObsRecordingSafely -Session ([pscustomobject]@{}) -Context $context
            $result.OutputPath | Should Be 'C:\recordings\take.mkv'
            $result.Recovered | Should Be $false
        }

        It 'recovers code 501 only from one new MKV after OBS is inactive' {
            $failure = [System.InvalidOperationException]::new('StopRecord code 501')
            $failure.Data['ObsStatusCode'] = 501
            Mock Invoke-ObsRequest { throw $failure } `
                -ParameterFilter { $RequestType -eq 'StopRecord' }
            Mock Invoke-ObsRequest { [pscustomobject]@{ outputActive = $false } } `
                -ParameterFilter { $RequestType -eq 'GetRecordStatus' }
            $path = Join-Path $TestDrive 'take.mkv'
            [System.IO.File]::WriteAllBytes($path, [byte[]](1, 2, 3))
            $context = [pscustomobject]@{
                RecordDirectory = $TestDrive
                ExistingPaths = @()
                StartedAtUtc = [datetime]::UtcNow.AddSeconds(-1)
            }
            $result = Stop-ObsRecordingSafely -Session ([pscustomobject]@{}) -Context $context
            $result.OutputPath | Should Be $path
            $result.Recovered | Should Be $true
        }

        It 'does not mask code 501 while OBS still reports active' {
            $failure = [System.InvalidOperationException]::new('StopRecord code 501')
            $failure.Data['ObsStatusCode'] = 501
            Mock Invoke-ObsRequest { throw $failure } `
                -ParameterFilter { $RequestType -eq 'StopRecord' }
            Mock Invoke-ObsRequest { [pscustomobject]@{ outputActive = $true } } `
                -ParameterFilter { $RequestType -eq 'GetRecordStatus' }
            $context = [pscustomobject]@{
                RecordDirectory = $TestDrive
                ExistingPaths = @()
                StartedAtUtc = [datetime]::UtcNow
            }
            $thrown = $false
            try {
                [void](Stop-ObsRecordingSafely -Session ([pscustomobject]@{}) -Context $context)
            }
            catch {
                $thrown = $true
            }
            $thrown | Should Be $true
        }
    }
}
