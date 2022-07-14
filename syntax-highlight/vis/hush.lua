-- ? LPeg lexer.

local l = require('lexer')
local token, word_match = l.token, l.word_match
local P, R, S = lpeg.P, lpeg.R, lpeg.S

local M = {_NAME = 'hush'}

local comment = token(l.COMMENT, '#' * l.nonnewline_esc^0)

local constant = token(l.CONSTANT, word_match{
  'true', 'false',
})

local identifier = token(l.IDENTIFIER, l.word)

local keyword = token(l.KEYWORD, word_match{
  'let', 'if', 'then', 'else', 'elseif', 'end', 'for', 'in', 'do', 'while',
  'function', 'return', 'not', 'and', 'or', 'true', 'false', 'nil', 'break',
  'self',
})

local operator = token(l.OPERATOR, word_match{
  'and', 'or', 'not',
} + S('+-/*%<>!=[]'))

local number = token(l.NUMBER, l.float + l.integer)

local sq_str = l.delimited_range("'", true)
local dq_str = l.delimited_range('"', true)
local string = token(l.STRING, sq_str + dq_str)

local type = token(l.TYPE, word_match{
  'int', 'char', 'float', 'string', 'bool', 'array', 'dict',
})

local ws = token(l.WHITESPACE, l.space^1)

M._rules = {
  {'constant', constant},
  {'comment', comment},
  {'keyword', keyword},
  {'number', number},
  {'operator', operator},
  {'string', string},
  {'type', type},
  {'whitespace', ws},
  {'identifier', identifier},
}

return M
