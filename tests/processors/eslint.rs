test_checker!(eslint, tool: "eslint", processor: "eslint",
    files: [(".eslintrc.json", "{}\n"), ("test.js", "var x = 1;\n")]);
