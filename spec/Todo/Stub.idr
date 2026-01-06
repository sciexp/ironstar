||| Todo bounded context stub
|||
||| Placeholder for ironstar-2it.20: Formalize Todo bounded context in Idris2 (Generic Example)
|||
||| Domain classification: Generic domain
||| - Todo serves as the canonical example aggregate for the template
||| - Demonstrates full Decider/View/Saga patterns
|||
||| Key abstractions to formalize:
||| - TodoCommand: Create, Complete, Uncomplete, Delete
||| - TodoEvent: Created, Completed, Uncompleted, Deleted
||| - TodoState: NonExistent, Active, Completed (state machine)
||| - TodoDecider: full command handling with validation
||| - TodoView: projections for list and detail views
||| - TodoSaga: example process manager (notifications on completion)
|||
||| This module serves as the worked example demonstrating:
||| 1. State machine encoding as sum types
||| 2. Command validation with Either err (List e)
||| 3. Totality of evolve (events cannot fail)
||| 4. Replay determinism proofs
||| 5. Composition with other aggregates via combine
|||
||| Dependencies: Core.Decider, Core.View, Core.Saga, Core.Effect, Core.Event
module Todo.Stub

%default total

||| Placeholder: Todo module not yet implemented
||| See ironstar-2it.20 for full specification
|||
||| Sketch of intended types (to be elaborated in .20):
|||
||| data TodoCommand = Create String | Complete | Uncomplete | Delete
||| data TodoEvent = Created String Timestamp | Completed Timestamp | ...
||| data TodoState = NonExistent | Active String | Completed String
|||
||| todoDecider : Decider TodoCommand TodoState TodoEvent String
public export
todoStub : ()
todoStub = ()
