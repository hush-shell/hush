(define-generic-mode 'hush-mode
  ;; Comments:
  '("#")
  ;; Keywords:
  '(
    "let" "if" "then" "else" "end" "for" "in" "do" "while"
    "function" "return" "not" "and" "or" "true" "false" "nil" "break" "self")
  ;; Additional definitions:
  '(("@[" . 'font-lock-operator)
    ;; ("=" . 'font-lock-operator)
    ;; (";" . 'font-lock-builtin)
    )
  ;; File extension:
  '("\\.hsh$")
  '((lambda () (modify-syntax-entry ?' "\"")))
  ;; Docstring:
  "A mode for hush scripts")
