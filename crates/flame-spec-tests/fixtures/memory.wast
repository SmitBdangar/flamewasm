;; Memory spec fixture
(module
  (memory 1)
  (export "memory" (memory 0))
  (func (export "size") (result i32) (memory.size))
  (func (export "grow") (param i32) (result i32) (memory.grow (local.get 0)))
  (func (export "load32") (param i32) (result i32) (i32.load (local.get 0)))
  (func (export "store32") (param i32 i32) (i32.store (local.get 0) (local.get 1)))
)

(assert_return (invoke "size") (i32.const 1))
(assert_return (invoke "grow" (i32.const 1)) (i32.const 1))
(assert_return (invoke "size") (i32.const 2))
(assert_return (invoke "store32" (i32.const 0) (i32.const 42)))
(assert_return (invoke "load32" (i32.const 0)) (i32.const 42))
(assert_trap   (invoke "load32" (i32.const 0x20000)) "out of bounds memory access")
