from setuptools import setup, find_packages

setup(
  name='hushlexer',
  packages=find_packages(),
  entry_points =
  '''
  [pygments.lexers]
  hushlexer = lexer.hush:HushLexer
  ''',
)
