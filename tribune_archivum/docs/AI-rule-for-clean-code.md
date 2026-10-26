# Clean Code

## 1. Readability

Write code that can be understood with a short read, without comments.

If you feel the need to comment on code inside a function, extract that code into a separate function named so that its purpose is clear from the name. Extract a function when a short read isn't enough.

Doc comments above a function are allowed when the how or why cannot be conveyed in its name. Keep them to reasoning a reader cannot recover from the code.

A function must earn its keep: the call site has to read more easily than the code it replaced. A function that only wraps a single expression, or that is called from one place and is no clearer than its body, usually doesn't.

## 2. Design Before Implementation

Before implementing a non-trivial change, determine what needs to happen and how the responsibilities should be divided.

Identify the main steps, their dependencies, and the boundaries between distinct responsibilities before writing their implementations.

Design the function structure before filling in the details. Function names and call sites should describe the flow of the solution, allowing the reader to understand the overall behavior without reading every implementation.

Keep the design proportional to the task. Small, straightforward changes do not require a formal plan or unnecessary abstractions.

If implementation reveals that the design is wrong, revise the design rather than forcing the implementation to fit it.

## 3. Functions and Responsibilities

Each function should have one clear responsibility, expressed by its name.

Separate distinct responsibilities into functions when doing so makes the code easier to understand. Avoid both excessively large functions and excessive fragmentation into trivial helpers.

Prefer explicit data flow and straightforward control flow over indirection, hidden side effects, and unnecessary layers of abstraction.

Keep related behavior together. Introduce new modules, types, interfaces, or abstractions only when they establish a meaningful boundary or simplify the design.

## 4. Tests

Group tests that test similar things under a common `mod`, with good naming.

Test observable behavior rather than implementation details. Test names should describe the behavior or outcome being verified.

Organize tests around meaningful behaviors and scenarios, including relevant edge cases and failure conditions. Avoid duplicating coverage without a clear reason.

When changing behavior, update or add tests that demonstrate the intended behavior and protect against regressions.

## 5. Verification

After implementing a change, run the relevant tests and checks.

When a change affects multiple responsibilities, verify their integration as well as their individual behavior where practical.

Do not consider a change complete merely because the code compiles or the implementation looks correct. Investigate failures introduced by the change, and report checks that could not be run.

Do not weaken tests, remove existing behavior, or suppress errors merely to make the change pass.


