;; Memory operations fixture
(module
  (memory 1)
  (export "memory" (memory 0))

  (func $store_load (export "store_load") (param $addr i32) (param $val i32) (result i32)
    (i32.store (local.get $addr) (local.get $val))
    (i32.load (local.get $addr))
  )

  (func $mem_size (export "mem_size") (result i32)
    (memory.size)
  )

  (func $mem_grow (export "mem_grow") (param $delta i32) (result i32)
    (memory.grow (local.get $delta))
  )
)
