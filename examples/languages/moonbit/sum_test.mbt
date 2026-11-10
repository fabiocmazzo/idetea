///|
test "sum_to" {
  assert_eq(@example.sum_to(10), 55)
  assert_eq(@example.sum_to(0), 0)
}

///|
test "greet" {
  assert_eq(@example.greet("TEA"), "Hello, TEA! 1 + ... + 10 = 55")
}
