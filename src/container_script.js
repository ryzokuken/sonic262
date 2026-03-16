(function() {
  "use strict";
  var vm = require("vm");
  var context = vm.createContext({
    setTimeout: setTimeout,
    console: console,
    print: function() {
      console.log.apply(console, arguments);
    },
    $262: {
      global: null,
      evalScript: function(code) {
        return vm.runInContext(code, context);
      },
      createRealm: function() {
        throw new Error("$262.createRealm is not supported by sonic262");
      },
      detachArrayBuffer: function() {
        throw new Error("$262.detachArrayBuffer is not supported by sonic262");
      },
      gc: function() {
        throw new Error("$262.gc is not supported by sonic262");
      },
      agent: {
        start: function() { throw new Error("$262.agent is not supported by sonic262"); },
        broadcast: function() { throw new Error("$262.agent is not supported by sonic262"); },
        getReport: function() { throw new Error("$262.agent is not supported by sonic262"); },
        sleep: function() { throw new Error("$262.agent is not supported by sonic262"); },
        monotonicNow: function() { throw new Error("$262.agent is not supported by sonic262"); }
      }
    }
  });
  context.$262.global = context;
  vm.runInContext(${code}, context);
})();
