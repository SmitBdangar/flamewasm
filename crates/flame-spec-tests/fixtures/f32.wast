;; Minimal f32 spec fixture
(module
  (func (export "add") (param f32 f32) (result f32) (f32.add (local.get 0) (local.get 1)))
  (func (export "sub") (param f32 f32) (result f32) (f32.sub (local.get 0) (local.get 1)))
  (func (export "mul") (param f32 f32) (result f32) (f32.mul (local.get 0) (local.get 1)))
  (func (export "div") (param f32 f32) (result f32) (f32.div (local.get 0) (local.get 1)))
  (func (export "sqrt")(param f32)     (result f32) (f32.sqrt (local.get 0)))
  (func (export "abs") (param f32)     (result f32) (f32.abs  (local.get 0)))
  (func (export "neg") (param f32)     (result f32) (f32.neg  (local.get 0)))
)

(assert_return (invoke "add" (f32.const 1.5) (f32.const 2.5)) (f32.const 4.0))
(assert_return (invoke "sub" (f32.const 5.0) (f32.const 1.0)) (f32.const 4.0))
(assert_return (invoke "mul" (f32.const 2.0) (f32.const 3.0)) (f32.const 6.0))
(assert_return (invoke "div" (f32.const 6.0) (f32.const 2.0)) (f32.const 3.0))
(assert_return (invoke "abs" (f32.const -1.0)) (f32.const 1.0))
(assert_return (invoke "neg" (f32.const 1.0)) (f32.const -1.0))
