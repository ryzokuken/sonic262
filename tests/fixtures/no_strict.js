/*---
description: Only runs in non-strict mode
flags: [noStrict]
---*/
// In non-strict mode, assigning to an undeclared variable succeeds
(function() { undeclaredVar = 1; })();
assert.sameValue(undeclaredVar, 1);
