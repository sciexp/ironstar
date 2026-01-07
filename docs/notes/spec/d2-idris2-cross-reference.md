# D2 diagram to Idris2 spec cross-reference

This document records the initial cross-reference between D2 event modeling diagrams and Idris2 type specifications.
Generated during issue ironstar-2it.21 (integration module creation).
Gaps identified here feed into issue ironstar-2it.16 (co-refinement).

## Summary

| Context | Aggregates | Commands | Events | Read Models | Status |
|---------|------------|----------|--------|-------------|--------|
| Session | 1/1 | 3/5 | 4/5 | 1/2 | Partial |
| Analytics | 3/3 | 4/8 | 4/9 | 1/4 | Partial |
| Workspace | 3/3 | 6/4 | 6/4 | 0/1 | Good |
| Todo | 1/1 | 4/6 | 4/6 | 1/1 | Good |

Legend: Idris2 count / D2 count

## Session context (Supporting)

**Aggregate alignment: Session**

| D2 Element | Idris2 Element | Status |
|------------|----------------|--------|
| Session (aggregate) | `Session.Session` module | Aligned |
| UserIdentity (VO) | `UserId` record | Aligned |

**Command gaps:**

| D2 Command | Idris2 Command | Gap Analysis |
|------------|----------------|--------------|
| InitiateOAuth | - | Missing: OAuth initiation not modeled |
| ProcessOAuthCallback | - | Missing: OAuth callback automation not modeled |
| CreateSession | `CreateSession` | Aligned |
| ExpireSession | - | Missing: Expiration is passive (boundary-driven) |
| Logout | `InvalidateSession` | Semantic match |
| - | `RefreshSession` | Extra: TTL extension not in D2 |

**Event gaps:**

| D2 Event | Idris2 Event | Gap Analysis |
|----------|--------------|--------------|
| OAuthLoginInitiated | - | Missing: OAuth flow start event |
| UserAuthenticatedViaGithub | - | Missing: Provider-specific auth event |
| SessionCreated | `SessionCreated` | Aligned |
| SessionExpired | `SessionExpired` | Aligned |
| UserLoggedOut | `SessionInvalidated` | Semantic match |
| - | `SessionRefreshed` | Extra: TTL extension event |

**Read model gaps:**

| D2 Read Model | Idris2 View | Gap Analysis |
|---------------|-------------|--------------|
| LoginOptions | - | Missing: Pre-auth UI state |
| SessionInfo | `ActiveSessionView` | Similar purpose |

**Resolution for .16:**
D2 models the full OAuth flow while Idris2 abstracts it.
Either expand Idris2 with OAuth lifecycle types OR simplify D2 to match the "session lifecycle starts after OAuth callback" boundary.
Recommendation: Keep Idris2 boundary-focused; OAuth details are infrastructure.

## Analytics context (Core)

**Aggregate alignment:**

| D2 Aggregate | Idris2 Module | Status |
|--------------|---------------|--------|
| QuerySession | `Analytics.QuerySession` | Aligned |
| Catalog | `Analytics.Catalog` | Aligned |
| ChartDefinition | `Analytics.Chart` | Clarified: Shared Kernel value objects, not aggregate |

**Command gaps:**

| D2 Command | Idris2 Command | Gap Analysis |
|------------|----------------|--------------|
| StartQuery | `StartQuery` | Aligned |
| CancelQuery | `CancelQuery` | Aligned |
| SelectCatalog | `SelectCatalog` | Aligned |
| RefreshCatalogMetadata | `RefreshCatalogMetadata` | Aligned |
| BeginExecution | - | Intentional: Automation, not domain command |
| CompleteQuery | - | Intentional: Maps to boundary filling `QueryCompleted` event |
| FailQuery | - | Intentional: Maps to boundary filling `QueryFailed` event |
| InvalidateCache | - | Intentional: Infrastructure concern, not domain command |

**Event gaps:**

| D2 Event | Idris2 Event | Gap Analysis |
|----------|--------------|--------------|
| QueryStarted | `QueryStarted` | Aligned |
| QueryCancelled | `QueryCancelled` | Aligned |
| ExecutionBegan | - | Intentional: Infrastructure progress signal |
| QueryCompleted | `QueryCompleted` | Aligned |
| QueryFailed | `QueryFailed` | Aligned |
| CatalogSelected | `CatalogSelected` | Aligned |
| CatalogMetadataRefreshed | `CatalogMetadataRefreshed` | Aligned |
| CacheInvalidated | - | Intentional: Infrastructure event |
| ChartDataProjected | - | Intentional: Projection completion signal |

**Read model gaps:**

| D2 Read Model | Idris2 View | Gap Analysis |
|---------------|-------------|--------------|
| DatasetBrowser | - | Missing: Could add `CatalogView` |
| QueryHistory | `QueryHistory` | Aligned |
| QueryResults | `QueryResults` (VO) | Value object, not projection |
| ChartData | `ChartData` (VO) | Value object in Analytics.Chart |

