# Error-code strategies

General order of moves for ownership and borrow errors, from best to last resort:

1. Restructure ownership: make one place own the data and pass it by value or move it once.
2. Use indices or keys into a collection instead of references held across mutations.
3. Split borrows: borrow distinct fields or slices separately, or end a borrow earlier by narrowing its scope.
4. Only then add lifetime parameters, and only then `clone()` - and a `clone()` must be justified by the data
   actually needing two owners.

Run `rustc --explain <code>` for the compiler's own explanation of any code.

## E0382 - use of moved value
A value was moved and used again. Decide which place owns it. Pass a reference to code that only reads it; move
it into the last user; restructure so the value is produced again where needed. Clone only if two independent
owners are really required.

## E0499 - more than one mutable borrow
Two `&mut` to the same data are alive at once. Shorten the first borrow so it ends before the second starts,
borrow disjoint fields separately, or work with indices instead of holding two mutable references.

## E0502 - mutable borrow while immutably borrowed
A shared borrow is still alive when a mutable one starts, often a reference into a collection kept while pushing to
it. Copy out the small value you need, keep an index instead of a reference, or finish reading before mutating.

## E0505 - move out while borrowed
A value is moved while a reference to it is alive. End the borrow before the move, or move first and borrow from the
new owner.

## E0597 - borrowed value does not live long enough
A reference outlives its referent. Return or store owned data instead of a reference to a local; move the value to
the scope that needs it. Do not try to fix this with lifetime annotations: they describe lifetimes, they do not
extend them.

## E0106 - missing lifetime specifier
A reference in a return type or struct has no lifetime the compiler can infer. First ask whether the struct or return
value should own the data instead. If borrowing is right, name the lifetime that ties the output to the input it
borrows from.

## E0277 - trait bound not satisfied
A type does not implement a required trait. Check whether the wrong type is passed (often a reference vs owned value,
or a missing `?`), add the bound to the generic parameter, or implement or derive the trait for your own type. For
`Send`/`Sync` errors in async code, find the non-`Send` value held across an `.await` and drop it before the await.

## E0308 - mismatched types
Read the expected and found types. Typical causes: a missing `?` or `.await`, a reference where an owned value is
expected, a branch returning a different type, or a trailing semicolon turning an expression into `()`. Fix the
source of the value rather than converting at the use site.

## E0599 - no method found
The method does not exist for this type, its trait is not in scope, or a bound is missing. Check the docs for the
version in `Cargo.lock`; the method may be invented or from another version. Import the trait if it exists there.

## E0425 - unresolved name
The name is not in scope: a typo, a missing `use`, a private item, or an item behind a disabled feature. Check the
module path and the crate's features; do not invent the item.
