;; OOB trap fixture — designed to trigger a memory out-of-bounds trap
(module
  (memory 1)
  (func $oob (export "oob_load") (result i32)
    ;; Access beyond the 1-page (64 KiB) boundary
    (i32.load (i32.const 0x10000))
  )
)
