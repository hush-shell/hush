;; Uncomment this to make defvar override previous values.
;; (mapc #'unintern '(hush-keywords
;;                    hush-mode-syntax-table
;;                    hush-font-lock-keywords
;;                    hush-font-lock
;;                    hush-mode-map))

(defvar hush-keywords
  '("let" "if" "then" "else" "elseif" "end" "for" "in" "do" "while" "function" "return"
    "not" "and" "or" "true" "false" "nil" "break" "self"))

(defvar hush-mode-syntax-table
  (with-syntax-table (copy-syntax-table)
    ;; comment syntax: begins with "#", ends with "\n"
    (modify-syntax-entry ?# "<")
    (modify-syntax-entry ?\n ">")

    ;; main string syntax: bounded by ' or "
    (modify-syntax-entry ?\' "\"")
    (modify-syntax-entry ?\" "\"")

    ;; single-character binary operators: punctuation
    (modify-syntax-entry ?+ ".")
    (modify-syntax-entry ?- ".")
    (modify-syntax-entry ?* ".")
    (modify-syntax-entry ?/ ".")
    (modify-syntax-entry ?% ".")
    (modify-syntax-entry ?> ".")
    (modify-syntax-entry ?< ".")
    (modify-syntax-entry ?= ".")
    (modify-syntax-entry ?! ".")

    (syntax-table))
  "`hush-mode' syntax table.")

(defvar hush-font-lock-keywords
  (concat "\\<\\(" (regexp-opt hush-keywords) "\\)\\>" ))

(defvar hush-font-lock
  `((,hush-font-lock-keywords . font-lock-keyword-face)))

(defun hush-font-lock-setup ()
  "Set up Hush font lock."
  (setq-local font-lock-defaults '((hush-font-lock) nil t)))

(defvar hush-mode-map (make-sparse-keymap) "The keymap for Hush scripts")
;; (define-key hush-mode-map (kbd "C-c t") 'find-file)

(define-derived-mode hush-mode lua-mode "hush" ()
  :syntax-table hush-mode-syntax-table
  (setq-local comment-start "# "
              comment-start-skip "##*[ \t]*"
              comment-use-syntax t)
  (hush-font-lock-setup))

(add-to-list 'auto-mode-alist '("\\.hsh\\'" . hush-mode))


;; Babel:
(defun org-babel-execute:hush (body params)
  "Execute a block of Hush code with org-babel."
  (org-babel-eval "hush" body))