**Resolution for .16:**
D2 shows more infrastructure events (ExecutionBegan, CacheInvalidated, ChartDataProjected) than Idris2.
These are appropriate for sequence diagrams but not domain events.
Recommendation: Keep Idris2 focused on domain events; D2 shows full system behavior.

## Workspace context (Supporting)

**Aggregate alignment:**

| D2 Aggregate | Idris2 Module | Status |
|--------------|---------------|--------|
| Dashboard | `Workspace.Dashboard` | Aligned |
| SavedQuery | `Workspace.SavedQuery` | Aligned |
| UserPreferences | `Workspace.Preferences` | Aligned |

**Command alignment:**

| D2 Command | Idris2 Command | Status |
|------------|----------------|--------|
| CreateDashboard | `CreateDashboard` | Aligned |
| AddWidget | `AddChart` | Semantic match |
| SaveQuery | `SaveQuery` | Aligned |
| UpdatePreferences | `UpdatePreferences` | Aligned |
| - | `RemoveChart` | Extra in Idris2 |
| - | `AddTab` | Extra in Idris2 |
| - | `MoveChartToTab` | Extra in Idris2 |
| - | `RenameDashboard` | Extra in Idris2 |

**Event alignment:**

| D2 Event | Idris2 Event | Status |
|----------|--------------|--------|
| DashboardCreated | `DashboardCreated` | Aligned |
| WidgetAdded | `ChartAdded` | Semantic match |
| QuerySaved | `QuerySaved` | Aligned |
| PreferencesUpdated | `PreferencesUpdated` | Aligned |
| - | `ChartRemoved` | Extra in Idris2 |
| - | `TabAdded` | Extra in Idris2 |
| - | `ChartMovedToTab` | Extra in Idris2 |
| - | `DashboardRenamed` | Extra in Idris2 |

**Read model gaps:**

| D2 Read Model | Idris2 View | Gap Analysis |
|---------------|-------------|--------------|
| DashboardView | - | Missing: Could add View type |

**Resolution for .16:**
Idris2 has MORE commands/events than D2 (more complete).
D2 diagrams should be expanded to match Idris2 completeness.
Recommendation: Update D2 to include tab management and chart removal.

## Todo context (Generic Example)

**Aggregate alignment:**

| D2 Aggregate | Idris2 Module | Status |
|--------------|---------------|--------|
| Todo | `Todo.Todo` | Aligned |

**Command gaps:**

| D2 Command | Idris2 Command | Gap Analysis |
|------------|----------------|--------------|
| CreateTodo | `Create` | Aligned |
| UpdateTodoText | - | Missing in Idris2 |
| CompleteTodo | `Complete` | Aligned |
| UncompleteTodo | `Uncomplete` | Aligned |
| DeleteTodo | `Delete` | Aligned |
| UpdateProjection | - | Intentional: Infrastructure automation |

**Event gaps:**

| D2 Event | Idris2 Event | Gap Analysis |
|----------|--------------|--------------|
| TodoCreated | `Created` | Aligned |
| TodoTextUpdated | - | Missing in Idris2 |
| TodoCompleted | `Completed` | Aligned |
| TodoUncompleted | `Uncompleted` | Aligned |
| TodoDeleted | `Deleted` | Aligned |
| TodoListProjected | - | Intentional: Projection signal |

**Read model alignment:**

| D2 Read Model | Idris2 View | Status |
|---------------|-------------|--------|
| TodoList | `TodoListView` | Aligned |

**Resolution for .16:**
Todo is intentionally minimal in Idris2 (Generic Example domain).
The missing UpdateTodoText could be added if desired for completeness.
Recommendation: Keep Todo minimal; complexity demonstrates in Analytics/Workspace.

## Cross-context integration

**Shared Kernel verification:**

| Shared Type | Contexts | Status |
|-------------|----------|--------|
| UserId | Session, Workspace | Session exports; Workspace should import |
| ChartDefinitionRef | Analytics, Workspace | Workspace imports Analytics.Chart |

**Customer-Supplier relationships:**

| Upstream | Downstream | Idris2 Status |
|----------|------------|---------------|
| Session | Analytics | Not explicit (boundary concern) |
| Session | Workspace | Not explicit (boundary concern) |
| Analytics | Workspace | Workspace.Dashboard imports Analytics.Chart |

**Zenoh key expression patterns:**
D2 documents `events/{context}/{id}/**` patterns.
Idris2 includes key expression constants in Analytics.Analytics.
Not yet formalized in Session, Workspace, Todo.

## Gap summary for .16 co-refinement

### High priority (structural alignment)

1. **Session OAuth lifecycle**: Decide if D2 should simplify or Idris2 should expand
2. **Workspace D2 update**: Add tab management, chart removal to D2 diagrams
3. **DashboardView**: Add View type to Workspace.Dashboard

### Medium priority (completeness)

4. **DatasetBrowser view**: Consider adding CatalogView to Analytics
5. **Todo UpdateTodoText**: Consider adding text update command/event
6. **Zenoh key patterns**: Formalize in all context modules

### Low priority (documentation)

7. **Infrastructure events in D2**: Annotate as "infrastructure" vs "domain"
8. **Cross-context dependencies**: Document import relationships more explicitly
