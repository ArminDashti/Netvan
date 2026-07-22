# Rendering Methods (SSRS 14 / VB.NET)

Two ways to get rendered output out of SSRS for comparison. Default to URL
Access; use SOAP only when you specifically need session control, history
snapshots, or stream-based rendering.

Report server base URL for testing: `http://localhost:3020`

## URL Access (default)

A plain HTTP GET against the report server. No SOAP client needed.

```
http://<server>/ReportServer/Pages/ReportViewer.aspx?<ReportPath>&rs:Command=Render&rs:Format=<Format>&<ParamName>=<ParamValue>
```

Notes:
- Report path is URL-encoded (`%2f` for `/`).
- `rs:Format=CSV` or `rs:Format=XML` for data validation. Avoid `PDF` /
  `EXCELOPENXML` / `WORD` unless the task is layout testing specifically.
- Report parameters are appended as ordinary query-string key=value pairs.
- Requires Windows auth (NTLM/Kerberos) against the report server.

### VB.NET example

```vb
Imports System.Net
Imports System.IO

Public Function GetReportCsv(reportPath As String, parameters As Dictionary(Of String, String)) As String
    Dim baseUrl As String = "http://localhost:3020/ReportServer"
    Dim encodedPath As String = Uri.EscapeDataString(reportPath)

    Dim paramString As String = ""
    For Each kvp In parameters
        paramString &= $"&{kvp.Key}={Uri.EscapeDataString(kvp.Value)}"
    Next

    Dim url As String = $"{baseUrl}?{encodedPath}&rs:Command=Render&rs:Format=CSV{paramString}"

    Dim request As HttpWebRequest = CType(WebRequest.Create(url), HttpWebRequest)
    request.UseDefaultCredentials = True ' NTLM via current Windows identity
    request.Credentials = CredentialCache.DefaultCredentials

    Using response As HttpWebResponse = CType(request.GetResponse(), HttpWebResponse)
        Using reader As New StreamReader(response.GetResponseStream())
            Return reader.ReadToEnd()
        End Using
    End Using
End Function
```

If running this from a non-domain context (e.g. a scheduled task under a
different account), supply explicit credentials instead of
`DefaultCredentials`:

```vb
request.Credentials = New NetworkCredential("username", "password", "DOMAIN")
```

### curl equivalent (for quick manual checks or shelling out)

```bash
curl --ntlm -u "DOMAIN\user:password" \
  "http://localhost:3020/ReportServer?%2fYourFolder%2fYourReport&rs:Command=Render&rs:Format=CSV&StartDate=2026-01-01&EndDate=2026-01-31"
```

## SOAP (ReportExecutionService) — use when URL Access isn't enough

Reach for this when you need: report history/snapshot execution, render output
as a stream without writing to disk first, or explicit multi-step execution
session control (load → set params → render as separate calls, e.g. to inspect
parameter info before rendering).

Service endpoint for SSRS 14: `http://localhost:3020/ReportServer/ReportExecution2005.asmx`

```vb
Imports YourProject.ReportExecutionServiceReference ' add as service reference

Public Function RenderReportSoap(reportPath As String, parameters As Dictionary(Of String, String)) As Byte()
    Dim rs As New ReportExecutionService()
    rs.Credentials = System.Net.CredentialCache.DefaultCredentials
    rs.Url = "http://localhost:3020/ReportServer/ReportExecution2005.asmx"

    Dim execInfo As ExecutionInfo = rs.LoadReport(reportPath, Nothing)

    Dim paramValues(parameters.Count - 1) As ParameterValue
    Dim i As Integer = 0
    For Each kvp In parameters
        paramValues(i) = New ParameterValue()
        paramValues(i).Name = kvp.Key
        paramValues(i).Value = kvp.Value
        i += 1
    Next
    rs.SetExecutionParameters(paramValues, "en-us")

    Dim extension As String = Nothing
    Dim mimeType As String = Nothing
    Dim encoding As String = Nothing
    Dim warnings As Warning() = Nothing
    Dim streamIds As String() = Nothing

    Dim result As Byte() = rs.Render("CSV", Nothing, extension, mimeType, encoding, warnings, streamIds)
    Return result
End Function
```

To render a report execution **history snapshot** instead of live data (useful
when checking "does the snapshot match what was true at snapshot time"), use
`LoadReportHistory` with a history ID in place of `LoadReport`.

## Choosing parameters for a sweep

For parameter-matrix testing (checking multiple parameter combinations, e.g.
catching cascading-parameter dead ends), URL Access is much cheaper to loop —
each combination is just a new GET. SOAP's `LoadReport`/`SetExecutionParameters`/
`Render` triple is more overhead per combination unless you're reusing the same
execution session deliberately.
