from pygments.lexer import RegexLexer, include, default, combined
from pygments.token import Text, Comment, Operator, Keyword, Name, String, \
    Number, Punctuation


class HushLexer(RegexLexer):
    """
    For `Hush <https://www.github.com/gahag/hush>` scripts.
    """

    name = 'Hush'
    aliases = ['hush']
    filenames = ['*.hsh']
    mimetypes = ['text/x-hush', 'application/x-hush']

    _comment = r'(?:#.*$)'
    _space = r'(?:\s+)'
    _s = r'(?:%s|%s)' % (_comment, _space)
    _name = r'(?:[^\W\d]\w*)'

    tokens = {
        'root': [
            # Hush allows a file to start with a shebang.
            (r'#!.*', Comment.Preproc),
            default('base'),
        ],
        'ws': [
            (_comment, Comment.Single),
            (_space, Text),
        ],
        'base': [
            include('ws'),

            (r'(?i)0x[\da-f]*(\.[\da-f]*)?(p[+-]?\d+)?', Number.Hex),
            (r'(?i)(\d*\.\d+|\d+\.\d*)(e[+-]?\d+)?', Number.Float),
            (r'(?i)\d+e[+-]?\d+', Number.Float),
            (r'\d+', Number.Integer),

            # multiline strings
            # (r'(?s)\[(=*)\[.*?\]\1\]', String),

            # (r'::', Punctuation, 'label'),
            # (r'\.{3}', Punctuation),
            (r'[\?\$!=<>{}|+\-*/%]+', Operator),
            (r'[\[\]().,:;]|@\[', Punctuation),
            (r'(and|or|not)\b', Operator.Word),

            (r'(break|self|do|else|elseif|end|for|if|in|return|then|while)\b', Keyword.Reserved),
            (r'(let)\b', Keyword.Declaration),
            (r'(true|false|nil)\b', Keyword.Constant),

            (r'(function)\b', Keyword.Reserved, 'funcname'),

            (r'[A-Za-z_]\w*(\.[A-Za-z_]\w*)?', Name),

            ("'", String.Char, combined('stringescape', 'sqs')), # TODO
            ('"', String.Double, combined('stringescape', 'dqs'))
        ],

        'funcname': [
            include('ws'),
            (_name, Name.Function, '#pop'),
            # inline function
            (r'\(', Punctuation, '#pop'),
        ],

        'stringescape': [
            (r'\\([abfnrtv\\"\']|[\r\n]{1,2}|z\s*|x[0-9a-fA-F]{2}|\d{1,3}|'
             r'u\{[0-9a-fA-F]+\})', String.Escape),
        ],

        'sqs': [
            (r"'", String.Single, '#pop'),
            (r"[^\\']+", String.Single),
        ],

        'dqs': [
            (r'"', String.Double, '#pop'),
            (r'[^\\"]+', String.Double),
        ]
    }

    def get_tokens_unprocessed(self, text):
        for index, token, value in RegexLexer.get_tokens_unprocessed(self, text):
            if token is Name:
                if '.' in value:
                    a, b = value.split('.')
                    yield index, Name, a
                    yield index + len(a), Punctuation, '.'
                    yield index + len(a) + 1, Name, b
                    continue
            yield index, token, value
