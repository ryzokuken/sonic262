(async function() {
  "use strict";
  var vm = require("vm");
  var fs = require("fs");
  var path = require("path");

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

  // Run harness preamble as script
  vm.runInContext(${preamble}, context);

  // Load and evaluate the test as a module
  var testDir = ${testDir};
  var testCode = ${testCode};

  async function linker(specifier, referencingModule) {
    var resolved;
    if (specifier.startsWith("./")) {
      resolved = path.resolve(testDir, specifier);
    } else {
      throw new Error("Unsupported module specifier: " + specifier);
    }
    var source = fs.readFileSync(resolved, "utf8");
    var mod = new vm.SourceTextModule(source, {
      context: context,
      identifier: resolved
    });
    await mod.link(linker);
    return mod;
  }

  var mod = new vm.SourceTextModule(testCode, {
    context: context,
    identifier: "test-module"
  });
  await mod.link(linker);
  await mod.evaluate();
})().catch(function(err) {
  console.error(err);
  process.exitCode = 1;
});
