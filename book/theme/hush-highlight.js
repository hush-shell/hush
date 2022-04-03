hljs.registerLanguage("hush", (hljs) => ({
  name: "Hush",
  keywords: {
    keyword: "let if then else end for in do while function return not and or break self",
    literal: "false nil true",
  },
  contains: [
    hljs.QUOTE_STRING_MODE,
    hljs.C_NUMBER_MODE,
    {
      scope: "string",
      begin: '"',
      end: '"',
      contains: [{ begin: "\\\\." }],
    },
    hljs.COMMENT("#", "$"),
  ],
}));

hljs.initHighlightingOnLoad();
