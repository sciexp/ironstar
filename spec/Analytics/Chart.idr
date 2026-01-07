||| Chart value objects for visualization specifications
|||
||| This module defines value objects for chart configurations,
||| supporting both ECharts and Vega-Lite visualization types.
|||
||| Note: This is a simplified specification for type system purposes.
||| Production implementations would include full ECharts/Vega-Lite schema types.
|||
||| Reference: docs/notes/architecture/core/bounded-contexts.md (Analytics section)
module Analytics.Chart

import Data.List

%default total

------------------------------------------------------------------------
-- Chart types
------------------------------------------------------------------------

||| Supported visualization types
||| Maps to ECharts and Vega-Lite chart types
public export
data ChartType
  = Line
  | Bar
  | Scatter
  | Pie
  | Heatmap
  | Area
  | Histogram
  | Boxplot

public export
Eq ChartType where
  Line == Line = True
  Bar == Bar = True
  Scatter == Scatter = True
  Pie == Pie = True
  Heatmap == Heatmap = True
  Area == Area = True
  Histogram == Histogram = True
  Boxplot == Boxplot = True
  _ == _ = False

------------------------------------------------------------------------
-- Chart options
------------------------------------------------------------------------

||| Axis configuration
public export
record AxisConfig where
  constructor MkAxisConfig
  label : String
  min : Maybe Double
  max : Maybe Double
  scale : String  -- "linear" | "log" | "time"

||| Series data point
||| Simplified representation; real implementation would support
||| different data shapes (1D, 2D, time-series, etc.)
public export
record DataPoint where
  constructor MkDataPoint
  x : Double
  y : Double
  label : Maybe String

||| Data series for visualization
public export
record Series where
  constructor MkSeries
  name : String
  dataPoints : List DataPoint
  seriesType : ChartType

||| Axis data (for categorical or time axes)
public export
record AxisData where
  constructor MkAxisData
  categories : List String

------------------------------------------------------------------------
-- Chart data
------------------------------------------------------------------------

||| Complete chart data including series and axes
public export
record ChartData where
  constructor MkChartData
  series : List Series
  xAxis : Maybe AxisData
  yAxis : Maybe AxisData

------------------------------------------------------------------------
-- Chart configuration
------------------------------------------------------------------------

||| Color palette for chart styling
public export
record ColorPalette where
  constructor MkColorPalette
  colors : List String  -- Hex color codes

||| Legend configuration
public export
record LegendConfig where
  constructor MkLegendConfig
  show : Bool
  position : String  -- "top" | "bottom" | "left" | "right"

||| Grid configuration (for positioning)
public export
record GridConfig where
  constructor MkGridConfig
  top : Nat
  bottom : Nat
  left : Nat
  right : Nat

||| Chart display options
||| This is a simplified schema; production would use ECharts/Vega-Lite types
public export
record ChartOptions where
  constructor MkChartOptions
  title : Maybe String
  xAxis : Maybe AxisConfig
  yAxis : Maybe AxisConfig
  legend : LegendConfig
  grid : GridConfig
  palette : ColorPalette
  interactive : Bool  -- Enable zoom, pan, tooltips

------------------------------------------------------------------------
-- Complete chart configuration
------------------------------------------------------------------------

||| Complete chart specification combining type and options
||| This is what Analytics bounded context produces and Workspace consumes
public export
record ChartConfig where
  constructor MkChartConfig
  chartType : ChartType
  options : ChartOptions

------------------------------------------------------------------------
-- Default configurations
------------------------------------------------------------------------

||| Default axis configuration
public export
defaultAxisConfig : AxisConfig
defaultAxisConfig = MkAxisConfig
  { label = ""
  , min = Nothing
  , max = Nothing
  , scale = "linear"
  }

||| Default legend configuration
public export
defaultLegend : LegendConfig
defaultLegend = MkLegendConfig
  { show = True
  , position = "top"
  }

||| Default grid configuration (20px padding on all sides)
public export
defaultGrid : GridConfig
defaultGrid = MkGridConfig
  { top = 20
  , bottom = 20
  , left = 20
  , right = 20
  }

||| Default color palette (Open Props colors)
public export
defaultPalette : ColorPalette
defaultPalette = MkColorPalette
  { colors = ["#0d9488", "#3b82f6", "#8b5cf6", "#ef4444", "#f59e0b"]
  }

||| Default chart options
public export
defaultChartOptions : ChartOptions
defaultChartOptions = MkChartOptions
  { title = Nothing
  , xAxis = Nothing
  , yAxis = Nothing
  , legend = defaultLegend
  , grid = defaultGrid
  , palette = defaultPalette
  , interactive = True
  }

------------------------------------------------------------------------
-- Chart validation
------------------------------------------------------------------------

||| Validate that chart data matches chart type
||| Returns Left with error message if invalid
public export
validateChartData : ChartType -> ChartData -> Either String ()
validateChartData Pie chartData =
  case series chartData of
    [] => Left "Pie chart requires at least one data series"
    [s] => Right ()
    _ => Left "Pie chart supports only one data series"
validateChartData _ chartData =
  case series chartData of
    [] => Left "Chart requires at least one data series"
    _ => Right ()
