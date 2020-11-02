Function("return this;")().require = require;
var vm = require("vm");
var eshostContext = vm.createContext({
  setTimeout,
  require,
  console,
  print(...args) {
    console.log(...args);
  }
});
vm.runInESHostContext = function(code, options) {
  return vm.runInContext(code, eshostContext, options);
};
vm.runInESHostContext(${code});
