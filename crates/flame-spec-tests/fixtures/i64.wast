;; Minimal i64 spec fixture
(module
  (func (export "add") (param i64 i64) (result i64) (i64.add (local.get 0) (local.get 1)))
  (func (export "sub") (param i64 i64) (result i64) (i64.sub (local.get 0) (local.get 1)))
  (func (export "mul") (param i64 i64) (result i64) (i64.mul (local.get 0) (local.get 1)))
  (func (export "eqz") (param i64) (result i32) (i64.eqz (local.get 0)))
  (func (export "eq")  (param i64 i64) (result i32) (i64.eq  (local.get 0) (local.get 1)))
  (func (export "extend_s") (param i32) (result i64) (i64.extend_i32_s (local.get 0)))
  (func (export "extend_u") (param i32) (result i64) (i64.extend_i32_u (local.get 0)))
)

(assert_return (invoke "add" (i64.const 1) (i64.const 1)) (i64.const 2))
(assert_return (invoke "sub" (i64.const 10) (i64.const 3)) (i64.const 7))
(assert_return (invoke "mul" (i64.const 5) (i64.const 6)) (i64.const 30))
(assert_return (invoke "eqz" (i64.const 0)) (i32.const 1))
(assert_return (invoke "eqz" (i64.const 1)) (i32.const 0))
(assert_return (invoke "eq"  (i64.const 42) (i64.const 42)) (i32.const 1))
(assert_return (invoke "extend_s" (i32.const -1)) (i64.const -1))
(assert_return (invoke "extend_u" (i32.const -1)) (i64.const 4294967295))
