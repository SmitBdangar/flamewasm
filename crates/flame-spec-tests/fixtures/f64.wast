;; Minimal f64 spec fixture
(module
  (func (export "add") (param f64 f64) (result f64) (f64.add (local.get 0) (local.get 1)))
  (func (export "sub") (param f64 f64) (result f64) (f64.sub (local.get 0) (local.get 1)))
  (func (export "mul") (param f64 f64) (result f64) (f64.mul (local.get 0) (local.get 1)))
  (func (export "div") (param f64 f64) (result f64) (f64.div (local.get 0) (local.get 1)))
  (func (export "sqrt")(param f64)     (result f64) (f64.sqrt (local.get 0)))
  (func (export "promote")(param f32)  (result f64) (f64.promote_f32 (local.get 0)))
)

(assert_return (invoke "add" (f64.const 1.5) (f64.const 2.5)) (f64.const 4.0))
(assert_return (invoke "mul" (f64.const 3.0) (f64.const 3.0)) (f64.const 9.0))
(assert_return (invoke "div" (f64.const 1.0) (f64.const 4.0)) (f64.const 0.25))
(assert_return (invoke "promote" (f32.const 1.0)) (f64.const 1.0))
