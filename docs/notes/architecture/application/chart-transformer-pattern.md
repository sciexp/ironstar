---
title: Chart transformer pattern
---

# Chart transformer pattern

> **Semantic foundation**: Chart transformation implements quotient projection.
> `ChartTransformer` is a functor from DuckDB query results to ECharts option JSON.
> Multiple QueryResult instances may map to the same chart (many-to-one quotient).
> See [semantic-model.md § DuckDB analytics as quotient](../core/semantic-model.md#duckdb-analytics-as-quotient-with-memoization).

The ChartTransformer pattern bridges the analytics layer (DuckDB query results) and the visualization layer (ECharts options JSON).
This document describes the trait design, column-to-axis mapping strategy, chart type configurations, and error handling patterns for transforming tabular data into chart specifications.

## The transformation problem

DuckDB returns query results as tabular data with typed columns and rows.
ECharts requires structured JSON configuration objects with specific schemas for each chart type.
The transformation must:

1. Infer semantic meaning from column types and names (category vs value axes)
2. Map tabular rows to chart-specific data structures (series arrays, pie slices, scatter points)
3. Generate appropriate axis configurations based on data characteristics
4. Handle missing data, type mismatches, and empty result sets gracefully
5. Preserve metadata for tooltips and legends

**Example transformation:**

```
DuckDB result:
┌──────────┬───────┐
│  month   │ sales │
├──────────┼───────┤
│ January  │  1500 │
│ February │  2300 │
│ March    │  1800 │
└──────────┴───────┘

ECharts option:
{
  "xAxis": { "type": "category", "data": ["January", "February", "March"] },
  "yAxis": { "type": "value" },
  "series": [{ "type": "bar", "data": [1500, 2300, 1800] }]
}
```

## ChartTransformer trait

The trait abstracts chart type differences while providing a consistent transformation interface.

```rust
use serde_json::Value as JsonValue;

/// Result from DuckDB query with column metadata
pub struct QueryResult {
    pub columns: Vec<ColumnMetadata>,
    pub rows: Vec<Vec<JsonValue>>,
}

pub struct ColumnMetadata {
    pub name: String,
    pub data_type: ColumnType,
}

pub enum ColumnType {
    Text,
    Integer,
    Float,
    Date,
    Timestamp,
    Boolean,
}

/// Configuration hints for transformation
pub struct ChartConfig {
    pub chart_type: ChartType,
    pub title: Option<String>,
    pub category_column: Option<String>, // Explicit column hint
    pub value_columns: Vec<String>,      // Explicit value columns
}

pub enum ChartType {
    Bar,
    Line,
    Pie,
    Scatter,
}

/// Transform DuckDB results into ECharts options
pub trait ChartTransformer {
    /// Transform query result into ECharts option JSON
    fn transform(
        &self,
        result: &QueryResult,
        config: &ChartConfig,
    ) -> Result<JsonValue, TransformError>;

    /// Validate that the result shape is compatible with this chart type
    fn validate(&self, result: &QueryResult) -> Result<(), TransformError>;

    /// Infer column roles (category vs value) from metadata
    fn infer_columns(&self, result: &QueryResult) -> ColumnAssignment;
}

pub struct ColumnAssignment {
    pub category_column: Option<usize>,
    pub value_columns: Vec<usize>,
}

#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    #[error("No data rows in query result")]
    EmptyResult,

    #[error("Chart type {chart_type} requires at least {required} columns, found {actual}")]
    InsufficientColumns {
        chart_type: String,
        required: usize,
        actual: usize,
    },

    #[error("Column '{column}' has incompatible type {actual} for {expected} axis")]
    IncompatibleColumnType {
        column: String,
        actual: String,
        expected: String,
    },

    #[error("Could not infer category column from available columns")]
    NoCategoryColumn,

    #[error("No numeric columns found for value axis")]
    NoValueColumns,

    #[error("JSON serialization failed: {0}")]
    SerializationError(#[from] serde_json::Error),
}
```

**Trait implementation pattern:**

Each chart type implements `ChartTransformer` with type-specific logic.
Implementations compose from shared utility functions for column inference and axis generation.

```rust
pub struct BarChartTransformer;

impl ChartTransformer for BarChartTransformer {
    fn validate(&self, result: &QueryResult) -> Result<(), TransformError> {
        if result.rows.is_empty() {
            return Err(TransformError::EmptyResult);
        }
        if result.columns.len() < 2 {
            return Err(TransformError::InsufficientColumns {
                chart_type: "bar".to_string(),
                required: 2,
                actual: result.columns.len(),
            });
        }
        Ok(())
    }

    fn infer_columns(&self, result: &QueryResult) -> ColumnAssignment {
        infer_columns_default(result)
    }

    fn transform(
        &self,
        result: &QueryResult,
        config: &ChartConfig,
    ) -> Result<JsonValue, TransformError> {
        self.validate(result)?;

        let assignment = self.infer_columns(result);
        let category_idx = assignment.category_column
            .ok_or(TransformError::NoCategoryColumn)?;

        if assignment.value_columns.is_empty() {
            return Err(TransformError::NoValueColumns);
        }

        let categories = extract_categories(result, category_idx);
        let series = extract_series(result, &assignment.value_columns, "bar");

        Ok(json!({
            "title": config.title.as_ref().map(|t| json!({ "text": t })),
            "xAxis": { "type": "category", "data": categories },
            "yAxis": { "type": "value" },
            "series": series,
            "tooltip": { "trigger": "axis" },
        }))
    }
}
```

## Column-to-axis mapping

The column inference strategy uses type information and naming heuristics to assign semantic roles.

**Default inference rules:**

1. **Category column** (x-axis for bar/line, label for pie):
   - First text column
   - First date/timestamp column if no text column exists
   - If multiple text columns, prefer those with "name", "label", "category" in the column name

2. **Value columns** (y-axis for bar/line, values for pie):
   - All numeric columns (integer, float)
   - Ordered by column index

3. **Special cases:**
   - Scatter charts require exactly 2 numeric columns (x and y coordinates)
   - Pie charts require exactly 1 category column and 1 value column
   - Line charts with time series prefer timestamp columns as category

**Inference implementation:**

```rust
fn infer_columns_default(result: &QueryResult) -> ColumnAssignment {
    let mut category_column = None;
    let mut value_columns = Vec::new();

    for (idx, col) in result.columns.iter().enumerate() {
        match col.data_type {
            ColumnType::Text => {
                if category_column.is_none() {
                    category_column = Some(idx);
                }
            }
            ColumnType::Date | ColumnType::Timestamp => {
                if category_column.is_none() {
                    category_column = Some(idx);
                }
            }
            ColumnType::Integer | ColumnType::Float => {
                value_columns.push(idx);
            }
            ColumnType::Boolean => {
                // Skip boolean columns unless explicitly configured
            }
        }
    }

    ColumnAssignment {
        category_column,
        value_columns,
    }
}

fn extract_categories(result: &QueryResult, category_idx: usize) -> Vec<JsonValue> {
    result.rows
        .iter()
        .map(|row| row[category_idx].clone())
        .collect()
}

fn extract_series(
    result: &QueryResult,
    value_column_indices: &[usize],
    chart_type: &str,
) -> Vec<JsonValue> {
    value_column_indices
        .iter()
        .map(|&col_idx| {
            let column_name = &result.columns[col_idx].name;
            let data: Vec<JsonValue> = result.rows
                .iter()
                .map(|row| row[col_idx].clone())
                .collect();

            json!({
                "name": column_name,
                "type": chart_type,
                "data": data,
            })
        })
        .collect()
}
```

## Chart type configurations

Each chart type has specific data shape requirements and transformation logic.

### Bar and line charts

**Requirements:**
- At least 1 category column (x-axis)
- At least 1 value column (y-axis)
- Multiple value columns create multiple series

**ECharts structure:**

```json
{
  "xAxis": { "type": "category", "data": ["A", "B", "C"] },
  "yAxis": { "type": "value" },
  "series": [
    { "name": "Series 1", "type": "bar", "data": [10, 20, 15] },
    { "name": "Series 2", "type": "bar", "data": [12, 18, 14] }
  ]
}
```

**Transformation:**

```rust
pub struct LineChartTransformer;

impl ChartTransformer for LineChartTransformer {
    // Same validation and inference as BarChartTransformer

    fn transform(
        &self,
        result: &QueryResult,
        config: &ChartConfig,
    ) -> Result<JsonValue, TransformError> {
        self.validate(result)?;

        let assignment = self.infer_columns(result);
        let category_idx = assignment.category_column
            .ok_or(TransformError::NoCategoryColumn)?;

        if assignment.value_columns.is_empty() {
            return Err(TransformError::NoValueColumns);
        }

        let categories = extract_categories(result, category_idx);
        let series = extract_series(result, &assignment.value_columns, "line");

        Ok(json!({
            "title": config.title.as_ref().map(|t| json!({ "text": t })),
            "xAxis": { "type": "category", "data": categories },
            "yAxis": { "type": "value" },
            "series": series,
            "tooltip": { "trigger": "axis" },
        }))
    }
}
```

### Pie charts

**Requirements:**
- Exactly 1 category column (labels)
- Exactly 1 value column (slice sizes)

**ECharts structure:**

```json
{
  "series": [
    {
      "type": "pie",
      "data": [
        { "name": "Category A", "value": 335 },
        { "name": "Category B", "value": 234 },
        { "name": "Category C", "value": 548 }
      ]
    }
  ]
}
```

**Transformation:**

```rust
pub struct PieChartTransformer;

impl ChartTransformer for PieChartTransformer {
    fn validate(&self, result: &QueryResult) -> Result<(), TransformError> {
        if result.rows.is_empty() {
            return Err(TransformError::EmptyResult);
        }
        if result.columns.len() != 2 {
            return Err(TransformError::InsufficientColumns {
                chart_type: "pie".to_string(),
                required: 2,
                actual: result.columns.len(),
            });
        }
        Ok(())
    }

    fn infer_columns(&self, result: &QueryResult) -> ColumnAssignment {
        infer_columns_default(result)
    }

    fn transform(
        &self,
        result: &QueryResult,
        config: &ChartConfig,
    ) -> Result<JsonValue, TransformError> {
        self.validate(result)?;

        let assignment = self.infer_columns(result);
        let category_idx = assignment.category_column
            .ok_or(TransformError::NoCategoryColumn)?;

        if assignment.value_columns.is_empty() {
            return Err(TransformError::NoValueColumns);
        }

        let value_idx = assignment.value_columns[0];

        let data: Vec<JsonValue> = result.rows
            .iter()
            .map(|row| {
                json!({
                    "name": row[category_idx],
                    "value": row[value_idx],
                })
            })
            .collect();

        Ok(json!({
            "title": config.title.as_ref().map(|t| json!({ "text": t })),
            "series": [
                {
                    "type": "pie",
                    "data": data,
                    "radius": "50%",
                }
            ],
            "tooltip": { "trigger": "item" },
        }))
    }
}
```

### Scatter charts

**Requirements:**
- Exactly 2 numeric columns (x and y coordinates)
- Optional 3rd numeric column for bubble size

**ECharts structure:**

```json
{
  "xAxis": { "type": "value" },
  "yAxis": { "type": "value" },
  "series": [
    {
      "type": "scatter",
      "data": [[10, 20], [15, 25], [12, 18]]
    }
  ]
}
```

**Transformation:**

```rust
pub struct ScatterChartTransformer;

impl ChartTransformer for ScatterChartTransformer {
    fn validate(&self, result: &QueryResult) -> Result<(), TransformError> {
        if result.rows.is_empty() {
            return Err(TransformError::EmptyResult);
        }

        let numeric_count = result.columns
            .iter()
            .filter(|col| matches!(col.data_type, ColumnType::Integer | ColumnType::Float))
            .count();

        if numeric_count < 2 {
            return Err(TransformError::InsufficientColumns {
                chart_type: "scatter".to_string(),
                required: 2,
                actual: numeric_count,
            });
        }

        Ok(())
    }

    fn infer_columns(&self, result: &QueryResult) -> ColumnAssignment {
        let value_columns: Vec<usize> = result.columns
            .iter()
            .enumerate()
            .filter(|(_, col)| matches!(col.data_type, ColumnType::Integer | ColumnType::Float))
            .map(|(idx, _)| idx)
            .take(2) // Only use first 2 numeric columns
            .collect();

        ColumnAssignment {
            category_column: None,
            value_columns,
        }
    }

    fn transform(
        &self,
        result: &QueryResult,
        config: &ChartConfig,
    ) -> Result<JsonValue, TransformError> {
        self.validate(result)?;

        let assignment = self.infer_columns(result);
        if assignment.value_columns.len() < 2 {
            return Err(TransformError::NoValueColumns);
        }

        let x_idx = assignment.value_columns[0];
        let y_idx = assignment.value_columns[1];

        let data: Vec<JsonValue> = result.rows
            .iter()
            .map(|row| json!([row[x_idx], row[y_idx]]))
            .collect();

        Ok(json!({
            "title": config.title.as_ref().map(|t| json!({ "text": t })),
            "xAxis": { "type": "value" },
            "yAxis": { "type": "value" },
            "series": [
                {
                    "type": "scatter",
                    "data": data,
                }
            ],
            "tooltip": { "trigger": "item" },
        }))
    }
}
```

## Error handling

The transformer layer validates data shape and column types before transformation.
Errors propagate to the handler layer where they become user-facing SSE error events.

**Error handling strategy:**

1. **Validation errors** (empty results, insufficient columns): Return clear error messages indicating what the chart type requires
2. **Type mismatch errors**: Indicate which column has the wrong type and what was expected
3. **Inference failures**: Suggest explicit column configuration when automatic inference fails

**Example error handling in handler:**

```rust
async fn handle_chart_request(
    query: String,
    chart_config: ChartConfig,
    pool: &DuckDbPool,
) -> Result<JsonValue, HandlerError> {
    let result = execute_query(pool, &query).await?;

    let transformer: Box<dyn ChartTransformer> = match chart_config.chart_type {
        ChartType::Bar => Box::new(BarChartTransformer),
        ChartType::Line => Box::new(LineChartTransformer),
        ChartType::Pie => Box::new(PieChartTransformer),
        ChartType::Scatter => Box::new(ScatterChartTransformer),
    };

    transformer.transform(&result, &chart_config)
        .map_err(|e| HandlerError::ChartTransformation {
            chart_type: chart_config.chart_type,
            source: e,
        })
}
```

**User-facing error messages:**

```rust
impl From<TransformError> for String {
    fn from(err: TransformError) -> String {
        match err {
            TransformError::EmptyResult => {
                "Query returned no data. Check your filters and try again.".to_string()
            }
            TransformError::InsufficientColumns { chart_type, required, actual } => {
                format!(
                    "{} charts require at least {} columns, but your query returned {}. \
                     Try selecting more columns or choosing a different chart type.",
                    chart_type, required, actual
                )
            }
            TransformError::NoCategoryColumn => {
                "Could not find a text or date column for the x-axis. \
                 Try explicitly specifying a category column.".to_string()
            }
            TransformError::NoValueColumns => {
                "Could not find any numeric columns for the chart values. \
                 Ensure your query includes at least one numeric column.".to_string()
            }
            TransformError::IncompatibleColumnType { column, actual, expected } => {
                format!(
                    "Column '{}' has type {} but {} axis requires {}. \
                     Try casting the column or selecting a different column.",
                    column, actual, expected, expected
                )
            }
            TransformError::SerializationError(e) => {
                format!("Failed to generate chart configuration: {}", e)
            }
        }
    }
}
```

## Example transformation

Complete example showing the full transformation pipeline from SQL query to ECharts options.

**Input query:**

```sql
SELECT
    product_category,
    SUM(revenue) as total_revenue,
    COUNT(*) as order_count
FROM orders
WHERE order_date >= '2024-01-01'
GROUP BY product_category
ORDER BY total_revenue DESC
LIMIT 10;
```

**DuckDB result:**

```
QueryResult {
    columns: [
        ColumnMetadata { name: "product_category", data_type: Text },
        ColumnMetadata { name: "total_revenue", data_type: Float },
        ColumnMetadata { name: "order_count", data_type: Integer },
    ],
    rows: [
        [json!("Electronics"), json!(45623.50), json!(127)],
        [json!("Clothing"), json!(38912.25), json!(201)],
        [json!("Home & Garden"), json!(32145.75), json!(89)],
    ],
}
```

**Chart configuration:**

```rust
let config = ChartConfig {
    chart_type: ChartType::Bar,
    title: Some("Revenue by Product Category".to_string()),
    category_column: None, // Will be inferred
    value_columns: vec![], // Will be inferred
};
```

**Transformation process:**

```rust
let transformer = BarChartTransformer;

// 1. Validate
transformer.validate(&result)?;
// ✓ 3 rows, 3 columns (meets minimum requirements)

// 2. Infer columns
let assignment = transformer.infer_columns(&result);
// category_column = Some(0)  // "product_category" is text
// value_columns = [1, 2]     // "total_revenue" and "order_count" are numeric

// 3. Transform
let echarts_option = transformer.transform(&result, &config)?;
```

**Output ECharts option:**

```json
{
  "title": {
    "text": "Revenue by Product Category"
  },
  "xAxis": {
    "type": "category",
    "data": ["Electronics", "Clothing", "Home & Garden"]
  },
  "yAxis": {
    "type": "value"
  },
  "series": [
    {
      "name": "total_revenue",
      "type": "bar",
      "data": [45623.50, 38912.25, 32145.75]
    },
    {
      "name": "order_count",
      "type": "bar",
      "data": [127, 201, 89]
    }
  ],
  "tooltip": {
    "trigger": "axis"
  }
}
```

**SSE transmission:**

The handler serializes the ECharts option and sends it via datastar signal patch:

```rust
use datastar::prelude::*;

async fn send_chart_update(
    sse: Sse,
    echarts_option: JsonValue,
) -> Result<(), HandlerError> {
    let signal = ServerSignal::new("chartOptions", echarts_option);
    sse.send_event(Event::PatchSignals(vec![signal])).await?;
    Ok(())
}
```

**Client-side rendering:**

The ds-echarts web component receives the signal and renders:

```html
<ds-echarts data-signal-chart-options></ds-echarts>
```

```typescript
// Inside ds-echarts component
connectedCallback() {
  this.chart = echarts.init(this);

  // Watch for signal updates
  this.datastar?.signal?.watch('chartOptions', (options) => {
    this.chart.setOption(options);
  });
}
```

The complete round-trip:
1. User submits query + chart type selection
2. Handler executes DuckDB query → QueryResult
3. ChartTransformer converts QueryResult → ECharts option JSON
4. Handler sends ECharts option via SSE → datastar signal
5. ds-echarts web component receives signal → renders chart

All type transformations are explicit and validated at each boundary.
