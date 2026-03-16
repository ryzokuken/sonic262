/*---
description: Only runs in strict mode
flags: [onlyStrict]
---*/
// In strict mode, assigning to an undeclared variable throws
var threw = false;
try {
  (function() { undeclaredVar = 1; })();
} catch (e) {
  threw = true;
}
assert.sameValue(threw, true);
