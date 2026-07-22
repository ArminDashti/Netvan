# Extracting Dataset SQL from .rdl Files

The `.rdl` file is XML. Datasets live under the `DataSets` element, each with a
`Query` containing the `CommandText` — that's the actual SQL SSRS executes.
There are two shapes to handle: embedded datasets (SQL is right there in the
RDL) and shared datasets (RDL only references an external `.rsd`).

RDL files for this project are at: `C:/Users/a.dashti/TFS/RDL`

## RDL namespace

SSRS 2017 (.rdl format version 2016, used by Report Builder/SSDT for SQL Server
2017) typically uses this namespace — check the actual file, as older RDL
versions differ:

```
http://schemas.microsoft.com/sqlserver/reporting/2016/01/reportdefinition
```

## Embedded dataset — extracting CommandText

```vb
Imports System.Xml
Imports System.Xml.Linq

Public Function ExtractEmbeddedQueries(rdlPath As String) As Dictionary(Of String, String)
    Dim result As New Dictionary(Of String, String)
    Dim doc As XDocument = XDocument.Load(rdlPath)

    ' Namespace varies by RDL version - detect from the root element
    Dim ns As XNamespace = doc.Root.Name.Namespace

    Dim dataSets = doc.Descendants(ns + "DataSet")
    For Each ds In dataSets
        Dim dsName As String = ds.Attribute("Name")?.Value
        Dim queryEl = ds.Element(ns + "Query")
        If queryEl Is Nothing Then Continue For

        Dim commandText As String = queryEl.Element(ns + "CommandText")?.Value

        ' Check if this is a reference to a shared dataset instead
        Dim sharedDsRef = queryEl.Element(ns + "DataSourceName")
        Dim sharedDataSetRef = ds.Element(ns + "SharedDataSet")

        If sharedDataSetRef IsNot Nothing Then
            Dim sharedRef As String = sharedDataSetRef.Element(ns + "SharedDataSetReference")?.Value
            result(dsName) = $"[SHARED DATASET REFERENCE: {sharedRef} - resolve via .rsd, see below]"
        ElseIf commandText IsNot Nothing Then
            result(dsName) = commandText
        End If
    Next

    Return result
End Function
```

## Shared datasets (.rsd files)

If a `<DataSet>` element contains a `<SharedDataSet>` child instead of an
inline `<Query>`, the actual SQL lives in a separate `.rsd` file (also XML, same
basic `Query`/`CommandText` shape) deployed to the Report Server, or in the
project's shared-dataset folder if you have source access.

```vb
Public Function ExtractSharedDatasetQuery(rsdPath As String) As String
    Dim doc As XDocument = XDocument.Load(rsdPath)
    Dim ns As XNamespace = doc.Root.Name.Namespace
    Return doc.Descendants(ns + "CommandText").FirstOrDefault()?.Value
End Function
```

If you don't have filesystem access to the `.rsd` (e.g. it only exists on the
Report Server catalog), it can be retrieved via the `ReportService2010` SOAP
endpoint (`GetItemDefinition` on the shared dataset's catalog path) — this is a
different service from `ReportExecutionService` used for rendering.

## Query parameters vs. report parameters

Watch for `<QueryParameters>` inside the dataset — these map report parameters
to `@ParamName` placeholders in the SQL. When you run the extracted query
independently, you need to supply the same values as `SqlParameter`s:

```vb
Dim queryParams = ds.Element(ns + "Query").Element(ns + "QueryParameters")?.Elements(ns + "QueryParameter")
For Each qp In queryParams
    Dim paramName As String = qp.Attribute("Name")?.Value
    Dim valueExpr As String = qp.Element(ns + "Value")?.Value
    ' valueExpr is often "=Parameters!StartDate.Value" - map this back to your
    ' report parameter values when building the independent SQL execution.
Next
```

## Running the extracted query independently

```vb
Imports System.Data.SqlClient

Public Function RunExtractedQuery(commandText As String, params As Dictionary(Of String, Object)) As DataTable
    Dim connectionString As String = "Server=10.10.12.52;Database=Pakhsh_Data_New;User Id=public01;Password=Public01Public01;"
    Using conn As New SqlConnection(connectionString)
        Using cmd As New SqlCommand(commandText, conn)
            For Each kvp In params
                cmd.Parameters.AddWithValue("@" & kvp.Key, kvp.Value)
            Next
            conn.Open()
            Dim dt As New DataTable()
            dt.Load(cmd.ExecuteReader())
            Return dt
        End Using
    End Using
End Function
```

## Things that will trip this up

- **Expression-based CommandText**: rare, but `CommandText` can itself be an
  expression rather than a literal string. If so, this approach can't extract
  it mechanically — flag it to the user rather than guessing.
- **Multiple data sources**: a report can pull from several datasets against
  different connections — match each dataset to its `DataSourceName` and the
  corresponding connection string, don't assume one connection for the whole
  report.
- **Stored procedures**: `CommandText` may just be a proc name with
  `CommandType=StoredProcedure` — same extraction applies, just execute as a
  stored procedure call rather than inline SQL.
