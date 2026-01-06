#!/bin/bash
# Issue graph reconciliation for fmodel-rust adoption
# Generated 2026-01-06
#
# This script reconciles the ironstar issue graph after the fmodel-rust
# adoption decision, closing superseded nyp.* tasks and rewiring dependencies
# to the new a9b.* tasks.

set -e

echo "=== Phase 1: Remove stale dependencies ==="
echo "Removing dependencies on soon-to-be-closed issues..."

# Issues depending on nyp.3 (EventRepository)
bd dep remove ironstar-amw ironstar-nyp.3
bd dep remove ironstar-nyp.4 ironstar-nyp.3
bd dep remove ironstar-apx.4 ironstar-nyp.3
bd dep remove ironstar-zuv.1 ironstar-nyp.3

# Issues depending on nyp.7 (ProjectionManager)
bd dep remove ironstar-r62.7 ironstar-nyp.7
bd dep remove ironstar-e6k.2 ironstar-nyp.7
bd dep remove ironstar-zuv.2 ironstar-nyp.7

# Issues depending on nyp.6 (Projection trait)
bd dep remove ironstar-nyp.37 ironstar-nyp.6
bd dep remove ironstar-nyp.38 ironstar-nyp.6

echo ""
echo "=== Phase 2: Add new dependencies ==="
echo "Wiring issues to new fmodel-rust tasks..."

# Wire to a9b.1 (SQLite EventRepository with SSE extensions)
bd dep add ironstar-amw ironstar-a9b.1
bd dep add ironstar-nyp.4 ironstar-a9b.1
bd dep add ironstar-apx.4 ironstar-a9b.1
bd dep add ironstar-zuv.1 ironstar-a9b.1

# Wire to a9b.8 (MaterializedView wiring)
bd dep add ironstar-r62.7 ironstar-a9b.8
bd dep add ironstar-e6k.2 ironstar-a9b.8
bd dep add ironstar-zuv.2 ironstar-a9b.8

# Wire to a9b.5 (Todo View - pure evolve function)
bd dep add ironstar-nyp.37 ironstar-a9b.5
bd dep add ironstar-nyp.38 ironstar-a9b.5

echo ""
echo "=== Phase 3: Close superseded issues ==="
echo "Closing tasks replaced by fmodel-rust architecture..."

# Close nyp.1 - database schema superseded by a9b.3 (fmodel-rust schema)
bd close ironstar-nyp.1 --reason "Superseded by ironstar-a9b.3 which implements fmodel-rust schema with previous_id optimistic locking instead of aggregate_sequence."

# Close nyp.3 - EventRepository superseded by a9b.1 (includes SSE extensions)
bd close ironstar-nyp.3 --reason "Superseded by ironstar-a9b.1 which implements full EventRepository including SSE extension methods (query_since_sequence, earliest_sequence, latest_sequence, query_all)."

# Close nyp.6 - Projection trait superseded by fmodel-rust View
bd close ironstar-nyp.6 --reason "Superseded by fmodel-rust View struct which uses pure evolve(&S, &E) -> S instead of mutable apply(&mut self). Mathematical foundations (Galois connections) remain valid for fmodel View."

# Close nyp.7 - ProjectionManager superseded by a9b.8 (MaterializedView)
bd close ironstar-nyp.7 --reason "Superseded by ironstar-a9b.8 (Wire Todo MaterializedView). Use fmodel-rust MaterializedView per aggregate rather than centralized ProjectionManager."

echo ""
echo "=== Phase 4: Verify reconciliation ==="
bd status

echo ""
echo "✓ Issue graph reconciliation complete"
echo ""
echo "Summary:"
echo "  - Closed 4 superseded tasks (nyp.1, nyp.3, nyp.6, nyp.7)"
echo "  - Rewired 11 dependent tasks to fmodel-rust architecture"
echo "  - New dependencies:"
echo "    • a9b.1 (EventRepository): 4 dependents"
echo "    • a9b.5 (Todo View): 2 dependents"
echo "    • a9b.8 (MaterializedView): 3 dependents"
echo ""
echo "Next steps:"
echo "  1. Review bd status output for any remaining blockers"
echo "  2. Commit beads changes: bd hooks run pre-commit && git add .beads/ && git commit -m 'chore(issues): reconcile for fmodel-rust adoption'"
echo "  3. Begin implementation with a9b.* tasks"
